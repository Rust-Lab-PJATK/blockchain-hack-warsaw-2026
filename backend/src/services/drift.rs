use std::borrow::Cow;

use drift_rs::{types::MarketId, DriftClient, RpcClient, TransactionBuilder, Wallet};
use loco_rs::prelude::*;

pub type SignatureText = String;

#[derive(Clone, Copy, Debug)]
pub enum PositionSide {
    Long,
    Short,
}

pub struct DriftService {
    client: DriftClient,
}

impl DriftService {
    pub async fn new(ctx: &AppContext) -> Self {
        let client = create_drift_client(ctx).await.expect("drift client create");

        Self { client }
    }

    pub async fn open_perp_position(
        &self,
        market_index: u16,
        side: PositionSide,
        base_asset_amount: u64,
    ) -> Result<SignatureText> {
        if base_asset_amount == 0 {
            return Err(Error::BadRequest(
                "base_asset_amount must be > 0".to_string(),
            ));
        }

        let sub_account = self.client.wallet().default_sub_account();
        let user = self
            .client
            .get_user_account(&sub_account)
            .await
            .map_err(Error::wrap)?;

        let signed_amount = i64::try_from(base_asset_amount)
            .map_err(|_| Error::BadRequest("base_asset_amount too large".to_string()))?;
        let signed_amount = match side {
            PositionSide::Long => signed_amount,
            PositionSide::Short => -signed_amount,
        };

        let order = drift_rs::types::NewOrder::market(MarketId::perp(market_index))
            .amount(signed_amount)
            .build();
        let tx = TransactionBuilder::new(
            self.client.program_data(),
            sub_account,
            Cow::Borrowed(&user),
            false,
        )
        .place_orders(vec![order])
        .build();

        let sig = self.client.sign_and_send(tx).await.map_err(Error::wrap)?;
        Ok(sig.to_string())
    }

    pub async fn close_perp_position(&self, market_index: u16) -> Result<SignatureText> {
        let sub_account = self.client.wallet().default_sub_account();
        let user = self
            .client
            .get_user_account(&sub_account)
            .await
            .map_err(Error::wrap)?;

        let position = self
            .client
            .perp_position(&sub_account, market_index)
            .await
            .map_err(Error::wrap)?
            .ok_or_else(|| {
                Error::BadRequest(format!("no perp position for market {market_index}"))
            })?;

        if position.base_asset_amount == 0 {
            return Err(Error::BadRequest("position is already flat".to_string()));
        }

        let close_amount = position.base_asset_amount.unsigned_abs();
        let signed_close_amount = if position.base_asset_amount > 0 {
            -(i64::try_from(close_amount)
                .map_err(|_| Error::BadRequest("position size too large".to_string()))?)
        } else {
            i64::try_from(close_amount)
                .map_err(|_| Error::BadRequest("position size too large".to_string()))?
        };

        let close_order = drift_rs::types::NewOrder::market(MarketId::perp(market_index))
            .amount(signed_close_amount)
            .reduce_only(true)
            .build();
        let tx = TransactionBuilder::new(
            self.client.program_data(),
            sub_account,
            Cow::Borrowed(&user),
            false,
        )
        .place_orders(vec![close_order])
        .build();

        let sig = self.client.sign_and_send(tx).await.map_err(Error::wrap)?;
        Ok(sig.to_string())
    }
}

async fn create_drift_client(ctx: &AppContext) -> Result<DriftClient> {
    let helius_api_key = ctx
        .config
        .settings
        .as_ref()
        .and_then(|s| s.get("helius_api_key"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Message("helius_api_key not set in config".to_string()))?;

    let rpc_url = format!("https://devnet.helius-rpc.com/?api-key={helius_api_key}");
    let wallet_path = ctx
        .config
        .settings
        .as_ref()
        .and_then(|s| s.get("wallet_private_key"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Message("wallet_private_key not set in config".to_string()))?;

    let wallet = Wallet::try_from_str(wallet_path).map_err(Error::wrap)?;
    let client = DriftClient::new(
        drift_rs::types::Context::DevNet,
        RpcClient::new(rpc_url),
        wallet,
    )
    .await
    .map_err(Error::wrap)?;

    Ok(client)
}

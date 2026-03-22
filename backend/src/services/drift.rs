use std::borrow::Cow;

use drift_rs::{
    types::{accounts::User, MarketId},
    DriftClient, RpcClient, TransactionBuilder, Wallet,
};
use loco_rs::prelude::*;

const SUB_ACCOUNT_ID: u16 = 0;

pub type SignatureText = String;

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum PerpMarket {
    SOL = 0,
    BTC,
    ETH,
    APT,
    BONK,
    POL,
    ARB,
    DOGE,
    BNB,
    SUI,
    PEPE,
    OP,
    RENDER,
    XRP,
    HNT,
    INJ,
    LINK,
    RLB,
    PYTH,
    TIA,
    JTO,
    SEI,
    AVAX,
    W,
    KMNO,
    WEN,
    TrumpWin2024,
    KamalaPopularVote2024,
    Random2024,
    NVDA,
}

#[derive(Debug, Clone, Copy)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy)]
pub enum PerpAmount {
    // Amount in Drift SDK native units - in perpetual orders it's ACTUAL BUY AMOUNT * 1 000 000 000
    NativeUnits(u64),
    // Actual amount you will see in Drift order book
    ActualUnits(f64),
}

impl PerpAmount {
    pub fn to_native_units(&self) -> u64 {
        match self {
            Self::ActualUnits(a) => (a * 1_000_000_000.) as u64,
            Self::NativeUnits(a) => *a,
        }
    }
}

pub struct DriftService {
    client: DriftClient,
}

impl DriftService {
    pub async fn new(ctx: &AppContext) -> Result<Self> {
        let client = create_drift_client(ctx).await?;

        Ok(Self { client })
    }

    pub async fn initialize_user_pda(&self) -> Result<SignatureText> {
        let sub_account = self.client.wallet().sub_account(SUB_ACCOUNT_ID);

        if self.client.get_user_account(&sub_account).await.is_ok() {
            return Err(Error::BadRequest(format!(
                "drift subaccount {} already initialized: {}",
                SUB_ACCOUNT_ID, sub_account
            )));
        }

        let mut placeholder_user = User::default();
        placeholder_user.authority = *self.client.wallet().authority();
        placeholder_user.sub_account_id = SUB_ACCOUNT_ID;
        if self.client.wallet().is_delegated() {
            placeholder_user.delegate = self.client.wallet().signer();
        }

        let tx = TransactionBuilder::new(
            self.client.program_data(),
            sub_account,
            Cow::Owned(placeholder_user),
            self.client.wallet().is_delegated(),
        )
        .initialize_user_account(SUB_ACCOUNT_ID, None, None)
        .build();

        let sig = self.client.sign_and_send(tx).await.map_err(Error::wrap)?;
        Ok(sig.to_string())
    }

    pub async fn open_perp_position(
        &self,
        market: PerpMarket,
        side: PositionSide,
        amount: PerpAmount,
    ) -> Result<SignatureText> {
        let normalized_amount = self.normalize_open_amount(market as u16, amount).await?;

        self.initialize_user_pda().await.ok();
        let sub_account = self.client.wallet().sub_account(SUB_ACCOUNT_ID);
        let user = self
            .client
            .get_user_account(&sub_account)
            .await
            .map_err(Error::wrap)?;

        let signed_amount = i64::try_from(normalized_amount)
            .map_err(|_| Error::BadRequest("base_asset_amount too large".to_string()))?;
        let signed_amount = match side {
            PositionSide::Long => signed_amount,
            PositionSide::Short => -signed_amount,
        };

        let order = drift_rs::types::NewOrder::market(MarketId::perp(market as u16))
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

    pub async fn close_perp_position(&self, market: PerpMarket) -> Result<SignatureText> {
        self.initialize_user_pda().await.ok();
        let sub_account = self.client.wallet().sub_account(SUB_ACCOUNT_ID);
        let user = self
            .client
            .get_user_account(&sub_account)
            .await
            .map_err(Error::wrap)?;

        let position = self
            .client
            .perp_position(&sub_account, market as u16)
            .await
            .map_err(Error::wrap)?
            .ok_or_else(|| Error::BadRequest(format!("no perp position for market {market:?}")))?;

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

        let close_order = drift_rs::types::NewOrder::market(MarketId::perp(market as u16))
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

    async fn normalize_open_amount(&self, market_index: u16, amount: PerpAmount) -> Result<u64> {
        let mut native_amount = amount.to_native_units();

        let market = self
            .client
            .get_perp_market_account(market_index)
            .await
            .map_err(Error::wrap)?;

        let min = market.amm.min_order_size;
        if native_amount < min {
            return Err(Error::BadRequest(format!(
                "requested amount is lower than min_order_size, {native_amount} < {min}"
            )));
        }

        let step = market.amm.order_step_size.max(1);
        let rem = native_amount % step;
        if rem != 0 {
            native_amount = native_amount.saturating_add(step - rem);
        }

        Ok(native_amount)
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
    let rpc = RpcClient::new(rpc_url);

    let wallet_private_key = ctx
        .config
        .settings
        .as_ref()
        .and_then(|s| s.get("wallet_private_key"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Message("wallet_private_key not set in config".to_string()))?;
    let wallet = Wallet::try_from_str(wallet_private_key).map_err(Error::wrap)?;

    let client = DriftClient::new(drift_rs::types::Context::DevNet, rpc, wallet)
        .await
        .map_err(Error::wrap)?;

    Ok(client)
}

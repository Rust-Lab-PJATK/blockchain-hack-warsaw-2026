use loco_rs::prelude::*;
use std::collections::HashMap;

pub type SignatureText = String;
pub type MarketData = HashMap<String, f64>;

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
    /// Amount in Drift SDK native units - in perpetual orders it's ACTUAL BUY AMOUNT * 1 000 000 000
    NativeUnits(u64),
    /// Actual amount you will see in Drift order book
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

/// Variables available for Lua condition evaluation, returned by `get_market_data`.
/// 
/// Data is fetched from Drift's on-chain oracle (Pyth oracle integration via Drift SDK).
/// Some fields use current price as approximation since Drift SDK doesn't provide historical data.
pub const MARKET_DATA_VARIABLES: &[(&str, &str)] = &[
    ("price", "Real oracle price from Drift's on-chain oracle/Pyth (precision: 1e6). Updated on-chain based on oracle feeds."),
    ("volume", "Not available from Drift SDK. Returns 0.0 (would require external market data source)."),
    ("high_24h", "24h high price estimate (approximated using current oracle price due to SDK limitations)."),
    ("low_24h", "24h low price estimate (approximated using current oracle price due to SDK limitations)."),
    ("open_24h", "24h open price estimate (approximated using current oracle price due to SDK limitations)."),
    ("change_pct", "24h price change percentage. Not available from Drift SDK. Returns 0.0 (would require historical price snapshots)."),
];

#[async_trait]
pub trait DriftProvider: Send + Sync {
    async fn initialize_user_pda(&self) -> Result<SignatureText>;
    async fn open_perp_position(
        &self,
        market: PerpMarket,
        side: PositionSide,
        amount: PerpAmount,
    ) -> Result<SignatureText>;
    async fn close_perp_position(&self, market: PerpMarket) -> Result<SignatureText>;
    /// Fetch current market data for a given symbol. Returns variable name → value map
    /// usable by the Lua condition evaluator.
    async fn get_market_data(&self, symbol: &str) -> Result<MarketData>;
    /// Get the unrealized PnL percentage for an open position on a given market.
    /// Returns None if no position is open.
    async fn get_position_pnl(&self, market: PerpMarket) -> Result<Option<f64>>;
    /// Get the current price for a specific perp market (for absolute stop-loss checks).
    async fn get_current_price(&self, market: PerpMarket) -> Result<f64>;
    /// Get the user's available balance (free collateral) in USD.
    async fn get_user_balance(&self) -> Result<f64>;
}

// ── Real implementation (requires drift feature + Solana RPC) ──────────────

#[cfg(feature = "drift")]
mod real {
    use super::*;
    use std::borrow::Cow;

    use drift_rs::{
        types::{accounts::User, MarketId},
        DriftClient, RpcClient, TransactionBuilder, Wallet,
    };

    const SUB_ACCOUNT_ID: u16 = 0;

    /// Map symbol strings (case-insensitive) to PerpMarket enum values
    fn symbol_to_perp_market(symbol: &str) -> Result<PerpMarket> {
        use PerpMarket::*;
        match symbol.to_uppercase().as_str() {
            "SOL" => Ok(SOL),
            "BTC" => Ok(BTC),
            "ETH" => Ok(ETH),
            "APT" => Ok(APT),
            "BONK" => Ok(BONK),
            "POL" => Ok(POL),
            "ARB" => Ok(ARB),
            "DOGE" => Ok(DOGE),
            "BNB" => Ok(BNB),
            "SUI" => Ok(SUI),
            "PEPE" => Ok(PEPE),
            "OP" => Ok(OP),
            "RENDER" => Ok(RENDER),
            "XRP" => Ok(XRP),
            "HNT" => Ok(HNT),
            "INJ" => Ok(INJ),
            "LINK" => Ok(LINK),
            "RLB" => Ok(RLB),
            "PYTH" => Ok(PYTH),
            "TIA" => Ok(TIA),
            "JTO" => Ok(JTO),
            "SEI" => Ok(SEI),
            "AVAX" => Ok(AVAX),
            "W" => Ok(W),
            "KMNO" => Ok(KMNO),
            "WEN" => Ok(WEN),
            "TRUMPWIN2024" => Ok(TrumpWin2024),
            "KAMALAPOPULARVOTE2024" => Ok(KamalaPopularVote2024),
            "RANDOM2024" => Ok(Random2024),
            "NVDA" => Ok(NVDA),
            _ => Err(Error::BadRequest(format!(
                "unknown perp market symbol: {}",
                symbol
            ))),
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
    }

    #[async_trait]
    impl DriftProvider for DriftService {
        async fn initialize_user_pda(&self) -> Result<SignatureText> {
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

        async fn open_perp_position(
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

        async fn close_perp_position(&self, market: PerpMarket) -> Result<SignatureText> {
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
                .ok_or_else(|| {
                    Error::BadRequest(format!("no perp position for market {market:?}"))
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

        async fn get_market_data(&self, symbol: &str) -> Result<MarketData> {
            let market = symbol_to_perp_market(symbol)?;
            let market_account = self
                .client
                .get_perp_market_account(market as u16)
                .await
                .map_err(Error::wrap)?;
            
            let price = market_account.amm.historical_oracle_data.last_oracle_price as f64 / 1_000_000.0;
            let mark_price_twap_5min = market_account.amm.last_mark_price_twap5min as f64 / 1_000_000.0;
            
            let volume_24h = market_account.amm.volume24h as f64;
            
            let mut data = HashMap::new();
            data.insert("price".to_string(), price);
            data.insert("volume".to_string(), volume_24h);
            data.insert("volume_24h".to_string(), volume_24h);
            data.insert("high_24h".to_string(), price);  // Use current price as approximation
            data.insert("low_24h".to_string(), price);   // Use current price as approximation
            data.insert("open_24h".to_string(), mark_price_twap_5min);  // Use 5min TWAP as proxy
            data.insert("change_pct".to_string(), 0.0);  // Would need historical data
            
            Ok(data)
        }

        async fn get_position_pnl(&self, market: PerpMarket) -> Result<Option<f64>> {
            let sub_account = self.client.wallet().sub_account(SUB_ACCOUNT_ID);
            let position = self
                .client
                .perp_position(&sub_account, market as u16)
                .await
                .map_err(Error::wrap)?;

            match position {
                Some(p) if p.base_asset_amount != 0 => {
                    // Simplified PnL: (quote_entry - quote_break_even) / quote_entry * 100
                    let entry = p.quote_entry_amount as f64;
                    let breakeven = p.quote_break_even_amount as f64;
                    if entry.abs() < f64::EPSILON {
                        return Ok(Some(0.0));
                    }
                    Ok(Some((entry - breakeven) / entry.abs() * 100.0))
                }
                _ => Ok(None),
            }
        }

        async fn get_current_price(&self, market: PerpMarket) -> Result<f64> {
            let market_account = self
                .client
                .get_perp_market_account(market as u16)
                .await
                .map_err(Error::wrap)?;
            // Oracle price is stored as fixed-point with PRICE_PRECISION (1e6)
            let price = market_account.amm.historical_oracle_data.last_oracle_price as f64 / 1_000_000.0;
            Ok(price)
        }

        async fn get_user_balance(&self) -> Result<f64> {
            let sub_account = self.client.wallet().sub_account(SUB_ACCOUNT_ID);
            let user = self
                .client
                .get_user_account(&sub_account)
                .await
                .map_err(Error::wrap)?;
            // total_collateral is in QUOTE_PRECISION (1e6)
            // Use settled_perp_pnl + total deposits as approximation
            let balance = user.total_deposits as f64 / 1_000_000.0;
            Ok(balance)
        }
    }

    impl DriftService {
        async fn normalize_open_amount(
            &self,
            market_index: u16,
            amount: PerpAmount,
        ) -> Result<u64> {
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
}

#[cfg(feature = "drift")]
pub use real::DriftService;

// ── Mock implementation (no drift-rs dependency) ───────────────────────────

#[cfg(not(feature = "drift"))]
mod mock {
    use super::*;

    pub struct MockDriftService;

    impl MockDriftService {
        pub async fn new(_ctx: &AppContext) -> Result<Self> {
            tracing::warn!("drift feature is disabled — using mock DriftService");
            Ok(Self)
        }
    }

    #[async_trait]
    impl DriftProvider for MockDriftService {
        async fn initialize_user_pda(&self) -> Result<SignatureText> {
            tracing::info!("[mock] initialize_user_pda called");
            Ok("mock_signature_init_user_pda".to_string())
        }

        async fn open_perp_position(
            &self,
            market: PerpMarket,
            side: PositionSide,
            amount: PerpAmount,
        ) -> Result<SignatureText> {
            tracing::info!(
                "[mock] open_perp_position: market={market:?}, side={side:?}, amount={}",
                amount.to_native_units()
            );
            Ok(format!(
                "mock_signature_open_{:?}_{:?}",
                market, side
            ))
        }

        async fn close_perp_position(&self, market: PerpMarket) -> Result<SignatureText> {
            tracing::info!("[mock] close_perp_position: market={market:?}");
            Ok(format!("mock_signature_close_{:?}", market))
        }

        async fn get_market_data(&self, symbol: &str) -> Result<MarketData> {
            tracing::info!("[mock] get_market_data: symbol={symbol}");
            let mut data = HashMap::new();
            // Return plausible mock data so conditions can be tested locally
            data.insert("price".to_string(), 125.0);
            data.insert("volume".to_string(), 5_000_000.0);
            data.insert("high_24h".to_string(), 130.0);
            data.insert("low_24h".to_string(), 120.0);
            data.insert("open_24h".to_string(), 123.0);
            data.insert("change_pct".to_string(), 1.6);
            Ok(data)
        }

        async fn get_position_pnl(&self, market: PerpMarket) -> Result<Option<f64>> {
            tracing::info!("[mock] get_position_pnl: market={market:?}");
            // Mock: return a small positive PnL
            Ok(Some(2.5))
        }

        async fn get_current_price(&self, market: PerpMarket) -> Result<f64> {
            tracing::info!("[mock] get_current_price: market={market:?}");
            Ok(125.0)
        }

        async fn get_user_balance(&self) -> Result<f64> {
            tracing::info!("[mock] get_user_balance");
            Ok(10_000.0)
        }
    }
}

#[cfg(not(feature = "drift"))]
pub use mock::MockDriftService;

/// Creates the appropriate drift provider based on the enabled feature.
pub async fn create_drift_provider(
    ctx: &AppContext,
) -> Result<Box<dyn DriftProvider>> {
    #[cfg(feature = "drift")]
    {
        let service = DriftService::new(ctx).await?;
        Ok(Box::new(service))
    }
    #[cfg(not(feature = "drift"))]
    {
        let service = MockDriftService::new(ctx).await?;
        Ok(Box::new(service))
    }
}

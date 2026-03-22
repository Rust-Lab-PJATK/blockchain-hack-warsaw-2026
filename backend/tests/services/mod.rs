use backend::{
    app::App,
    services::drift::{DriftService, PerpAmount, PerpMarket, PositionSide},
};
use loco_rs::prelude::request;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_open_position_long() {
    dotenvy::dotenv().ok();

    request::<App, _, _>(|_request, ctx| async move {
        let service = DriftService::new(&ctx).await.expect("service init");
        let result = service
            .open_perp_position(
                PerpMarket::SOL,
                PositionSide::Long,
                PerpAmount::ActualUnits(0.01),
            )
            .await;

        assert!(result.is_ok())
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_close_position_long() {
    dotenvy::dotenv().ok();

    request::<App, _, _>(|_request, ctx| async move {
        let service = DriftService::new(&ctx).await.expect("service init");
        let result = service.close_perp_position(PerpMarket::SOL).await;

        assert!(result.is_ok())
    })
    .await;
}

use backend::{
    app::App,
    services::drift::{DriftService, PositionSide},
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
            .open_perp_position(0, PositionSide::Long, 20_000_000)
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
        let result = service.close_perp_position(0).await;

        assert!(result.is_ok())
    })
    .await;
}

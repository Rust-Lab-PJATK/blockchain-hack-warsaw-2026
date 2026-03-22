#[cfg(feature = "drift")]
mod drift_tests {
    use backend::{
        app::App,
        services::drift::{DriftService, DriftProvider, PerpAmount, PerpMarket, PositionSide},
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
}

#[cfg(not(feature = "drift"))]
mod drift_mock_tests {
    use backend::{
        app::App,
        services::drift::{MockDriftService, DriftProvider, PerpAmount, PerpMarket, PositionSide},
    };
    use loco_rs::prelude::request;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_mock_open_position_long() {
        request::<App, _, _>(|_request, ctx| async move {
            let service = MockDriftService::new(&ctx).await.expect("mock service init");
            let result = service
                .open_perp_position(
                    PerpMarket::SOL,
                    PositionSide::Long,
                    PerpAmount::ActualUnits(0.01),
                )
                .await;

            assert!(result.is_ok());
            assert!(result.unwrap().starts_with("mock_signature"));
        })
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn test_mock_close_position_long() {
        request::<App, _, _>(|_request, ctx| async move {
            let service = MockDriftService::new(&ctx).await.expect("mock service init");
            let result = service.close_perp_position(PerpMarket::SOL).await;

            assert!(result.is_ok());
            assert!(result.unwrap().starts_with("mock_signature"));
        })
        .await;
    }
}

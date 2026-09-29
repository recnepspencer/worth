use std::io;
use std::sync::Arc;
use std::time::Duration;

use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundTerminalObservation,
};

use super::super::inbound_completion::{
    BankRailCloseAssessment, BankRailCompletionRoute, BankRailMaintenanceBatch,
};
use super::super::BankHttpServer;

struct PendingRoute;

impl BankRailCompletionRoute for PendingRoute {
    fn receive_and_sign(
        &self,
        _: &[u8],
        _: &WorthQueryRequestScope,
    ) -> Result<(Vec<u8>, bool), WorthQueryInboundAdmissionDenial> {
        unreachable!()
    }

    fn observe_terminal(&self, _: [u8; 32]) -> Option<WorthQueryInboundTerminalObservation> {
        None
    }

    fn maintain_custody(
        &self,
        _: &WorthQueryRequestScope,
    ) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial> {
        Ok(BankRailMaintenanceBatch {
            remaining: 1,
            ..Default::default()
        })
    }
}

#[tokio::test]
async fn failed_http_worker_preserves_rail_route_in_close_failure() {
    let (shutdown, _receiver) = tokio::sync::oneshot::channel();
    let (live_shutdown, _) = tokio::sync::watch::channel(false);
    let server = BankHttpServer {
        local_address: "127.0.0.1:1".parse().unwrap(),
        rail_completion: Some(Arc::new(PendingRoute)),
        rail_maintenance_wake: None,
        shutdown: Some(shutdown),
        live_shutdown,
        server_task: Some(tokio::spawn(async {
            Err(io::Error::other("injected HTTP worker failure"))
        })),
        dispatcher_task: None,
        continuation_task: None,
        elevation_task: None,
        recovery_task: None,
        rail_maintenance_task: None,
        rail_maintenance_deadline: Duration::from_secs(1),
        live_thread: None,
    };
    let failure = match server.shutdown().await {
        Ok(_) => panic!("worker failure must be reported"),
        Err(failure) => failure,
    };
    assert_eq!(failure.error().to_string(), "injected HTTP worker failure");
    let close = failure.into_close();
    let rail = close.into_rail_completion().expect("rail route retained");
    assert_eq!(
        rail.assessment(),
        BankRailCloseAssessment::Assessed {
            known_remaining: 1,
            outstanding_dispatches: 0,
            retained_accepted_occurrences: 0,
            blocked: 0,
            unavailable_routes: 0,
            next_expiry_unix_seconds: None,
        }
    );
    assert!(matches!(
        rail.into_continuation().continue_once().await,
        BankRailCloseAssessment::Assessed {
            known_remaining: 1,
            ..
        }
    ));
}

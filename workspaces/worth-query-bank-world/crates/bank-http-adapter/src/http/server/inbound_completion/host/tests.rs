use std::time::Duration;

use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundTerminalObservation,
};

use super::*;
use crate::http::server::inbound_completion::route::{
    BankRailCompletionRoute, BankRailMaintenanceBatch,
};
use crate::http::server::inbound_completion::BankRailCloseAssessment;

struct UnresolvedRoute;

impl BankRailCompletionRoute for UnresolvedRoute {
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
            blocked: 1,
            ..Default::default()
        })
    }
}

#[tokio::test]
async fn failed_callback_worker_still_returns_installed_continuation() {
    let (shutdown, _receiver) = oneshot::channel();
    let (maintenance_shutdown, _) = watch::channel(false);
    let server = BankRailCallbackServer {
        address: "127.0.0.1:1".parse().unwrap(),
        maximum_deadline: Duration::from_secs(1),
        route: Arc::new(UnresolvedRoute),
        wake: Arc::new(Notify::new()),
        shutdown: Some(shutdown),
        maintenance_shutdown,
        server_task: tokio::spawn(async { Ok(()) }),
        maintenance_task: tokio::spawn(async {
            Err(io::Error::other("injected maintenance failure"))
        }),
    };
    let failure = match server.shutdown().await {
        Ok(_) => panic!("worker failure must be reported"),
        Err(failure) => failure,
    };
    assert_eq!(failure.error().to_string(), "injected maintenance failure");
    let close = failure.into_close();
    let expected = close_assessment_with_one_pending();
    assert_eq!(close.assessment(), expected);
    assert_eq!(close.into_continuation().continue_once().await, expected);
}

fn close_assessment_with_one_pending() -> BankRailCloseAssessment {
    BankRailCloseAssessment::Assessed {
        known_remaining: 1,
        outstanding_dispatches: 0,
        retained_accepted_occurrences: 0,
        blocked: 1,
        unavailable_routes: 0,
        next_expiry_unix_seconds: None,
    }
}

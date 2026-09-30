use std::sync::atomic::{AtomicUsize, Ordering};

use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundTerminalObservation,
};

use super::*;
use crate::http::server::inbound_completion::route::BankRailMaintenanceBatch;

struct RetainedRoute {
    calls: AtomicUsize,
}

impl BankRailCompletionRoute for RetainedRoute {
    fn receive_and_sign(
        &self,
        _: &[u8],
        _: &WorthQueryRequestScope,
    ) -> Result<super::super::route::BankSignedCustodyOutcome, WorthQueryInboundAdmissionDenial>
    {
        unreachable!("the close court never receives a callback")
    }

    fn observe_terminal(&self, _: [u8; 32]) -> Option<WorthQueryInboundTerminalObservation> {
        None
    }

    fn maintain_custody(
        &self,
        _: &WorthQueryRequestScope,
    ) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial> {
        let turn = self.calls.fetch_add(1, Ordering::AcqRel);
        Ok(BankRailMaintenanceBatch {
            remaining: usize::from(turn == 0),
            blocked: usize::from(turn == 0),
            unavailable_routes: u8::from(turn == 0),
            ..Default::default()
        })
    }
}

#[tokio::test]
async fn orderly_close_exposes_unresolved_and_keeps_exact_route_for_later_cue() {
    let route = Arc::new(RetainedRoute {
        calls: AtomicUsize::new(0),
    });
    let continuation = BankRailCompletionContinuation::new(route.clone(), Duration::from_secs(1));
    let close = continuation.close().await;
    assert_eq!(
        close.assessment(),
        BankRailCloseAssessment::Assessed {
            known_remaining: 1,
            outstanding_dispatches: 0,
            retained_accepted_occurrences: 0,
            blocked: 1,
            unavailable_routes: 1,
            next_expiry_unix_seconds: None,
        }
    );
    assert_eq!(
        close.into_continuation().continue_once().await,
        BankRailCloseAssessment::Assessed {
            known_remaining: 0,
            outstanding_dispatches: 0,
            retained_accepted_occurrences: 0,
            blocked: 0,
            unavailable_routes: 0,
            next_expiry_unix_seconds: None,
        }
    );
    assert_eq!(route.calls.load(Ordering::Acquire), 2);
}

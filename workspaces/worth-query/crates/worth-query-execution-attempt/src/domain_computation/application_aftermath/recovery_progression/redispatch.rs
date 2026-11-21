//! Performed re-dispatch evidence backed by `worth-proof` effect law.

use worth_proof::{ActionMarker, Performed};

use crate::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleAuthorityIdentity;
use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
use crate::domain_computation::primary_graph::WorthQueryPerformedExternalRedispatchSeal;

worth_proof::authority_marker!(WorthQueryExternalRedispatchAuthority);

struct WorthQueryExternalRedispatchAction;
impl ActionMarker for WorthQueryExternalRedispatchAction {}

#[derive(Debug, Eq, PartialEq)]
struct WorthQueryExternalRedispatchOutcome {
    handle: WorthQueryRecoveryHandleAuthorityIdentity,
    dispatch: WorthQueryExternalEffectDispatch,
}

/// Evidence that the runtime performed one re-dispatch through the sole
/// external-effect classification boundary.
#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryPerformedExternalRedispatch {
    performed: Performed<
        WorthQueryExternalRedispatchAction,
        WorthQueryExternalRedispatchAuthority,
        WorthQueryExternalRedispatchOutcome,
    >,
}

impl WorthQueryPerformedExternalRedispatch {
    pub fn record(seal: WorthQueryPerformedExternalRedispatchSeal) -> Self {
        let (handle, dispatch) = seal.into_parts();
        Self {
            performed: Performed::record(
                &WorthQueryExternalRedispatchAuthority::witness(),
                WorthQueryExternalRedispatchOutcome { handle, dispatch },
            ),
        }
    }

    pub fn handle_authority(&self) -> WorthQueryRecoveryHandleAuthorityIdentity {
        self.performed.outcome().handle
    }

    pub fn dispatch(&self) -> &WorthQueryExternalEffectDispatch {
        &self.performed.outcome().dispatch
    }

    pub fn into_dispatch(self) -> WorthQueryExternalEffectDispatch {
        self.performed.into_outcome().dispatch
    }
}

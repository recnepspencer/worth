use std::sync::Arc;

use crate::domain_computation::vocabulary::managed_run::safe_point_observation::WorthQueryManagedSafePointObservation;
use worth_runtime_bridge::facade::{
    BridgeBoundExecutionBasis, BridgeExecutionSafePointFailureKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedSafePointFailureKind {
    BridgeObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedSafePointFailure {
    run_identity: Arc<str>,
    kind: WorthQueryManagedSafePointFailureKind,
    bridge_kind: Option<BridgeExecutionSafePointFailureKind>,
    detail: Arc<str>,
}

impl WorthQueryManagedSafePointFailure {
    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub const fn kind(&self) -> WorthQueryManagedSafePointFailureKind {
        self.kind
    }

    pub const fn bridge_kind(&self) -> Option<BridgeExecutionSafePointFailureKind> {
        self.bridge_kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub(super) fn observe_managed_run_safe_point(
    run_identity: &Arc<str>,
    bridge_basis: &BridgeBoundExecutionBasis,
) -> Result<WorthQueryManagedSafePointObservation, WorthQueryManagedSafePointFailure> {
    let bridge_evidence =
        bridge_basis
            .observe_safe_point()
            .map_err(|failure| WorthQueryManagedSafePointFailure {
                run_identity: Arc::clone(run_identity),
                kind: WorthQueryManagedSafePointFailureKind::BridgeObservation,
                bridge_kind: Some(failure.kind()),
                detail: Arc::from(failure.detail()),
            })?;
    Ok(WorthQueryManagedSafePointObservation {
        run_identity: Arc::clone(run_identity),
        bridge_evidence,
    })
}

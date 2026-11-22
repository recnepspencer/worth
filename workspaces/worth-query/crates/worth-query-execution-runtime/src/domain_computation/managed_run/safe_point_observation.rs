use std::sync::Arc;

use worth_runtime_bridge::facade::{
    BridgeBoundExecutionBasis, BridgeExecutionQueuePressureState, BridgeExecutionSafePointCounters,
    BridgeExecutionSafePointFailureKind, BridgeExecutionSafePointObservation,
    BridgeExecutionSafePointSignalState,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedSafePointObservation {
    run_identity: Arc<str>,
    bridge_evidence: BridgeExecutionSafePointObservation,
}

impl WorthQueryManagedSafePointObservation {
    pub fn run_identity(&self) -> &str {
        &self.run_identity
    }

    pub fn bridge_evidence(&self) -> &BridgeExecutionSafePointObservation {
        &self.bridge_evidence
    }

    pub const fn signal_state(&self) -> BridgeExecutionSafePointSignalState {
        self.bridge_evidence.signal_state()
    }

    pub const fn pressure_state(&self) -> BridgeExecutionQueuePressureState {
        self.bridge_evidence.pressure_state()
    }

    pub const fn queue_depth(&self) -> u64 {
        self.bridge_evidence.queue_depth()
    }

    pub const fn queue_capacity(&self) -> u64 {
        self.bridge_evidence.queue_capacity()
    }

    pub const fn observation_ordinal(&self) -> u64 {
        self.bridge_evidence.observation_ordinal()
    }

    pub const fn lifecycle_ordinal(&self) -> u64 {
        self.bridge_evidence.lifecycle_ordinal()
    }

    pub const fn counters(&self) -> BridgeExecutionSafePointCounters {
        self.bridge_evidence.counters()
    }
}

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

use worth_signal::facade::ResourceInFlightStatus;

use crate::source::with_async_request_signal_runtime;

use super::step_contract::BridgeManagedExecutionStepContractIdentity;
use super::{
    queue_pressure::project_pressure_state, BridgeBoundExecutionBasis,
    BridgeExecutionBasisIdentity, BridgeExecutionQueuePressureState,
    BridgeManagedExecutionIntentIdentity,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionSafePointSignalState {
    Active,
    Fulfilled,
    Rejected,
    Superseded,
    Cancelled,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeExecutionSafePointCounters {
    exact_signal_request_lookup_count: usize,
    pressure_classification_count: usize,
}

impl BridgeExecutionSafePointCounters {
    pub const fn exact_signal_request_lookup_count(self) -> usize {
        self.exact_signal_request_lookup_count
    }

    pub const fn pressure_classification_count(self) -> usize {
        self.pressure_classification_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeExecutionSafePointObservation {
    basis_identity: BridgeExecutionBasisIdentity,
    intent_identity: BridgeManagedExecutionIntentIdentity,
    step_contract_identity: BridgeManagedExecutionStepContractIdentity,
    observation_ordinal: u64,
    lifecycle_ordinal: u64,
    signal_state: BridgeExecutionSafePointSignalState,
    pressure_state: BridgeExecutionQueuePressureState,
    queue_depth: u64,
    queue_capacity: u64,
    timeout_wake_identity: Option<u64>,
    counters: BridgeExecutionSafePointCounters,
}

impl BridgeExecutionSafePointObservation {
    pub fn basis_identity(&self) -> &BridgeExecutionBasisIdentity {
        &self.basis_identity
    }

    pub fn intent_identity(&self) -> &BridgeManagedExecutionIntentIdentity {
        &self.intent_identity
    }

    pub fn step_contract_identity(&self) -> &str {
        self.step_contract_identity.as_str()
    }

    pub const fn observation_ordinal(&self) -> u64 {
        self.observation_ordinal
    }

    pub const fn lifecycle_ordinal(&self) -> u64 {
        self.lifecycle_ordinal
    }

    pub const fn signal_state(&self) -> BridgeExecutionSafePointSignalState {
        self.signal_state
    }

    pub const fn pressure_state(&self) -> BridgeExecutionQueuePressureState {
        self.pressure_state
    }

    pub const fn queue_depth(&self) -> u64 {
        self.queue_depth
    }

    pub const fn queue_capacity(&self) -> u64 {
        self.queue_capacity
    }

    pub const fn timeout_wake_identity(&self) -> Option<u64> {
        self.timeout_wake_identity
    }

    pub const fn counters(&self) -> BridgeExecutionSafePointCounters {
        self.counters
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeExecutionSafePointFailureKind {
    AtomicExecutionUnsupported,
    SignalRuntimeThreadAffinityViolation,
    SignalObservationDenied,
    SignalRequestMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeExecutionSafePointFailure {
    kind: BridgeExecutionSafePointFailureKind,
    detail: String,
}

impl BridgeExecutionSafePointFailure {
    fn new(kind: BridgeExecutionSafePointFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub const fn kind(&self) -> BridgeExecutionSafePointFailureKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl BridgeBoundExecutionBasis {
    pub fn observe_safe_point(
        &self,
    ) -> Result<BridgeExecutionSafePointObservation, BridgeExecutionSafePointFailure> {
        let managed = self.posture.managed().ok_or_else(|| {
            BridgeExecutionSafePointFailure::new(
                BridgeExecutionSafePointFailureKind::AtomicExecutionUnsupported,
                "atomic execution has no managed safe-point contract",
            )
        })?;
        let report = with_async_request_signal_runtime(self.bridge_runtime_key, |runtime| {
            runtime.observe_resource_safe_point(&managed.queue)
        })
        .map_err(|error| {
            BridgeExecutionSafePointFailure::new(
                BridgeExecutionSafePointFailureKind::SignalRuntimeThreadAffinityViolation,
                format!(
                    "bridge Signal runtime {} belongs to thread {:?}, not {:?}",
                    error.runtime_key(),
                    error.owner(),
                    error.current()
                ),
            )
        })?
        .map_err(|denial| {
            BridgeExecutionSafePointFailure::new(
                BridgeExecutionSafePointFailureKind::SignalObservationDenied,
                format!(
                    "Signal denied safe-point observation for request {}: {:?}",
                    denial.request_id().get(),
                    denial.class()
                ),
            )
        })?;
        if report.request() != self.request.request_handle() {
            return Err(BridgeExecutionSafePointFailure::new(
                BridgeExecutionSafePointFailureKind::SignalRequestMismatch,
                "Signal safe-point evidence belongs to another request attempt",
            ));
        }
        let signal_state = match report.status() {
            ResourceInFlightStatus::Active => BridgeExecutionSafePointSignalState::Active,
            ResourceInFlightStatus::Fulfilled => BridgeExecutionSafePointSignalState::Fulfilled,
            ResourceInFlightStatus::Rejected => BridgeExecutionSafePointSignalState::Rejected,
            ResourceInFlightStatus::Superseded => BridgeExecutionSafePointSignalState::Superseded,
            ResourceInFlightStatus::Cancelled => BridgeExecutionSafePointSignalState::Cancelled,
            ResourceInFlightStatus::TimedOut => BridgeExecutionSafePointSignalState::TimedOut,
        };
        let pressure_state = project_pressure_state(report.pressure().class());
        let signal_counters = report.counters();
        Ok(BridgeExecutionSafePointObservation {
            basis_identity: self.identity.clone(),
            intent_identity: self.managed_intent.identity().clone(),
            step_contract_identity: managed.step_contract.identity_proof().clone(),
            observation_ordinal: report.ordinal().get(),
            lifecycle_ordinal: report.lifecycle_ordinal().get(),
            signal_state,
            pressure_state,
            queue_depth: report.pressure().queue_depth(),
            queue_capacity: report.pressure().queue_capacity(),
            timeout_wake_identity: report.timeout_wake_id().map(|wake| wake.get()),
            counters: BridgeExecutionSafePointCounters {
                exact_signal_request_lookup_count: signal_counters.exact_request_lookup_count(),
                pressure_classification_count: signal_counters.pressure_classification_count(),
            },
        })
    }
}

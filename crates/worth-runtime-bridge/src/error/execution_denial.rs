//! One exhaustive translation door from execution to Bridge's resource causes.
use worth_execution::{LeaseDenial, MapKernelStop, MemoryLimitDenial, WorkCeilingDenial};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeExecutionDenial {
    WorkerLimitExceedsParent,
    MemoryLimitExceedsParent,
    WorkLimitExceedsParent,
    MemoryExhausted(MemoryLimitDenial),
    ChargedBytesOverflow,
    UnrelatedNestedLease,
    NoActiveExecutionScope,
    EquivalenceContractUnavailable,
    Cancelled,
    DeadlineElapsed,
    WorkCounterOverflow,
    WorkCeiling,
    NestedStopped,
    Panicked,
}

impl From<LeaseDenial> for BridgeExecutionDenial {
    fn from(denial: LeaseDenial) -> Self {
        match denial {
            LeaseDenial::WorkerLimitExceedsParent => Self::WorkerLimitExceedsParent,
            LeaseDenial::MemoryLimitExceedsParent => Self::MemoryLimitExceedsParent,
            LeaseDenial::WorkLimitExceedsParent => Self::WorkLimitExceedsParent,
            LeaseDenial::MemoryExhausted(denial) => Self::MemoryExhausted(denial),
            LeaseDenial::ChargedBytesOverflow => Self::ChargedBytesOverflow,
            LeaseDenial::UnrelatedNestedLease => Self::UnrelatedNestedLease,
            LeaseDenial::NoActiveExecutionScope => Self::NoActiveExecutionScope,
            LeaseDenial::EquivalenceContractUnavailable => Self::EquivalenceContractUnavailable,
        }
    }
}

impl From<MapKernelStop> for BridgeExecutionDenial {
    fn from(stop: MapKernelStop) -> Self {
        match stop {
            MapKernelStop::Cancelled => Self::Cancelled,
            MapKernelStop::DeadlineElapsed => Self::DeadlineElapsed,
            MapKernelStop::WorkCounterOverflow => Self::WorkCounterOverflow,
            MapKernelStop::WorkCeiling => Self::WorkCeiling,
            MapKernelStop::NestedStopped => Self::NestedStopped,
        }
    }
}

impl From<WorkCeilingDenial> for BridgeExecutionDenial {
    fn from(denial: WorkCeilingDenial) -> Self {
        match denial {
            WorkCeilingDenial::Admission(denial) => denial.into(),
            WorkCeilingDenial::Stopped(stop) => stop.into(),
            WorkCeilingDenial::Panicked => Self::Panicked,
        }
    }
}

impl BridgeExecutionDenial {
    pub const fn label(self) -> &'static str {
        match self {
            Self::WorkerLimitExceedsParent => "worker-limit-exceeds-parent",
            Self::MemoryLimitExceedsParent => "memory-limit-exceeds-parent",
            Self::WorkLimitExceedsParent => "work-limit-exceeds-parent",
            Self::MemoryExhausted(_) => "memory-exhausted",
            Self::ChargedBytesOverflow => "charged-bytes-overflow",
            Self::UnrelatedNestedLease => "unrelated-nested-lease",
            Self::NoActiveExecutionScope => "no-active-execution-scope",
            Self::EquivalenceContractUnavailable => "equivalence-contract-unavailable",
            Self::Cancelled => "execution-cancelled",
            Self::DeadlineElapsed => "execution-deadline-elapsed",
            Self::WorkCounterOverflow => "work-counter-overflow",
            Self::WorkCeiling => "work-ceiling",
            Self::NestedStopped => "nested-stopped",
            Self::Panicked => "execution-panicked",
        }
    }
}

impl crate::snapshot::BridgeSnapshotReadError {
    /// Carry resource refusals through delivery adapters while retaining the
    /// adapter's existing domain-error classification.
    pub(crate) fn delivery_kind(
        &self,
        domain: super::BridgeDeliveryErrorKind,
    ) -> super::BridgeDeliveryErrorKind {
        use crate::snapshot::BridgeSnapshotReadErrorKind as Kind;
        match self.kind() {
            Kind::ExecutionDenied(denial) => {
                super::BridgeDeliveryErrorKind::ExecutionDenied(denial)
            }
            Kind::ExternalSnapshotReadFailure
            | Kind::SnapshotIdentityMismatch
            | Kind::DuplicateRecord
            | Kind::RecordCountMismatch
            | Kind::MissingRecord
            | Kind::ExtraRecord
            | Kind::ProjectionMaskRejected
            | Kind::AspectContractValidationDenied => domain,
        }
    }
}

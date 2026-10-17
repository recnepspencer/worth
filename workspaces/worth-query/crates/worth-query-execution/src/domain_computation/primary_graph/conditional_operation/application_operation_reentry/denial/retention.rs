//! Retaining read evidence may stop the request or fail physically.
use super::WorthQueryTemporalReentryDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryInvariantProjectionDenial;
use worth_execution::{
    ExecutionAllocationDenialClass as Class, LeaseDenial as Lease, MapKernelStop as Stop,
    MemoryLimitLevel as Level, WorkCeilingDenial as Request,
};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Native;
impl Denial {
    pub(super) fn retention_stop(denial: &WorthQueryInvariantProjectionDenial) -> Option<Self> {
        if let Some(stop) = denial.source_retention_interruption() {
            return Some(Self::from_native_stop(match stop { worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => Native::Cancelled, worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => Native::DeadlineElapsed }));
        }
        match denial.allocation_denial()?.classify() {
            Class::Physical(_) => None,
            Class::Request(request) => Some(Self::from_execution_request_stop(request)),
        }
    }
    pub(super) fn from_execution_request_stop(request: Request) -> Self {
        match request {
            Request::Panicked => Self::from_native_stop(Native::WorkerFailed),
            Request::Stopped(stop) => Self::from_native_stop(match stop {
                Stop::Cancelled => Native::Cancelled,
                Stop::DeadlineElapsed => Native::DeadlineElapsed,
                Stop::WorkCounterOverflow => Native::WorkCounterOverflow,
                Stop::WorkCeiling => Native::WorkExhausted,
                Stop::NestedStopped => Native::NestedStopped,
            }),
            Request::Admission(lease) => Self::from_native_stop(match lease {
                Lease::WorkerLimitExceedsParent => Native::WorkerLimitExceedsParent,
                Lease::MemoryLimitExceedsParent => Native::MemoryLimitExceedsParent,
                Lease::WorkLimitExceedsParent => Native::WorkLimitExceedsParent,
                Lease::ChargedBytesOverflow => Native::ChargedBytesOverflow,
                Lease::UnrelatedNestedLease => Native::UnrelatedNestedLease,
                Lease::NoActiveExecutionScope => Native::NoActiveExecutionScope,
                Lease::EquivalenceContractUnavailable => Native::EquivalenceContractUnavailable,
                Lease::MemoryExhausted(memory) => match memory.level {
                    Level::Policy { ancestor } => Native::PolicyMemoryExhausted {
                        requested: memory.requested,
                        admitted: memory.admitted,
                        ancestor,
                    },
                    Level::Process => Native::ProcessMemoryExhausted {
                        requested: memory.requested,
                        admitted: memory.admitted,
                    },
                    Level::Declared => Native::DeclaredMemoryExhausted {
                        requested: memory.requested,
                        admitted: memory.admitted,
                    },
                },
            }),
        }
    }
}

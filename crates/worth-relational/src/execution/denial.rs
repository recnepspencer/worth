//! The exhaustive door from execution failures to Relational causes.
use worth_execution::{
    LeaseDenial, MapKernelFailure, MapKernelStop, MapStop, MemoryLimitDenial, MemoryLimitLevel,
};
use worth_foundational::PartitionIdentity;

use super::read_only_packets::PacketAdmissionDenial;
use super::{PacketBudgetDenial, PacketExecutionStop, RelationalExecutionDenialCause as Cause};

pub(crate) fn lease_denial(denial: LeaseDenial) -> Cause {
    match denial {
        LeaseDenial::WorkerLimitExceedsParent => Cause::WorkerLimitExceedsParent,
        LeaseDenial::MemoryLimitExceedsParent => Cause::MemoryLimitExceedsParent,
        LeaseDenial::WorkLimitExceedsParent => Cause::WorkLimitExceedsParent,
        LeaseDenial::MemoryExhausted(MemoryLimitDenial {
            requested,
            admitted,
            level,
        }) => match level {
            MemoryLimitLevel::Policy { ancestor } => Cause::PolicyMemoryExhausted {
                requested,
                admitted,
                ancestor,
            },
            MemoryLimitLevel::Process => Cause::ProcessMemoryExhausted {
                requested,
                admitted,
            },
            MemoryLimitLevel::Declared => Cause::DeclaredMemoryExhausted {
                requested,
                admitted,
            },
        },
        LeaseDenial::ChargedBytesOverflow => Cause::ChargedBytesOverflow,
        LeaseDenial::UnrelatedNestedLease => Cause::UnrelatedNestedLease,
        LeaseDenial::NoActiveExecutionScope => Cause::NoActiveExecutionScope,
        LeaseDenial::EquivalenceContractUnavailable => Cause::EquivalenceContractUnavailable,
    }
}

/// A leased preparation reports request causes even when native staging
/// refuses before packet execution. Physical layout and allocator refusals
/// retain the allocation owner's conflict and checked quote.
pub(crate) fn allocation_commit_error(
    error: crate::transactions::data::TransactionCommitError,
) -> crate::transactions::data::TransactionCommitError {
    use crate::transactions::data::{CommitExecutionDenial, TransactionCommitError};
    use worth_execution::ExecutionAllocationDenialClass as Class;

    match error {
        TransactionCommitError::Conflict { error, commit_log } => {
            let denial: CommitExecutionDenial =
                match error.allocation_denial().map(|denial| denial.classify()) {
                    Some(Class::Request(stop)) => stop.into(),
                    Some(Class::Physical(_)) | None => {
                        return TransactionCommitError::Conflict { error, commit_log }
                    }
                };
            TransactionCommitError::Execution {
                denial,
                context: error.context,
                commit_log,
            }
        }
        error => error,
    }
}

pub(crate) fn kernel_stop(stop: MapKernelStop) -> Cause {
    match stop {
        MapKernelStop::Cancelled => Cause::Cancelled,
        MapKernelStop::DeadlineElapsed => Cause::DeadlineElapsed,
        MapKernelStop::WorkCounterOverflow => Cause::WorkCounterOverflow,
        MapKernelStop::WorkCeiling => Cause::WorkExhausted,
        MapKernelStop::NestedStopped => Cause::NestedStopped,
    }
}

impl From<MapKernelStop> for Cause {
    fn from(stop: MapKernelStop) -> Self {
        kernel_stop(stop)
    }
}

fn map_kernel_failure<E, R>(
    failure: &MapKernelFailure<E>,
    native: impl FnOnce(Cause) -> R,
    domain: impl FnOnce(&E) -> R,
) -> R {
    match failure {
        MapKernelFailure::Stop(stop) => native(kernel_stop(*stop)),
        MapKernelFailure::ResultCapacityExceeded => native(Cause::ResultCapacityExceeded),
        MapKernelFailure::Panic => native(Cause::WorkerFailed),
        MapKernelFailure::Domain(denial) => domain(denial),
    }
}

pub(crate) fn map_stop<E, R>(
    stop: &MapStop<E>,
    native: impl FnOnce(Cause) -> R,
    domain: impl FnOnce(&E) -> R,
) -> R {
    match stop {
        MapStop::Admission(denial) => native(lease_denial(*denial)),
        MapStop::WorkExhausted { .. } => native(Cause::WorkExhausted),
        MapStop::Failure { cause, .. } => map_kernel_failure(cause, native, domain),
    }
}

fn packet_budget_denial(denial: &PacketBudgetDenial) -> Cause {
    match denial {
        PacketBudgetDenial::ScratchCapacityExceeded => Cause::ScratchCapacityExceeded,
        PacketBudgetDenial::UncheckedCustomKernel => Cause::UncheckedCustomKernel,
    }
}

/// WorkCeiling and WorkExhausted share the authority's single work budget.
pub(crate) fn kernel_failure(failure: MapKernelFailure<PacketBudgetDenial>) -> Cause {
    map_kernel_failure(&failure, std::convert::identity, packet_budget_denial)
}

fn packet_admission(denial: PacketAdmissionDenial) -> Cause {
    match denial {
        PacketAdmissionDenial::ExpectedIdentitiesNotCanonical => {
            Cause::ExpectedIdentitiesNotCanonical
        }
        PacketAdmissionDenial::MemoryOverflow => Cause::MemoryOverflow,
    }
}

fn packet_denial(stop: PacketExecutionStop) -> (Cause, Option<u64>) {
    match stop {
        PacketExecutionStop::Admission(denial) => (packet_admission(denial), None),
        PacketExecutionStop::Execution { boundary, reason } => {
            let cause = map_stop(&reason, std::convert::identity, packet_budget_denial);
            (cause, boundary.map(PartitionIdentity::value))
        }
    }
}

impl From<PacketExecutionStop> for crate::transactions::data::CommitExecutionDenial {
    fn from(stop: PacketExecutionStop) -> Self {
        let (cause, partition_identity) = packet_denial(stop);
        Self {
            kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause),
            partition_identity,
        }
    }
}

impl From<PacketExecutionStop> for crate::indexes::data::DerivedIndexExecutionDenial {
    fn from(stop: PacketExecutionStop) -> Self {
        let (cause, partition_identity) = packet_denial(stop);
        Self {
            kind: crate::indexes::data::DerivedIndexExecutionDenialKind::Cause(cause),
            partition_identity,
        }
    }
}

#[cfg(test)]
mod keyless_admission;
#[cfg(test)]
mod leased_causes;
#[cfg(test)]
mod tests;

impl From<worth_execution::WorkCeilingDenial> for Cause {
    fn from(stop: worth_execution::WorkCeilingDenial) -> Self {
        match stop {
            worth_execution::WorkCeilingDenial::Admission(denial) => lease_denial(denial),
            worth_execution::WorkCeilingDenial::Stopped(stop) => kernel_stop(stop),
            worth_execution::WorkCeilingDenial::Panicked => Cause::WorkerFailed,
        }
    }
}

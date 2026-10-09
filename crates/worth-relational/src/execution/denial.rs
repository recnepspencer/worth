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
    use crate::transactions::data::{
        CommitExecutionDenial, CommitExecutionDenialKind, TransactionCommitError,
    };
    use worth_execution::ExecutionAllocationDenialKind as Kind;

    match error {
        TransactionCommitError::Conflict { error, commit_log } => {
            let cause = match error.allocation_denial().map(|denial| denial.kind()) {
                Some(Kind::Lease(denial)) => lease_denial(denial),
                Some(Kind::Cancelled) => Cause::Cancelled,
                Some(Kind::DeadlineElapsed) => Cause::DeadlineElapsed,
                _ => return TransactionCommitError::Conflict { error, commit_log },
            };
            TransactionCommitError::Execution {
                denial: CommitExecutionDenial {
                    kind: CommitExecutionDenialKind::Cause(cause),
                    partition_identity: None,
                },
                context: error.context,
                commit_log,
            }
        }
        error => error,
    }
}

/// WorkCeiling and WorkExhausted deliberately share WorkExhausted, mirroring
/// the authority's one work budget whether a checkpoint or map admission stops.
pub(crate) fn kernel_failure(failure: MapKernelFailure<PacketBudgetDenial>) -> Cause {
    match failure {
        MapKernelFailure::Stop(stop) => match stop {
            MapKernelStop::Cancelled => Cause::Cancelled,
            MapKernelStop::DeadlineElapsed => Cause::DeadlineElapsed,
            MapKernelStop::WorkCounterOverflow => Cause::WorkCounterOverflow,
            MapKernelStop::WorkCeiling => Cause::WorkExhausted,
            MapKernelStop::NestedStopped => Cause::NestedStopped,
        },
        MapKernelFailure::ResultCapacityExceeded => Cause::ResultCapacityExceeded,
        MapKernelFailure::Panic => Cause::WorkerFailed,
        MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded) => {
            Cause::ScratchCapacityExceeded
        }
        MapKernelFailure::Domain(PacketBudgetDenial::UncheckedCustomKernel) => {
            Cause::UncheckedCustomKernel
        }
    }
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
            let cause = match reason {
                MapStop::Admission(denial) => lease_denial(denial),
                MapStop::WorkExhausted { .. } => Cause::WorkExhausted,
                MapStop::Failure { cause, .. } => kernel_failure(cause),
            };
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

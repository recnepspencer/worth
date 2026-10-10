//! Exhaustive native causes for request-bearing query reads.
use super::{QueryReadExecutionStop, QueryReadPacketDenial};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};
use worth_execution::{
    MapDenial, MapKernelFailure, MapKernelStop, MapStop, ScanDenial, WorkCeilingDenial,
};

impl QueryReadExecutionStop {
    /// Semantic fragment absence has no execution cause. Resource and control
    /// failures retain their native cause and the selected partition.
    pub fn execution_denial(&self) -> Option<CommitExecutionDenial> {
        // None classifies canonicality or fragment failures as non-request stops;
        // it does not consume this stop. Query's denial handoffs own the original
        // stop separately, so classification cannot erase its native cause.
        let (cause, partition_identity) = match self {
            Self::Request(denial) => (
                Some(match denial {
                    WorkCeilingDenial::Admission(denial) => {
                        crate::execution::denial::lease_denial(*denial)
                    }
                    WorkCeilingDenial::Stopped(stop) => kernel_stop(*stop),
                    WorkCeilingDenial::Panicked => Cause::WorkerFailed,
                }),
                None,
            ),
            Self::PreparationAdmission(denial) | Self::CompletionAdmission(denial) => {
                (scan_denial(*denial), None)
            }
            Self::PreparationLease(denial) => {
                (Some(crate::execution::denial::lease_denial(*denial)), None)
            }
            Self::PreparationStopped { reason, .. } | Self::CompletionStopped { reason, .. } => {
                (stop_cause(reason, |()| None), None)
            }
            Self::PacketAdmission(denial) => (map_denial(*denial), None),
            Self::PacketStopped {
                boundary, reason, ..
            } => (
                stop_cause(reason, |denial| match denial {
                    QueryReadPacketDenial::FragmentUnavailable => None,
                }),
                boundary.map(worth_foundational::PartitionIdentity::value),
            ),
        };
        cause.map(|cause| CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity,
        })
    }
}
fn kernel_stop(stop: MapKernelStop) -> Cause {
    match stop {
        MapKernelStop::Cancelled => Cause::Cancelled,
        MapKernelStop::DeadlineElapsed => Cause::DeadlineElapsed,
        MapKernelStop::WorkCounterOverflow => Cause::WorkCounterOverflow,
        MapKernelStop::WorkCeiling => Cause::WorkExhausted,
        MapKernelStop::NestedStopped => Cause::NestedStopped,
    }
}
fn stop_cause<E>(reason: &MapStop<E>, domain: impl FnOnce(&E) -> Option<Cause>) -> Option<Cause> {
    match reason {
        MapStop::Admission(denial) => Some(crate::execution::denial::lease_denial(*denial)),
        MapStop::WorkExhausted { .. } => Some(Cause::WorkExhausted),
        MapStop::Failure { cause, .. } => match cause {
            MapKernelFailure::Stop(stop) => Some(kernel_stop(*stop)),
            MapKernelFailure::ResultCapacityExceeded => Some(Cause::ResultCapacityExceeded),
            MapKernelFailure::Panic => Some(Cause::WorkerFailed),
            MapKernelFailure::Domain(denial) => domain(denial),
        },
    }
}
fn scan_denial(denial: ScanDenial) -> Option<Cause> {
    match denial {
        ScanDenial::MemoryOverflow => Some(Cause::MemoryOverflow),
        ScanDenial::IdentitiesNotCanonical | ScanDenial::CoverageMismatch => None,
    }
}
fn map_denial(denial: MapDenial) -> Option<Cause> {
    match denial {
        MapDenial::MemoryOverflow => Some(Cause::MemoryOverflow),
        MapDenial::ExpectedIdentitiesNotCanonical
        | MapDenial::CoverageMismatch
        | MapDenial::ReadKeysNotCanonical { .. }
        | MapDenial::WriteKeysNotCanonical { .. }
        | MapDenial::WriteSetOverlap { .. }
        | MapDenial::ReadWriteConflict { .. } => None,
    }
}

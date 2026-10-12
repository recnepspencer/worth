//! Exhaustive native causes for request-bearing query reads.
use super::{QueryReadExecutionStop, QueryReadPacketDenial};
use crate::execution::denial::{kernel_stop, map_stop};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenial, CommitExecutionDenialKind};
use worth_execution::{MapDenial, ScanDenial, WorkCeilingDenial};

impl QueryReadExecutionStop {
    /// Semantic fragment absence has no execution cause. Caller input, resource
    /// and control failures retain their native cause and selected partition.
    pub fn execution_denial(&self) -> Option<CommitExecutionDenial> {
        // None classifies fragment absence and other shape failures as non-request stops;
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
                (map_stop(reason, Some, |()| None), None)
            }
            Self::PacketAdmission(denial) => (map_denial(*denial), None),
            Self::PacketStopped {
                boundary, reason, ..
            } => (
                map_stop(reason, Some, |denial| match denial {
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
fn scan_denial(denial: ScanDenial) -> Option<Cause> {
    match denial {
        ScanDenial::MemoryOverflow => Some(Cause::MemoryOverflow),
        ScanDenial::IdentitiesNotCanonical => Some(Cause::ExpectedIdentitiesNotCanonical),
        ScanDenial::CoverageMismatch => None,
    }
}
fn map_denial(denial: MapDenial) -> Option<Cause> {
    match denial {
        MapDenial::MemoryOverflow => Some(Cause::MemoryOverflow),
        MapDenial::ExpectedIdentitiesNotCanonical => Some(Cause::ExpectedIdentitiesNotCanonical),
        MapDenial::CoverageMismatch
        | MapDenial::ReadKeysNotCanonical { .. }
        | MapDenial::WriteKeysNotCanonical { .. }
        | MapDenial::WriteSetOverlap { .. }
        | MapDenial::ReadWriteConflict { .. } => None,
    }
}

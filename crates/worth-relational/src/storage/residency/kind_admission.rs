//! Original-request admission for publication's historical kind reconstruction.
use super::{
    kind_commitment, kind_projection, RegionContentFailure, RelationalRegionKindCommitment,
};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencyKindBasis;
mod retained_admission;
#[cfg(test)]
#[path = "kind_admission_tests.rs"]
mod tests;

mod maintained;
use super::kind_incremental::RegionKindIndexes;
use crate::storage::substrate::maintenance::{ChangeWork, CommitChange, MaintenanceRefusal};
use std::sync::Arc;

/// Reconstruct only at a cold trust boundary, under the original request.
pub(super) fn reconstruct(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition: &PartitionState,
) -> Result<Arc<RegionKindIndexes>, super::RelationalResidencyDemandFailure> {
    use super::RelationalResidencyDemandFailure as Failure;
    use crate::storage::substrate::maintenance::ReconstructionLane;
    ReconstructionLane::recover(execution, |_, work| {
        maintained::controlled(
            execution,
            &mut |units, bytes| work.prepare(units, bytes),
            &mut |checkpoint, _| RegionKindIndexes::capture_controlled(partition, checkpoint),
        )
    })
    .map(Arc::new)
    .map_err(|error| match error {
        maintained::KindMaintenanceError::Admission(MaintenanceRefusal::Execution(cause)) => {
            Failure::Request(cause)
        }
        other => Failure::StorageUnavailable(crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Region(partition.partition_id),
            cause: match other {
                maintained::KindMaintenanceError::Storage(cause) => cause,
                _ => Cause::SourceUnavailable,
            },
        }),
    })
}

/// Cost: journal-selected membership paths and newly appended metadata only.
pub(super) fn update(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition: &mut PartitionState,
    previous: &PartitionState,
    prior: &Arc<RegionKindIndexes>,
    journal: &crate::storage::overlay::PartitionMutationJournal,
    change: Option<&CommitChange>,
) -> Result<Arc<RegionKindIndexes>, RegionContentFailure> {
    let id = partition.partition_id;
    let mut owner = maintained::KindMaintainer::new(execution, partition, previous);
    let result = ChangeWork::run_phase(execution, "kind_commitment", |work| match change {
        Some(change) => crate::storage::substrate::maintenance::apply_change(
            &mut owner,
            Some(prior),
            change,
            work,
        ),
        // Unpublished candidate images have storage-owner journals but no
        // canonical commit artifact. They use the same private delta engine.
        None => owner.apply_journal(prior, journal, work),
    });
    result.map_err(|error| match error {
        maintained::KindMaintenanceError::Admission(MaintenanceRefusal::Execution(cause)) => {
            RegionContentFailure::Execution(cause)
        }
        maintained::KindMaintenanceError::Admission(_) => {
            RegionContentFailure::Storage(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(id),
                cause: Cause::SourceUnavailable,
            })
        }
        maintained::KindMaintenanceError::Storage(cause) => {
            RegionContentFailure::Storage(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(id),
                cause,
            })
        }
    })
}

/// One reconstruction engine over owned values. The driver supplies admission.
pub(super) fn normalize_value(
    partition: &mut PartitionState,
    checkpoint: &mut kind_projection::ProjectionCheckpoint<'_>,
    retain: &mut kind_projection::ProjectionRetention<'_>,
) -> Result<RelationalRegionKindCommitment, Cause> {
    // Admit bounded simultaneous traversal stacks before the first iterator.
    checkpoint(0, 65_536)?;
    let projection = kind_projection::canonical_history_controlled(partition, checkpoint)?;
    kind_projection::rebuild_controlled(
        partition,
        &projection,
        AdjacencyKindBasis::Historical,
        checkpoint,
        retain,
    )?;
    let head =
        kind_projection::capture_controlled(partition, AdjacencyKindBasis::Current, checkpoint)?;
    let history =
        kind_projection::capture_controlled(partition, AdjacencyKindBasis::Historical, checkpoint)?;
    Ok(RelationalRegionKindCommitment {
        head: kind_commitment::digest_controlled(&head, checkpoint)?,
        history: kind_commitment::digest_controlled(&history, checkpoint)?,
    })
}

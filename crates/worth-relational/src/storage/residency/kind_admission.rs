//! Original-request admission for publication's historical kind reconstruction.
use super::{
    kind_commitment, kind_projection, RegionContentFailure, RelationalRegionKindCommitment,
};
use crate::execution::request_stage;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencyKindBasis;
mod retained_admission;
#[cfg(test)]
#[path = "kind_admission_tests.rs"]
mod tests;

pub(super) fn normalize(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    partition: &mut PartitionState,
) -> Result<RelationalRegionKindCommitment, RegionContentFailure> {
    let first_stop = std::cell::Cell::new(None);
    let result = request_stage::optional_stage(
        execution,
        |context, memory| {
            let mut checkpoint = |work, bytes| {
                if first_stop.get().is_some() {
                    return Err(Cause::SourceUnavailable);
                }
                let admission = request_stage::checkpoint(context, work)
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)
                    .and_then(|()| memory.claim(bytes));
                if let Err(cause) = admission {
                    first_stop.set(Some(cause));
                    return Err(Cause::SourceUnavailable);
                }
                Ok(())
            };
            let mut retain = |prior: Option<&crate::storage::partition::HistoricalKindCustody>,
                              bytes: u64| {
                if first_stop.get().is_some() {
                    return Err(Cause::SourceUnavailable);
                }
                match retained_admission::admit(prior, bytes, &mut |bytes| {
                    execution
                        .reserve_retained_memory(bytes)
                        .map_err(crate::execution::RelationalExecutionDenialCause::from)
                }) {
                    Ok(custody) => Ok(custody),
                    Err(cause) => {
                        first_stop.set(Some(cause));
                        Err(Cause::SourceUnavailable)
                    }
                }
            };
            normalize_value(partition, &mut checkpoint, &mut retain)
        },
        |_| 0,
    );
    // The internal source sentinel is never a public request refusal. Concrete
    // original stops win; only its nested sentinel may yield to the first cause.
    let result = preserve_original_stop(result, first_stop.get())
        .map_err(RegionContentFailure::Execution)?;
    let commitment = result.map_err(|cause| {
        RegionContentFailure::Storage(crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Region(partition.partition_id),
            cause,
        })
    })?;
    partition.clear_runtime_pin_counters();
    Ok(commitment)
}

/// The opaque SDK sentinel cannot replace the first concrete owner refusal.
fn preserve_original_stop<T>(
    result: Result<T, crate::execution::RelationalExecutionDenialCause>,
    first: Option<crate::execution::RelationalExecutionDenialCause>,
) -> Result<T, crate::execution::RelationalExecutionDenialCause> {
    use crate::execution::RelationalExecutionDenialCause as Stop;
    match result {
        Err(Stop::NestedStopped) => Err(first.unwrap_or(Stop::NestedStopped)),
        Err(cause) => Err(cause),
        Ok(value) => match first {
            Some(cause) => Err(cause),
            None => Ok(value),
        },
    }
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

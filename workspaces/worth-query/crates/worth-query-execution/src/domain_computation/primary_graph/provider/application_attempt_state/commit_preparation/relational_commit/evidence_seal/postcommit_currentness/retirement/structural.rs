//! Whole owned structural reads can end in this commit's actual lifecycle truth.
use super::{committed_retirement, Fact};
use crate::domain_computation::primary_graph::structural_walk::StructuralWalkObservation;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::EntityId, mvcc::CompanionPreflightStop, runtime::RelationalRuntime,
    snapshots::SnapshotHandle, transactions::RelationalExecutionDenialCause as Cause,
};

pub(in super::super) fn prepare(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    fact: &Fact,
    retired: &BTreeSet<EntityId>,
) -> Result<Option<Fact>, CompanionPreflightStop> {
    let (anchor, source) = match fact {
        Fact::Aggregate { observation } => (observation.key().member(), observation.source()),
        Fact::StructuralWalk { observation } => {
            let Some(anchor) = observation.key().roots().first() else {
                return Ok(None);
            };
            (*anchor, observation.source())
        }
        _ => return Ok(None),
    };
    if source.runtime_instance_id() != snapshot.runtime_instance_id()
        || source.branch_id() != snapshot.branch_id()
        || !retired.contains(&anchor)
    {
        return Ok(None);
    }
    let (result, _) = execution
        .run_native_with_checkpoint(|context| {
            StructuralWalkObservation::checkpoint(context)?;
            let Some(view) = runtime.read_truth().project_snapshot(snapshot) else {
                return Ok(None);
            };
            let mut proves = |entity| {
                StructuralWalkObservation::checkpoint(context)?;
                Ok(retired.contains(&entity)
                    && view
                        .entity_retirement(entity)
                        .is_some_and(|record| record.deleted_at_version() == view.version_id()))
            };
            let covered = match fact {
                Fact::Aggregate { observation } => all_retired(
                    std::iter::once(observation.key().member())
                        .chain(observation.native().members()),
                    &mut proves,
                )?,
                Fact::StructuralWalk { observation } => all_retired(
                    observation
                        .key()
                        .roots()
                        .iter()
                        .copied()
                        .chain(
                            observation
                                .native()
                                .visits()
                                .iter()
                                .map(|visit| visit.entity.entity_id),
                        )
                        .chain(
                            observation
                                .native()
                                .relations()
                                .iter()
                                .flat_map(|relation| [relation.source, relation.target]),
                        ),
                    &mut proves,
                )?,
                _ => unreachable!("structural fact selected above"),
            };
            StructuralWalkObservation::checkpoint(context)?;
            Ok::<_, Cause>(if covered {
                committed_retirement(runtime, snapshot, anchor, fact)
            } else {
                None
            })
        })
        .map_err(|cause| CompanionPreflightStop::Execution(cause.into()))?;
    result.map_err(CompanionPreflightStop::Execution)
}

fn all_retired(
    entities: impl Iterator<Item = EntityId>,
    proves: &mut impl FnMut(EntityId) -> Result<bool, Cause>,
) -> Result<bool, Cause> {
    for entity in entities {
        if !proves(entity)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests;

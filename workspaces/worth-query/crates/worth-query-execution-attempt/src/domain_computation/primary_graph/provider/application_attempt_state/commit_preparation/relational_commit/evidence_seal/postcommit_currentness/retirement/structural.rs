//! Whole owned structural reads can end in this commit's actual lifecycle truth.
use super::super::RebaseVerificationFailure as Failure;
use super::{committed_retirement, Fact};
use crate::domain_computation::primary_graph::structural_walk::StructuralWalkObservation;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::EntityId,
    mvcc::{CompanionPreflightStop, RelationalRequiredOutputPublication},
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause as Cause,
};

/// A complete predecessor-owned footprint may end in real candidate retirements.
/// Partial and foreign footprints keep their ordinary structural comparison.
pub(in super::super) fn support_equal_after_own_effect(
    execution: &ExecutionRequest<'_, '_>,
    after_own: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    complete: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    fact: &Fact,
    predecessor: &RelationalRequiredOutputPublication,
) -> Result<Option<bool>, Failure> {
    match fact {
        Fact::Aggregate { .. } => {}
        Fact::StructuralWalk { observation } if !observation.key().roots().is_empty() => {}
        _ => return Ok(None),
    }
    let left = after_own.project_candidate();
    let right = complete.project_candidate();
    let (answer, _) = execution
        .run_native_with_checkpoint(|context| {
            for entity in entities(fact) {
                StructuralWalkObservation::checkpoint(context).map_err(Failure::execution)?;
                if !predecessor
                    .contains_output_entity(execution, entity)
                    .map_err(Failure::execution)?
                    || left
                        .read_entity_retirement(execution, entity)
                        .map_err(Failure::execution)?
                        .is_none()
                {
                    return Ok(None);
                }
            }
            for entity in entities(fact) {
                StructuralWalkObservation::checkpoint(context).map_err(Failure::execution)?;
                let a = left
                    .read_entity_retirement(execution, entity)
                    .map_err(Failure::execution)?
                    .ok_or_else(Failure::native_unavailable)?;
                let Some(b) = right
                    .read_entity_retirement(execution, entity)
                    .map_err(Failure::execution)?
                else {
                    return Ok(Some(false));
                };
                if a.kind_id() != b.kind_id()
                    || a.created_at_version() != b.created_at_version()
                    || a.deleted_at_version() != b.deleted_at_version()
                {
                    return Ok(Some(false));
                }
            }
            Ok(Some(true))
        })
        .map_err(|cause| Failure::execution(cause.into()))?;
    answer
}

pub fn prepare(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    snapshot: &SnapshotHandle,
    fact: &Fact,
    retired: &BTreeSet<EntityId>,
    predecessor: Option<&RelationalRequiredOutputPublication>,
) -> Result<Option<Fact>, CompanionPreflightStop> {
    let (anchor, source) = match fact {
        Fact::Aggregate { observation } => (observation.key().member(), observation.source()),
        Fact::StructuralWalk { observation } => {
            let Some(anchor) = observation.key().roots().first() else {
                return Ok(None);
            };
            (*anchor, observation.source())
        }
        // This Source-only observation cannot be rewritten into a synthetic
        // retirement witness. Preserve its original source and currentness.
        Fact::DerivedSource { .. } => return Ok(None),
        _ => return Ok(None),
    };
    if source.runtime_instance_id() != snapshot.runtime_instance_id() || !retired.contains(&anchor)
    {
        return Ok(None);
    }
    let same_branch = source
        .committed_snapshot()
        .is_some_and(|source| source.branch_id() == snapshot.branch_id());
    if !same_branch && predecessor.is_none() {
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
                if !same_branch
                    && !predecessor
                        .expect("checked predecessor")
                        .contains_output_entity(execution, entity)?
                {
                    return Ok(false);
                }
                Ok(retired.contains(&entity)
                    && view
                        .read_entity_retirement(execution, entity)?
                        .is_some_and(|record| record.deleted_at_version() == view.version_id()))
            };
            let covered = all_retired(entities(fact), &mut proves)?;
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

fn entities(fact: &Fact) -> impl Iterator<Item = EntityId> + '_ {
    let aggregate = match fact {
        Fact::Aggregate { observation } => Some(observation),
        _ => None,
    };
    let walk = match fact {
        Fact::StructuralWalk { observation } => Some(observation),
        _ => None,
    };
    aggregate
        .into_iter()
        .flat_map(|observation| {
            std::iter::once(observation.key().member()).chain(observation.native().members())
        })
        .chain(walk.into_iter().flat_map(|observation| {
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
                )
        }))
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

//! Judge a selected Source against the exact native effects and the applied image.
use super::super::parent_read_footprint::ManagedIndexParentReadFootprint;
use super::reach::Reach;
use super::relevance::SourceRelevance;
use crate::authority::mutation::MutationEffect;
use crate::execution::request_stage::checkpoint;
use crate::identity::data::{EntityId, KindId};
use crate::schema::data::RelationalSchemaRegistry;
use crate::storage::overlay::{CurrentPartitionAccess, WorkingState};
use crate::transactions::data::primitives::RecordRef;
use crate::validation::candidate_source::{
    ManagedIndexSourcePreparationDenial as Denial, ManagedIndexSourceProgram,
};
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, FieldKey};

#[allow(clippy::too_many_arguments)]
pub(super) fn source_is_unchanged(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
    footprint: &ManagedIndexParentReadFootprint,
    registry: &RelationalSchemaRegistry,
    state: &WorkingState,
    effect: &MutationEffect,
    only: Option<&[RecordRef]>,
    reach: Option<Reach>,
) -> Result<bool, Denial> {
    let key_of = |entity: EntityId| scope_key_value(state, footprint.scope_key.as_ref()?, entity);
    let relevance =
        SourceRelevance::of(registry, program, scope, footprint, effect, &key_of).reached_by(reach);
    let (unchanged, _) = execution
        .run_native_with_checkpoint(|context| {
            let only = match only {
                Some(records) => {
                    checkpoint(context, records.len() as u64 + 1)?;
                    Some(records.iter().collect::<BTreeSet<_>>())
                }
                None => None,
            };
            let judged =
                |record: &RecordRef| only.as_ref().is_none_or(|only| only.contains(record));
            for delta in &effect.adjacency.deltas {
                checkpoint(context, 1)?;
                if judged(&RecordRef::Relation(delta.relation_id)) && relevance.adjacency(delta) {
                    return Ok(false);
                }
            }
            let mut described = BTreeSet::new();
            for delta in &effect.publication.canonical_deltas {
                checkpoint(context, 1)?;
                if judged(&delta.target) && relevance.delta(delta) {
                    return Ok(false);
                }
                described.insert(&delta.target);
            }
            for record in &effect.publication.changed_records {
                checkpoint(context, 1)?;
                if described.contains(record) || !judged(record) {
                    continue;
                }
                let kind = match record {
                    RecordRef::Entity(id) => state
                        .get_current_partition(id.partition_id)
                        .and_then(|partition| partition.entity(*id))
                        .and_then(|slot| slot.kind_id()),
                    RecordRef::Relation(id) => state
                        .get_current_partition(id.partition_id)
                        .and_then(|partition| partition.relation(*id))
                        .and_then(|slot| slot.kind_id()),
                };
                if relevance.undescribed(record, kind) {
                    return Ok(false);
                }
            }
            Ok(true)
        })
        .map_err(|cause| {
            crate::derived_relation::DerivedRelationReadDenial::Request(cause.into())
        })?;
    unchanged.map_err(|cause: crate::execution::RelationalExecutionDenialCause| {
        Denial::from(crate::derived_relation::DerivedRelationReadDenial::Request(
            cause,
        ))
    })
}

/// One entity's scope-key value in the candidate's working state, when it is
/// an entity of the key's kind with that field present.
fn scope_key_value(
    state: &WorkingState,
    (kind, aspect, field): &(KindId, AspectKey, FieldKey),
    entity: EntityId,
) -> Option<worth_foundational::facade::AspectValue> {
    use worth_foundational::facade::ContractValidatedAspectValueView as View;
    let partition = state.get_current_partition(entity.partition_id)?;
    let slot = partition.entity(entity)?;
    if slot.kind_id()? != *kind {
        return None;
    }
    match slot
        .extra()
        .authoritative_aspect_state()?
        .get(aspect)?
        .view()
    {
        View::Struct(value) => value.get(field).cloned(),
        View::Scalar(value) => Some(value.clone()),
    }
}

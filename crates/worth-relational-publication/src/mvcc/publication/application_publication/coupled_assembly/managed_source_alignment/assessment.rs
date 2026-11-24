//! Judge a selected Source against the exact native effects and the applied image.
use super::super::parent_read_footprint::ManagedIndexParentReadFootprint;
use super::relevance::SourceRelevance;
use contracts::execution::request_stage::checkpoint;
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::record_ids::EntityId;
use contracts::schema::data::RelationalSchemaRegistry;
use contracts::transactions::data::primitives::RecordRef;
use contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial as Denial;
use contracts::validation::candidate_source::program::ManagedIndexSourceProgram;
use std::collections::BTreeSet;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port::authority::mutation::effect::MutationEffect;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub fn source_is_unchanged(
    execution: &ExecutionRequest<'_, '_>,
    program: &ManagedIndexSourceProgram,
    scope: EntityId,
    footprint: &ManagedIndexParentReadFootprint,
    registry: &RelationalSchemaRegistry,
    state: &WorkingState,
    effect: &MutationEffect,
) -> Result<bool, Denial> {
    let key_of = |entity: EntityId| scope_key_value(state, footprint.scope_key.as_ref()?, entity);
    let relevance = SourceRelevance::of(registry, program, scope, footprint, effect, &key_of);
    let (unchanged, _) = execution
        .run_native_with_checkpoint(|context| {
            for delta in &effect.adjacency.deltas {
                checkpoint(context, 1)?;
                if relevance.adjacency(delta) {
                    return Ok(false);
                }
            }
            let mut described = BTreeSet::new();
            for delta in &effect.publication.canonical_deltas {
                checkpoint(context, 1)?;
                if relevance.delta(delta) {
                    return Ok(false);
                }
                described.insert(&delta.target);
            }
            for record in &effect.publication.changed_records {
                checkpoint(context, 1)?;
                if described.contains(record) {
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
            contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request(
                cause.into(),
            )
        })?;
    unchanged.map_err(
        |cause: contracts::execution::denial_cause::RelationalExecutionDenialCause| {
            Denial::from(
                contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request(cause),
            )
        },
    )
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

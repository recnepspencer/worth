//! Ordinary graph authors cannot write the kinds bound to Native aggregate ownership.
use crate::identity::data::KindId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::{
    CreateIntent, EntityMutationIntent as Entity, MutationIntent,
    RelationMutationIntent as Relation, TransactionCommitError,
};
use crate::validation::engine::state_view::InvariantStateView;

pub(crate) fn ordinary_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::execution::RequestWorkBudget>,
    registry: &RelationalSchemaRegistry,
    before: &InvariantStateView<'_>,
    intents: &[MutationIntent],
    mode: Option<crate::mvcc::RelationalMaterializationTransactionMode>,
) -> Result<(), TransactionCommitError> {
    if !registry
        .relation_kinds
        .values()
        .any(|kind| kind.relation_integrity.aggregate_identity.is_some())
    {
        return Ok(());
    }
    super::funding::run(execution, budget, |work| {
    for contract in registry
        .relation_kinds
        .values()
        .filter_map(|kind| kind.relation_integrity.aggregate_identity.as_ref())
    {
        work.checkpoint(1)?;
        let private = |kind: KindId| {
            [
                contract.storage.aggregate_kind,
                contract.storage.lineage_kind,
                contract.membership_kind,
            ]
            .contains(&kind)
        };
        for intent in intents {
            work.checkpoint(1)?;
            let (declared, record) = match intent {
                MutationIntent::Create(create) => (
                    Some(match create {
                        CreateIntent::Entity(value) => value.kind_id,
                        CreateIntent::EntityAspects(value) => value.kind_id,
                        CreateIntent::BulkEntities(value) => value.kind_id,
                        CreateIntent::Relation(value) => value.kind_id,
                        CreateIntent::RelationAspects(value) => value.kind_id,
                        CreateIntent::BulkRelations(value) => value.kind_id,
                    }),
                    None,
                ),
                MutationIntent::Entity(value) => {
                    let (id, replacement) = match value {
                        Entity::UpdateFields(value) => (value.entity_id, None),
                        Entity::ApplyAspectPatch(value) => (value.entity_id, None),
                        Entity::Delete(value) => (value.entity_id, None),
                        Entity::Replace(value) => {
                            (value.entity_id, Some(value.replacement.kind_id))
                        }
                        Entity::Revalidate(_) => continue,
                    };
                    (
                        replacement,
                        before.entity_metadata(id).map(|entry| entry.kind_id),
                    )
                }
                MutationIntent::Relation(value) => {
                    let (id, replacement) = match value {
                        Relation::UpdateEndpoints(value) => {
                            (value.relation_id, Some(value.kind_id))
                        }
                        Relation::ApplyAspectPatch(value) => (value.relation_id, None),
                        Relation::Delete(value) => (value.relation_id, None),
                        Relation::Revalidate(_) => continue,
                    };
                    (
                        replacement,
                        before.relation_metadata(id).map(|entry| entry.kind_id),
                    )
                }
                MutationIntent::Materialization(value) => {
                    let physical_owner = matches!(mode, Some(crate::mvcc::RelationalMaterializationTransactionMode::Suspend)) && value.is_suspension()
                        || matches!(mode, Some(crate::mvcc::RelationalMaterializationTransactionMode::Rematerialize)) && value.is_rematerialization();
                    if physical_owner { continue; }
                    let kind = match value.record() {
                        crate::transactions::data::RecordRef::Entity(id) => before.entity_metadata(id).map(|entry| entry.kind_id),
                        crate::transactions::data::RecordRef::Relation(id) => before.relation_metadata(id).map(|entry| entry.kind_id),
                    };
                    (None, kind)
                }
            };
            if declared.into_iter().chain(record).any(private) {
                return work.invalid(super::preparation::PlanDenial::OrdinaryMetadataWrite);
            }
        }
    }
    Ok(GuardAdmission)
    }).map(|_| ())
}
struct GuardAdmission;
impl worth_execution::ChargedBytes for GuardAdmission {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

//! Ordinary graph authors cannot write Native-owned metadata kinds.
use crate::contracts::identity::data::primitives::KindId;
use crate::contracts::schema::data::current_publication_storage::CurrentPublicationStorageOwnerPort as _;
use crate::contracts::schema::data::relation_integrity::aggregate::AggregateIdentityContractDeclarationOwnerPort as _;
use crate::contracts::schema::data::{
    RelationalSchemaRegistry, RelationalSchemaRegistryFieldsPort as _,
};
use crate::contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent as Entity, MutationIntent,
    RelationMutationIntent as Relation,
};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::validation::engine::state_view::InvariantStateView;

/// Selection is not a metadata-owner grant, even when its old payload is genuine.
pub fn retained_entity_selection(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::contracts::execution::request_work::RequestWorkBudget>,
    registry: &RelationalSchemaRegistry,
    kind: KindId,
) -> Result<(), TransactionCommitError> {
    super::funding::run(execution, budget, |work| {
        if (*registry.owner_current_publication_storage())
            .as_ref()
            .is_some_and(|storage| storage.metadata_kinds().contains(&kind))
        {
            work.checkpoint(1)?;
            return work.invalid(super::preparation::PlanDenial::CurrentMetadataWrite {
                kind,
                record: None,
            });
        }
        for contract in (*registry.owner_relation_kinds())
            .values()
            .filter_map(|entry| entry.relation_integrity.aggregate_identity.as_ref())
        {
            work.checkpoint(1)?;
            if contract.metadata_kinds().contains(&kind) {
                return work.invalid(super::preparation::PlanDenial::OrdinaryMetadataWrite);
            }
        }
        Ok(GuardAdmission)
    })
    .map(|_| ())
}

pub fn ordinary_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::contracts::execution::request_work::RequestWorkBudget>,
    registry: &RelationalSchemaRegistry,
    before: &InvariantStateView<'_>,
    intents: &[MutationIntent],
    mode: Option<crate::contracts::mvcc::transaction::intent::materialization_mode::RelationalMaterializationTransactionMode>,
) -> Result<(), TransactionCommitError> {
    if (*registry.owner_current_publication_storage()).is_none()
        && !(*registry.owner_relation_kinds())
            .values()
            .any(|kind| kind.relation_integrity.aggregate_identity.is_some())
    {
        return Ok(());
    }
    super::funding::run(execution, budget, |work| {
    for metadata in (*registry
        .owner_relation_kinds())
        .values()
        .filter_map(|kind| kind.relation_integrity.aggregate_identity.as_ref())
        .map(|contract| PrivateKinds::Aggregate(contract.metadata_kinds()))
        .chain((*registry.owner_current_publication_storage()).as_ref()
            .map(|storage| PrivateKinds::Publication(storage.metadata_kinds())))
    {
        work.checkpoint(1)?;
        let private = |kind: KindId| metadata.contains(kind);
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
                    let physical_owner = matches!(mode, Some(crate::contracts::mvcc::transaction::intent::materialization_mode::RelationalMaterializationTransactionMode::Suspend)) && value.is_suspension()
                        || matches!(mode, Some(crate::contracts::mvcc::transaction::intent::materialization_mode::RelationalMaterializationTransactionMode::Rematerialize)) && value.is_rematerialization();
                    if physical_owner { continue; }
                    let kind = match value.record() {
                        crate::contracts::transactions::data::primitives::RecordRef::Entity(id) => before.entity_metadata(id).map(|entry| entry.kind_id),
                        crate::contracts::transactions::data::primitives::RecordRef::Relation(id) => before.relation_metadata(id).map(|entry| entry.kind_id),
                    };
                    (None, kind)
                }
            };
            if declared.into_iter().chain(record).any(private) {
                if (*registry.owner_current_publication_storage()).is_some() {
                    let target=match intent {
                        MutationIntent::Entity(Entity::UpdateFields(v))=>Some(crate::contracts::transactions::data::primitives::RecordRef::Entity(v.entity_id)),
                        MutationIntent::Entity(Entity::ApplyAspectPatch(v))=>Some(crate::contracts::transactions::data::primitives::RecordRef::Entity(v.entity_id)),
                        MutationIntent::Entity(Entity::Delete(v))=>Some(crate::contracts::transactions::data::primitives::RecordRef::Entity(v.entity_id)),
                        MutationIntent::Entity(Entity::Replace(v))=>Some(crate::contracts::transactions::data::primitives::RecordRef::Entity(v.entity_id)),
                        MutationIntent::Relation(Relation::UpdateEndpoints(v))=>Some(crate::contracts::transactions::data::primitives::RecordRef::Relation(v.relation_id)),
                        MutationIntent::Relation(Relation::ApplyAspectPatch(v))=>Some(crate::contracts::transactions::data::primitives::RecordRef::Relation(v.relation_id)),
                        MutationIntent::Relation(Relation::Delete(v))=>Some(crate::contracts::transactions::data::primitives::RecordRef::Relation(v.relation_id)),
                        _=>None,
                    };
                    return work.invalid(super::preparation::PlanDenial::CurrentMetadataWrite{kind:declared.or(record).expect("a private declared or retained kind matched"),record:target});
                }
                return work.invalid(super::preparation::PlanDenial::OrdinaryMetadataWrite);
            }
        }
    }
    Ok(GuardAdmission)
    }).map(|_| ())
}
enum PrivateKinds {
    Aggregate([KindId; 3]),
    Publication([KindId; 10]),
}
impl PrivateKinds {
    fn contains(&self, kind: KindId) -> bool {
        match self {
            Self::Aggregate(kinds) => kinds.contains(&kind),
            Self::Publication(kinds) => kinds.contains(&kind),
        }
    }
}
struct GuardAdmission;
impl worth_execution::ChargedBytes for GuardAdmission {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

/// A private relation create is refused before ordinary duplicate-pair admission.
/// Otherwise an existing metadata link could disguise a single-writer violation
/// as an unrelated business-relation identity conflict.
pub fn current_relation_creations(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    budget: Option<&crate::contracts::execution::request_work::RequestWorkBudget>,
    registry: &RelationalSchemaRegistry,
    intents: &[MutationIntent],
) -> Result<(), TransactionCommitError> {
    let Some(storage) = registry.owner_current_publication_storage() else {
        return Ok(());
    };
    super::funding::run(execution, budget, |work| {
        for intent in intents {
            work.checkpoint(1)?;
            let kind = match intent {
                MutationIntent::Create(CreateIntent::Relation(v)) => v.kind_id,
                MutationIntent::Create(CreateIntent::RelationAspects(v)) => v.kind_id,
                MutationIntent::Create(CreateIntent::BulkRelations(v)) => v.kind_id,
                _ => continue,
            };
            if storage.metadata_kinds().contains(&kind) {
                return work.invalid(super::preparation::PlanDenial::CurrentMetadataWrite {
                    kind,
                    record: None,
                });
            }
        }
        Ok(GuardAdmission)
    })
    .map(|_| ())
}

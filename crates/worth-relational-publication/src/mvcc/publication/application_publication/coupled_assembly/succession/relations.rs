//! A Birth chain's head relations (§1.5 Identity): a successor round
//! re-creates each under the same (rescoped) key and partition, with the same
//! kind and actual endpoints, and the fold keeps the head's relation, so the
//! chain still creates each relation once and its batches name it as the
//! head's correspondence does.
use super::super::refusals::{overflow, postponed};
use contracts::identity::data::primitives::KindId;
use contracts::identity::data::record_ids::EntityId;
use contracts::publication::data::authored_retirement_refusal::CoupledProvenancePostponement as Reason;
use contracts::transactions::data::intents::mutation_intent::CreateIntent;
use contracts::transactions::data::intents::mutation_intent::MutationIntent;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::EntityReference;
use worth_relational_contracts::internal_port as contracts;

/// One relation creation: its key text, partition, kind and actual endpoints.
pub(super) type Relink = (
    String,
    contracts::identity::data::primitives::PartitionId,
    KindId,
    EntityId,
    EntityId,
);

/// The relation creation `intent` names, over the chain's allocations; `None`
/// for any other intent. A relation with fields, a relation-aspect create or a
/// bulk relation create is a structural effect the fold does not lower.
pub(super) fn relink(
    bindings: &contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    symbols: &contracts::symbols::data::string_interner::StringInterner,
    intent: &MutationIntent,
    source: EntityId,
    charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
) -> Result<Option<Relink>, TransactionCommitError> {
    let MutationIntent::Create(create) = intent else {
        return Ok(None);
    };
    let spec = match create {
        CreateIntent::Relation(spec) if spec.fields.is_empty() => spec,
        CreateIntent::Relation(_)
        | CreateIntent::RelationAspects(_)
        | CreateIntent::BulkRelations(_) => {
            return Err(postponed(Reason::StructuralEffect, source))
        }
        CreateIntent::Entity(_)
        | CreateIntent::EntityAspects(_)
        | CreateIntent::BulkEntities(_) => return Ok(None),
    };
    let key = spec
        .client_key
        .resolve_with_interner(symbols)
        .ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "coupled local key has no actual Native symbol meaning",
            )
        })?;
    charge(key.len() as u64 + 1)?;
    Ok(Some((
        key.to_owned(),
        spec.partition_id,
        spec.kind_id,
        endpoint(bindings, &spec.source, source, charge)?,
        endpoint(bindings, &spec.target, source, charge)?,
    )))
}

fn endpoint(
    bindings: &contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    reference: &EntityReference,
    source: EntityId,
    charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
) -> Result<EntityId, TransactionCommitError> {
    match reference {
        EntityReference::Existing(id) => Ok(*id),
        EntityReference::Created(created) => {
            charge(bindings.resolve_work_bound(created).ok_or_else(overflow)?)?;
            bindings
                .resolve(created)
                .ok_or_else(|| postponed(Reason::StructuralEffect, source))
        }
    }
}

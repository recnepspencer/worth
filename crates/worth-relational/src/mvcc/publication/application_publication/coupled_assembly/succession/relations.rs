//! A Birth chain's head relations (§1.5 Identity): a successor round
//! re-creates each with the same kind and actual endpoints, and the fold keeps
//! the head's relation, so the chain still creates each relation once.
use super::*;
use crate::identity::data::KindId;
use crate::transactions::data::EntityReference;

/// One relation creation: its kind and actual endpoints.
pub(super) type Relink = (KindId, EntityId, EntityId);

/// The relation creation `intent` names, over the chain's allocations; `None`
/// for any other intent. A relation with fields, a relation-aspect create or a
/// bulk relation create is a structural effect the fold does not lower.
pub(super) fn relink(
    bindings: &crate::transactions::data::CommitCreatedEntityBindings,
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
            return Err(continuation::postponed(Reason::StructuralEffect, source))
        }
        CreateIntent::Entity(_)
        | CreateIntent::EntityAspects(_)
        | CreateIntent::BulkEntities(_) => return Ok(None),
    };
    Ok(Some((
        spec.kind_id,
        endpoint(bindings, &spec.source, source, charge)?,
        endpoint(bindings, &spec.target, source, charge)?,
    )))
}

fn endpoint(
    bindings: &crate::transactions::data::CommitCreatedEntityBindings,
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
                .ok_or_else(|| continuation::postponed(Reason::StructuralEffect, source))
        }
    }
}

//! A chain's relation identity (§1.5 Identity). A successor round re-creates
//! each head relation under the same (rescoped) key and partition, with the
//! same kind and actual endpoints, and deletes again each relation its latest
//! round deleted (#277). The fold keeps the head's relation and the latest
//! round's delete, so the chain still creates and deletes each relation once
//! and its batches name them as the head's correspondence does.
use super::super::refusals::{overflow, postponed};
use crate::identity::data::EntityId;
use crate::identity::data::KindId;
use crate::publication::data::CoupledProvenancePostponement as Reason;
use crate::transactions::data::intents::mutation_intent::{CreateIntent, MutationIntent};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::EntityReference;
use std::collections::BTreeSet;

/// One relation creation: its key text, partition, kind and actual endpoints.
pub(super) type Relink = (
    String,
    crate::identity::data::PartitionId,
    KindId,
    EntityId,
    EntityId,
);

/// The relation creation `intent` names, over the chain's allocations; `None`
/// for any other intent. A relation with fields, a relation-aspect create or a
/// bulk relation create is a structural effect the fold does not lower.
fn relink(
    bindings: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    symbols: &crate::symbols::data::StringInterner,
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
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
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
    bindings: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
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

/// The relation a delete intent removes; `None` for any other intent.
fn unrelink(intent: &MutationIntent) -> Option<crate::identity::data::RelationId> {
    match intent {
        MutationIntent::Relation(crate::transactions::data::intents::mutation_intent::RelationMutationIntent::Delete(
            delete,
        )) => Some(delete.relation_id),
        _ => None,
    }
}

/// The relation effects a successor round must reproduce exactly once (§1.5
/// Identity): every head relation create, re-created over the same endpoints,
/// and every relation the latest round deleted, deleted again (#277).
pub(super) struct RelationIdentity {
    links: BTreeSet<Relink>,
    unlinks: BTreeSet<crate::identity::data::RelationId>,
    relinked: BTreeSet<Relink>,
    unrelinked: BTreeSet<crate::identity::data::RelationId>,
}

impl RelationIdentity {
    /// The relation effects of the chain's latest round, `latest`.
    pub(super) fn of_chain(
        bindings: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        symbols: &crate::symbols::data::StringInterner,
        latest: &[MutationIntent],
        source: EntityId,
        charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
    ) -> Result<Self, TransactionCommitError> {
        let mut identity = Self {
            links: BTreeSet::new(),
            unlinks: BTreeSet::new(),
            relinked: BTreeSet::new(),
            unrelinked: BTreeSet::new(),
        };
        for intent in latest {
            if let Some(link) = relink(bindings, symbols, intent, source, charge)? {
                identity.links.insert(link);
            } else if let Some(relation) = unrelink(intent) {
                identity.unlinks.insert(relation);
            }
        }
        Ok(identity)
    }

    /// Admit one successor intent: `true` for a relation effect the chain
    /// made and the round reproduces once, `false` for any non-relation
    /// intent. A relation effect the chain never made, or one repeated, is a
    /// `StructuralEffect`.
    pub(super) fn admit(
        &mut self,
        bindings: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        symbols: &crate::symbols::data::StringInterner,
        intent: &MutationIntent,
        source: EntityId,
        charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
    ) -> Result<bool, TransactionCommitError> {
        let admitted = if let Some(link) = relink(bindings, symbols, intent, source, charge)? {
            self.links.contains(&link) && self.relinked.insert(link)
        } else if let Some(relation) = unrelink(intent) {
            self.unlinks.contains(&relation) && self.unrelinked.insert(relation)
        } else {
            return Ok(false);
        };
        if admitted {
            Ok(true)
        } else {
            Err(postponed(Reason::StructuralEffect, source))
        }
    }

    /// Cover one latest-round intent: `true` for a relation effect the round
    /// reproduced, `false` for any non-relation intent. One it left out is an
    /// `UncoveredWrite`.
    pub(super) fn covers(
        &self,
        bindings: &crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        symbols: &crate::symbols::data::StringInterner,
        intent: &MutationIntent,
        source: EntityId,
        charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
    ) -> Result<bool, TransactionCommitError> {
        let covered = if let Some(link) = relink(bindings, symbols, intent, source, charge)? {
            self.relinked.contains(&link)
        } else if let Some(relation) = unrelink(intent) {
            self.unrelinked.contains(&relation)
        } else {
            return Ok(false);
        };
        if covered {
            Ok(true)
        } else {
            Err(postponed(Reason::UncoveredWrite, source))
        }
    }
}

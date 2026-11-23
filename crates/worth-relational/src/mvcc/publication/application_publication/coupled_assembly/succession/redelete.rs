//! A chain's entity deletes (§1.5 Identity, #277). A successor round deletes
//! again each pre-existing entity its chain's latest round deleted. A delete
//! carries no payload: over the same image, cascade policy and custody, the
//! round alone would write exactly the records the earlier round's delete
//! wrote. The fold keeps that delete and applies no second one.
use super::super::refusals::postponed;
use crate::identity::data::EntityId;
use crate::publication::data::CoupledProvenancePostponement as Reason;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use std::collections::BTreeSet;

/// The entity deletes a successor round must reproduce exactly once.
pub(super) struct Redeletes {
    /// The pre-existing entities the chain's latest round deleted.
    chain: BTreeSet<EntityId>,
    /// Those the successor round deleted again.
    proved: BTreeSet<EntityId>,
}

impl Redeletes {
    /// The deletes of the chain's latest round, `latest`. A delete of a record
    /// the chain `created` is never one: a later round refuses it as fresh.
    pub(super) fn of_chain(
        latest: &[MutationIntent],
        created: &BTreeSet<EntityId>,
        charge: &mut impl FnMut(u64) -> Result<(), TransactionCommitError>,
    ) -> Result<Self, TransactionCommitError> {
        let mut chain = BTreeSet::new();
        for intent in latest {
            charge(1)?;
            chain.extend(intent.deleted_entity().filter(|id| !created.contains(id)));
        }
        Ok(Self {
            chain,
            proved: BTreeSet::new(),
        })
    }

    /// Admit one successor intent: `true` for a delete of an entity the chain
    /// deleted, `false` for any other intent. The re-delete is that record's
    /// one write in the round (`targets`): repeated, or beside another write
    /// of the record, it is a `StructuralEffect`.
    pub(super) fn admit(
        &mut self,
        intent: &MutationIntent,
        targets: &mut BTreeSet<EntityId>,
    ) -> Result<bool, TransactionCommitError> {
        let Some(id) = intent.deleted_entity().filter(|id| self.chain.contains(id)) else {
            return Ok(false);
        };
        if !targets.insert(id) {
            return Err(postponed(Reason::StructuralEffect, id));
        }
        self.proved.insert(id);
        Ok(true)
    }

    /// Cover one latest-round intent: `true` for a delete the round
    /// reproduced, `false` for any other intent. One it left out is an
    /// `UncoveredWrite`: the fold cannot restore the record.
    pub(super) fn covers(&self, intent: &MutationIntent) -> Result<bool, TransactionCommitError> {
        let Some(id) = intent.deleted_entity().filter(|id| self.chain.contains(id)) else {
            return Ok(false);
        };
        if self.proved.contains(&id) {
            Ok(true)
        } else {
            Err(postponed(Reason::UncoveredWrite, id))
        }
    }

    /// The proved re-deletes: the only deletes the fold drops.
    pub(super) fn into_proved(self) -> BTreeSet<EntityId> {
        self.proved
    }
}

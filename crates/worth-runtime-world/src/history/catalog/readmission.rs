//! Recovered publication storage and lookup share the existing exact catalog.
use super::{lock_index, lock_state, validate_owner, CompositeHistoryCatalog};
use crate::history::retention::{CompositeHistoryProtectionObligation, HistoryProtectionClass};
use crate::history::{
    PublicationEvidence, WorldReadmittedPublication, WorldReadmittedPublicationDenial as Denial,
};
use crate::identity::CompositeCommitIdentity;

impl CompositeHistoryCatalog {
    pub(crate) fn readmitted_native_publication(
        &self,
        owner: crate::identity::RuntimeWorldOwnerIdentity,
        occurrence: &crate::durability::AdmittedWorldOccurrence,
        effect: worth_relational::facade::durability::ReadmittedRelationalAssociatedEffect,
    ) -> Result<WorldReadmittedPublication, Denial> {
        let open = occurrence.open();
        let identity = crate::identity::readmission::readmit_commit(owner, open.commit)
            .ok_or(Denial::ForeignHome)?;
        let state = lock_state(&self.state);
        validate_owner(&state, owner).map_err(Denial::History)?;
        let publication = state
            .entries
            .get(&identity)
            .and_then(|slot| slot.get())
            .and_then(|entry| entry.publication.as_ref())
            .ok_or(Denial::NotReadmitted)?;
        let record = match publication {
            PublicationEvidence::Readmitted(record) => record.clone(),
            PublicationEvidence::Live(live) => {
                let facts = live.facts().ok_or(Denial::NotReadmitted)?;
                let before = facts.movement.before();
                let after = facts.movement.after();
                let attempt = crate::identity::readmission::readmit_attempt(
                    owner,
                    open.association.attempt(),
                )
                .ok_or(Denial::ForeignHome)?;
                if before.selected_commit().ordinal() != open.expected.commit.ordinal()
                    || before.lifecycle_incarnation().ordinal()
                        != open.expected.incarnation.ordinal()
                    || before.reference_generation().get() != open.expected.generation.get()
                    || after.selected_commit() != &identity
                    || after.branch_identity().name().as_str() != open.association.branch().name()
                    || live.attempt_identity() != &attempt
                {
                    return Err(Denial::NativeMismatch);
                }
                crate::history::ReadmittedPublicationRecord::from_source(
                    before.clone(),
                    after.clone(),
                    attempt,
                    occurrence,
                    effect,
                )
            }
        };
        lock_index(&state.reachability)
            .increment_direct_protection(&identity)
            .map_err(Denial::History)?;
        Ok(WorldReadmittedPublication {
            record,
            _history: CompositeHistoryProtectionObligation::new(
                state.reachability.clone(),
                identity,
                HistoryProtectionClass::ExplicitObligation,
            ),
            _catalog: self.clone(),
        })
    }

    pub(crate) fn readmitted_publication(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Result<WorldReadmittedPublication, Denial> {
        let state = lock_state(&self.state);
        validate_owner(&state, identity.owner_identity()).map_err(Denial::History)?;
        let record = match state
            .entries
            .get(identity)
            .and_then(|slot| slot.get())
            .and_then(|entry| entry.publication.as_ref())
        {
            Some(PublicationEvidence::Readmitted(record)) => record.clone(),
            Some(PublicationEvidence::Live(_)) | None => return Err(Denial::NotReadmitted),
        };
        lock_index(&state.reachability)
            .increment_direct_protection(identity)
            .map_err(Denial::History)?;
        Ok(WorldReadmittedPublication {
            record,
            _history: CompositeHistoryProtectionObligation::new(
                state.reachability.clone(),
                identity.clone(),
                HistoryProtectionClass::ExplicitObligation,
            ),
            _catalog: self.clone(),
        })
    }
}

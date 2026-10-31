//! Recovered publication storage and lookup share the existing exact catalog.
use super::{lock_index, lock_state, validate_owner, CompositeHistoryCatalog};
use crate::history::retention::{CompositeHistoryProtectionObligation, HistoryProtectionClass};
use crate::history::{
    PublicationEvidence, WorldReadmittedPublication, WorldReadmittedPublicationDenial as Denial,
};
use crate::identity::CompositeCommitIdentity;

impl CompositeHistoryCatalog {
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

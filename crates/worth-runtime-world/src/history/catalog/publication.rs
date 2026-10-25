//! Recovery of canonical performed facts uses the history entry's original
//! exclusive delivery lane. It neither retries owner work nor moves a cell.

use std::sync::Arc;

use crate::history::{ExplicitCommitHistoryProtectionObligation, PublicationDeliveryClaim};
use crate::identity::CompositeCommitIdentity;

use super::super::retention::{CompositeHistoryProtectionObligation, HistoryProtectionClass};
use super::support::{lock_state, validate_owner};
#[cfg(test)]
use super::CompositeHistoryCatalogDenial;
use super::{lock_index, CompositeHistoryCatalog};

impl CompositeHistoryCatalog {
    #[cfg(test)]
    pub(crate) fn claim_performed_publication(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Result<Option<PublicationDeliveryClaim>, CompositeHistoryCatalogDenial> {
        match self.recover_delivery(identity) {
            Ok(claim) => Ok(Some(claim)),
            Err(crate::recovery::PerformedPublicationRecoveryDenial::Catalog(denial)) => {
                Err(denial)
            }
            Err(_) => Ok(None),
        }
    }

    pub(crate) fn recover_delivery(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Result<PublicationDeliveryClaim, crate::recovery::PerformedPublicationRecoveryDenial> {
        let state = lock_state(&self.state);
        validate_owner(&state, identity.owner_identity())?;
        use crate::recovery::PerformedPublicationRecoveryDenial as Denial;
        let entry = state
            .entries
            .get(identity)
            .and_then(|slot| slot.get())
            .ok_or(Denial::MissingCommit)?;
        let publication = entry.publication.as_ref().ok_or(Denial::NotPerformed)?;
        if publication.facts().is_none() {
            return Err(Denial::NotPerformed);
        }
        lock_index(&state.reachability).increment_direct_protection(identity)?;
        let history = ExplicitCommitHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                Arc::clone(&state.reachability),
                identity.clone(),
                HistoryProtectionClass::ExplicitObligation,
            ),
        );
        publication.try_claim_delivery(history)
    }
}

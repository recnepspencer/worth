//! Exact independent retention of one committed publication envelope.

use std::sync::Arc;

use crate::history::retention::{CompositeHistoryProtectionObligation, HistoryProtectionClass};
use crate::identity::CompositeCommitIdentity;

use super::support::{lock_state, validate_owner};
use super::{lock_index, CompositeHistoryCatalog, CompositeHistoryCatalogDenial};

/// World-issued, move-only lifetime protection for a performed publication.
/// It holds the catalog and prevents explicit reclamation of its exact commit.
#[derive(Debug)]
pub struct RuntimeWorldPerformedPublicationProtection {
    identity: CompositeCommitIdentity,
    _history: CompositeHistoryProtectionObligation,
    _catalog: CompositeHistoryCatalog,
}

impl RuntimeWorldPerformedPublicationProtection {
    pub fn commit_identity(&self) -> &CompositeCommitIdentity {
        &self.identity
    }
}

impl CompositeHistoryCatalog {
    pub(crate) fn protect_performed_publication(
        &self,
        identity: &CompositeCommitIdentity,
    ) -> Result<RuntimeWorldPerformedPublicationProtection, CompositeHistoryCatalogDenial> {
        let state = lock_state(&self.state);
        validate_owner(&state, identity.owner_identity())?;
        let entry = state
            .entries
            .get(identity)
            .and_then(|slot| slot.get())
            .ok_or_else(|| {
                CompositeHistoryCatalogDenial::UnknownProtectionTarget(identity.clone())
            })?;
        if entry
            .publication
            .as_ref()
            .is_none_or(|envelope| envelope.facts().is_none())
        {
            return Err(CompositeHistoryCatalogDenial::NotPerformedPublication(
                identity.clone(),
            ));
        }
        lock_index(&state.reachability).increment_direct_protection(identity)?;
        Ok(RuntimeWorldPerformedPublicationProtection {
            identity: identity.clone(),
            _history: CompositeHistoryProtectionObligation::new(
                Arc::clone(&state.reachability),
                identity.clone(),
                HistoryProtectionClass::ExplicitObligation,
            ),
            _catalog: self.clone(),
        })
    }
}

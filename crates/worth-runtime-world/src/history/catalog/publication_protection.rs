//! Exact independent retention of one committed publication envelope.

use std::sync::Arc;

use crate::history::retention::{CompositeHistoryProtectionObligation, HistoryProtectionClass};
use crate::history::ExplicitCommitHistoryProtectionObligation;
use crate::identity::CompositeCommitIdentity;

use super::support::{lock_state, validate_owner};
use super::{lock_index, CompositeHistoryCatalog, CompositeHistoryCatalogDenial};

/// World-issued, move-only lifetime protection for a performed publication.
/// It prevents reclamation or history retirement of its exact commit. An issued
/// protection also holds the catalog; one detached from a consumed delivery
/// keeps exactly the lifetime that delivery had.
#[derive(Debug)]
pub struct RuntimeWorldPerformedPublicationProtection {
    identity: CompositeCommitIdentity,
    _history: CompositeHistoryProtectionObligation,
    _catalog: Option<CompositeHistoryCatalog>,
}

impl RuntimeWorldPerformedPublicationProtection {
    pub fn commit_identity(&self) -> &CompositeCommitIdentity {
        &self.identity
    }

    pub(crate) fn detached_from_delivery(
        history: ExplicitCommitHistoryProtectionObligation,
    ) -> Self {
        Self {
            identity: history.commit_identity().clone(),
            _history: history.into_protection(),
            _catalog: None,
        }
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
            _catalog: Some(self.clone()),
        })
    }
}

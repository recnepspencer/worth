//! Admission of bounded history capacity before component owner effects.

use std::sync::Arc;

use crate::branch::ProductBranchReferenceSnapshot;
use crate::history::CanonicalPublicationEnvelope;
use crate::identity::{CompositeCommitIdentity, CompositePublicationAttemptIdentity};

use super::support::{lock_state, validate_owner, validate_parent_for_reservation};
#[cfg(test)]
use super::CompositeRuntimeWorldCommit;
use super::{
    counters, lock_index, metadata, CompositeCommitParent, CompositeHistoryCatalog,
    CompositeHistoryCatalogDenial, HistoryReservationMetadata, ReservedCompositeCommitCapacity,
};

impl CompositeHistoryCatalog {
    #[cfg(test)]
    pub(crate) fn reserve(
        &self,
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<ReservedCompositeCommitCapacity, CompositeHistoryCatalogDenial> {
        self.reserve_commit_capacity(commit.identity().clone(), commit.parent().clone())
    }

    /// Reserve immutable commit and metadata capacity before a commit exists.
    /// The owner supplies the identity and exact parent; installation later
    /// accepts only the matching immutable commit.
    pub(crate) fn reserve_commit_capacity(
        &self,
        identity: CompositeCommitIdentity,
        parent: CompositeCommitParent,
    ) -> Result<ReservedCompositeCommitCapacity, CompositeHistoryCatalogDenial> {
        self.reserve_capacity(identity, parent, None)
    }

    pub(crate) fn reserve_publication_capacity(
        &self,
        identity: CompositeCommitIdentity,
        attempt: CompositePublicationAttemptIdentity,
        expected: ProductBranchReferenceSnapshot,
    ) -> Result<ReservedCompositeCommitCapacity, CompositeHistoryCatalogDenial> {
        for actual in [attempt.owner_identity(), expected.owner_identity()] {
            if actual != identity.owner_identity() {
                return Err(CompositeHistoryCatalogDenial::ForeignOwner {
                    expected: identity.owner_identity(),
                    actual,
                });
            }
        }
        let parent = CompositeCommitParent::Ordinary(crate::history::OrdinaryParent::new(
            expected.selected_commit().clone(),
        ));
        let publication =
            CanonicalPublicationEnvelope::reserve(identity.clone(), attempt, expected);
        self.reserve_capacity(identity, parent, Some(publication))
    }

    fn reserve_capacity(
        &self,
        identity: CompositeCommitIdentity,
        parent: CompositeCommitParent,
        publication: Option<Arc<CanonicalPublicationEnvelope>>,
    ) -> Result<ReservedCompositeCommitCapacity, CompositeHistoryCatalogDenial> {
        let mut state = lock_state(&self.state);
        validate_owner(&state, identity.owner_identity())?;
        if state.entries.contains_key(&identity) || state.reservations.contains_key(&identity) {
            return Err(CompositeHistoryCatalogDenial::DuplicateCommit);
        }
        if matches!(parent, CompositeCommitParent::Root)
            && (state.root.is_some() || state.root_reserved || state.root_ever_installed)
        {
            return Err(CompositeHistoryCatalogDenial::RootAlreadyInstalled);
        }
        validate_parent_for_reservation(&mut state, &parent)?;
        let maximum_commits = state.limits.maximum_commits().get();
        if state.entries.len() >= maximum_commits {
            return Err(CompositeHistoryCatalogDenial::CommitCapacityExhausted {
                maximum: maximum_commits,
            });
        }

        let commit_charge = metadata::HistoryMetadataCharge::for_parent(&parent)
            .and_then(|charge| charge.with_publication(publication.as_deref()))
            .map_err(|_| CompositeHistoryCatalogDenial::ArithmeticOverflow)?;
        let reservation_charge = metadata::HistoryReservationCharge::for_parent(&parent)
            .map_err(|_| CompositeHistoryCatalogDenial::ArithmeticOverflow)?;
        let preview = {
            counters::lock_counters(&state.counters).record_metadata_reservation_check();
            state
                .metadata
                .preview_reservation(
                    reservation_charge,
                    commit_charge,
                    state.limits.maximum_metadata_bytes().get(),
                )
                .map_err(|denial| match denial {
                    metadata::HistoryMetadataLedgerDenial::ArithmeticOverflow => {
                        CompositeHistoryCatalogDenial::ArithmeticOverflow
                    }
                    metadata::HistoryMetadataLedgerDenial::Capacity {
                        maximum,
                        used,
                        requested,
                    } => CompositeHistoryCatalogDenial::MetadataCapacityExhausted {
                        maximum,
                        used,
                        requested,
                    },
                })?
        };

        if let CompositeCommitParent::Ordinary(parent) = &parent {
            let mut reachability = lock_index(&state.reachability);
            reachability.increment_descendant_dependency(parent.commit())?;
        }
        // Populate the eventual indexes now. Promotion fills these resident
        // slots; it never asks either lookup index to allocate after effects.
        let slots = super::slots::ReservedHistorySlots::reserve(&mut state, &identity);
        state.metadata.reserve_confirmed(preview);
        counters::lock_counters(&state.counters).record_metadata_reservation();
        let reservation = HistoryReservationMetadata {
            parent: parent.clone(),
            commit_charge,
            reservation_charge,
        };
        assert!(state
            .reservations
            .insert(identity.clone(), reservation.clone())
            .is_none());
        if matches!(parent, CompositeCommitParent::Root) {
            state.root_reserved = true;
        }
        Ok(ReservedCompositeCommitCapacity::new(
            Arc::clone(&self.state),
            identity,
            reservation,
            slots,
            publication,
        ))
    }
}

//! Admission of bounded history capacity before component owner effects.

use std::sync::Arc;

use super::reservation::ReservedHistoryPublication;
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
        self.reserve_capacity(identity, parent, ReservedHistoryPublication::NoPublication)
    }

    pub(crate) fn reserve_readmitted_capacity(
        &self,
        identity: CompositeCommitIdentity,
        parent: CompositeCommitParent,
    ) -> Result<ReservedCompositeCommitCapacity, CompositeHistoryCatalogDenial> {
        self.reserve_capacity(identity, parent, ReservedHistoryPublication::Readmitted)
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
        let revision = Arc::clone(&lock_state(&self.state).publication_revision);
        let publication =
            CanonicalPublicationEnvelope::reserve(identity.clone(), attempt, expected, revision);
        self.reserve_capacity(
            identity,
            parent,
            ReservedHistoryPublication::Live(publication),
        )
    }

    fn reserve_capacity(
        &self,
        identity: CompositeCommitIdentity,
        parent: CompositeCommitParent,
        publication: ReservedHistoryPublication,
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
        if let Some(maximum_commits) =
            maximum_commits.filter(|maximum| state.entries.len() >= *maximum)
        {
            return Err(CompositeHistoryCatalogDenial::CommitCapacityExhausted {
                maximum: maximum_commits,
            });
        }

        let commit_charge = metadata::HistoryMetadataCharge::for_parent(&parent)
            .and_then(|charge| publication.charge(charge))
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
        assert!(state.inspection_order.insert(identity.clone()));
        state.publication_revision.advance();
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

    /// Install a surviving commit whose direct parent was durably disposed
    /// before reopen. Preserves the commit's authoritative `Ordinary(parent)`
    /// while wiring catalog reachability and spliced edge distance to its
    /// nearest surviving installed ancestor (if any).
    pub(crate) fn install_readmitted_horizon_commit(
        &self,
        commit: Arc<super::CompositeRuntimeWorldCommit>,
        pins: crate::retention::HistoryRetentionObligation,
        spliced_parent: Option<(CompositeCommitIdentity, usize)>,
    ) -> Result<crate::history::ProductHeadHistoryProtectionObligation, CompositeHistoryCatalogDenial>
    {
        use crate::history::retention::{
            CompositeHistoryProtectionObligation, HistoryProtectionClass,
            ProductHeadHistoryProtectionObligation,
        };
        if !pins.matches_basis(commit.basis()) {
            return Err(CompositeHistoryCatalogDenial::HistoryPinBasisMismatch);
        }
        let identity = commit.identity().clone();
        let mut state = lock_state(&self.state);
        validate_owner(&state, identity.owner_identity())?;
        if state.entries.contains_key(&identity) || state.reservations.contains_key(&identity) {
            return Err(CompositeHistoryCatalogDenial::DuplicateCommit);
        }
        if let Some((ancestor, generations)) = &spliced_parent {
            if ancestor.owner_identity() != state.owner {
                return Err(CompositeHistoryCatalogDenial::ForeignParent {
                    expected: state.owner,
                    actual: ancestor.owner_identity(),
                });
            }
            if *generations < 2
                || state.retiring.contains(ancestor)
                || state
                    .entries
                    .get(ancestor)
                    .is_none_or(|slot| slot.get().is_none())
            {
                return Err(CompositeHistoryCatalogDenial::MissingParent(
                    ancestor.clone(),
                ));
            }
        }
        let maximum_commits = state.limits.maximum_commits().get();
        if let Some(maximum_commits) =
            maximum_commits.filter(|maximum| state.entries.len() >= *maximum)
        {
            return Err(CompositeHistoryCatalogDenial::CommitCapacityExhausted {
                maximum: maximum_commits,
            });
        }
        let commit_charge = metadata::HistoryMetadataCharge::for_commit(&commit)
            .map_err(|_| CompositeHistoryCatalogDenial::ArithmeticOverflow)?;
        let reservation_charge = metadata::HistoryReservationCharge::for_parent(commit.parent())
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
        if let Some((ancestor, _)) = &spliced_parent {
            lock_index(&state.reachability).increment_descendant_dependency(ancestor)?;
        }
        let slots = super::slots::ReservedHistorySlots::reserve(&mut state, &identity);
        assert!(state.inspection_order.insert(identity.clone()));
        state.metadata.reserve_confirmed(preview);
        state.metadata.promote(reservation_charge, commit_charge);
        counters::lock_counters(&state.counters).record_metadata_promotion();
        let publication = if commit.relational_publication_identity().is_some() {
            Some(crate::history::PublicationEvidence::ReadmittedHorizon)
        } else {
            None
        };
        let entry = super::entry::CompositeHistoryCatalogEntry {
            pins,
            commit: Some(commit),
            publication,
            metadata_charge: commit_charge,
        };
        let marks_root = spliced_parent.is_none();
        slots.install_with_parent(
            &mut state,
            entry,
            spliced_parent.as_ref().map(|(ancestor, _)| ancestor),
            marks_root,
        );
        if let Some((parent, generations)) = spliced_parent {
            state.spliced.insert(
                identity.clone(),
                super::history_retirement::SplicedParent {
                    parent,
                    generations,
                },
            );
        }
        let reachability = Arc::clone(&state.reachability);
        lock_index(&reachability).protect_reserved_slot(&slots.reachability);
        state.publication_revision.advance();
        Ok(ProductHeadHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                reachability,
                identity,
                HistoryProtectionClass::ProductHead,
            ),
        ))
    }
}

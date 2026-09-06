use std::sync::{Arc, Mutex};

use super::denial::CompositeHistoryCatalogDenial;
use super::entry::CompositeHistoryCatalogEntry;
use super::metadata::{HistoryMetadataCharge, HistoryReservationMetadata};
use super::slots::ReservedHistorySlots;
use super::support::{lock_state, release_reservation};
use super::CompositeHistoryCatalogState;
use super::CompositeRuntimeWorldCommit;

/// Rollback custody for the one root entry staged during bootstrap. The
/// guard is armed only after installation and is disarmed after every root
/// artifact has been installed in its owning registry.
#[must_use = "a staged root history entry must be committed or rolled back"]
pub(crate) struct InstalledRootCommitRollback {
    state: Arc<Mutex<CompositeHistoryCatalogState>>,
    identity: crate::identity::CompositeCommitIdentity,
    armed: bool,
}

impl std::fmt::Debug for InstalledRootCommitRollback {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("InstalledRootCommitRollback")
            .field("identity", &self.identity)
            .field("armed", &self.armed)
            .finish_non_exhaustive()
    }
}

impl Drop for InstalledRootCommitRollback {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = lock_state(&self.state);
        let entry = super::support::remove_installed(&mut state, &self.identity);
        assert!(
            matches!(
                entry.get().expect("installed root").commit().parent(),
                super::CompositeCommitParent::Root
            ),
            "bootstrap rollback owns only the root history entry"
        );
        state.root_reserved = false;
        state.root_ever_installed = false;
        self.armed = false;
        drop(state);
        drop(entry);
    }
}

impl InstalledRootCommitRollback {
    pub(super) fn new(
        state: Arc<Mutex<CompositeHistoryCatalogState>>,
        identity: crate::identity::CompositeCommitIdentity,
    ) -> Self {
        Self {
            state,
            identity,
            armed: true,
        }
    }

    pub(crate) fn commit(mut self) {
        self.armed = false;
    }
}

#[must_use = "reserved commit capacity must be installed or dropped"]
pub(crate) struct ReservedCompositeCommitCapacity {
    state: Arc<Mutex<CompositeHistoryCatalogState>>,
    identity: crate::identity::CompositeCommitIdentity,
    reservation: HistoryReservationMetadata,
    slots: Option<ReservedHistorySlots>,
    publication: Option<Arc<crate::history::CanonicalPublicationEnvelope>>,
    armed: bool,
}

impl std::fmt::Debug for ReservedCompositeCommitCapacity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReservedCompositeCommitCapacity")
            .field("identity", &self.identity)
            .field("parent", &self.reservation.parent)
            .field("metadata_charge", &self.reservation.commit_charge)
            .finish_non_exhaustive()
    }
}

impl Drop for ReservedCompositeCommitCapacity {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = lock_state(&self.state);
        release_reservation(&mut state, &self.identity);
        self.armed = false;
    }
}

impl ReservedCompositeCommitCapacity {
    pub(crate) fn publication_envelope(
        &self,
    ) -> Option<&Arc<crate::history::CanonicalPublicationEnvelope>> {
        self.publication.as_ref()
    }

    pub(super) fn new(
        state: Arc<Mutex<CompositeHistoryCatalogState>>,
        identity: crate::identity::CompositeCommitIdentity,
        reservation: HistoryReservationMetadata,
        slots: ReservedHistorySlots,
        publication: Option<Arc<crate::history::CanonicalPublicationEnvelope>>,
    ) -> Self {
        Self {
            state,
            identity,
            reservation,
            slots: Some(slots),
            publication,
            armed: true,
        }
    }

    pub(crate) fn install(
        mut self,
        commit: Arc<CompositeRuntimeWorldCommit>,
        pins: crate::retention::HistoryRetentionObligation,
    ) -> Result<Arc<CompositeRuntimeWorldCommit>, CompositeHistoryCatalogDenial> {
        let mut state = lock_state(&self.state);
        self.validate_installation(&state, &commit, &pins)?;
        let entry = promote_reserved_commit(
            &mut state,
            &self.identity,
            self.slots.as_ref().expect("live reserved slots"),
            commit,
            self.publication.clone(),
            pins,
        );
        self.armed = false;
        self.slots = None;
        Ok(entry)
    }

    /// Install and protect under the same history-owner lock. Denial preserves
    /// this reservation; a successful result already owns the new entry's
    /// first direct protection, so reclamation cannot enter an unprotected gap.
    pub(crate) fn try_install_product_head(
        &mut self,
        commit: Arc<CompositeRuntimeWorldCommit>,
        pins: crate::retention::HistoryRetentionObligation,
    ) -> Result<crate::history::ProductHeadHistoryProtectionObligation, CompositeHistoryCatalogDenial>
    {
        use crate::history::retention::{
            CompositeHistoryProtectionObligation, HistoryProtectionClass,
            ProductHeadHistoryProtectionObligation,
        };
        let mut state = lock_state(&self.state);
        self.validate_installation(&state, &commit, &pins)?;
        let identity = self.identity.clone();
        let reachability = Arc::clone(&state.reachability);
        promote_reserved_commit(
            &mut state,
            &self.identity,
            self.slots.as_ref().expect("live reserved slots"),
            commit,
            self.publication.clone(),
            pins,
        );
        super::lock_index(&reachability).protect_reserved_slot(
            &self
                .slots
                .as_ref()
                .expect("live reserved slots")
                .reachability,
        );
        self.armed = false;
        self.slots = None;
        Ok(ProductHeadHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                reachability,
                identity,
                HistoryProtectionClass::ProductHead,
            ),
        ))
    }

    /// Install the publisher's delivery protection in the same transaction as
    /// the new head protection. Both counts are fixed for this fresh entry;
    /// no protection capacity is acquired after a successful product CAS.
    pub(crate) fn try_install_publication(
        &mut self,
        commit: Arc<CompositeRuntimeWorldCommit>,
        pins: crate::retention::HistoryRetentionObligation,
    ) -> Result<
        (
            crate::history::ProductHeadHistoryProtectionObligation,
            crate::history::PublicationDeliveryClaim,
        ),
        CompositeHistoryCatalogDenial,
    > {
        use crate::history::retention::{
            CompositeHistoryProtectionObligation, ExplicitCommitHistoryProtectionObligation,
            HistoryProtectionClass, ProductHeadHistoryProtectionObligation,
        };
        let publication = self
            .publication
            .as_ref()
            .ok_or(CompositeHistoryCatalogDenial::ReservationCommitMismatch)?;
        let mut state = lock_state(&self.state);
        self.validate_installation(&state, &commit, &pins)?;
        if commit.provenance()
            != &crate::history::CompositeCommitProvenance::Publication(
                publication.attempt_identity().clone(),
            )
        {
            return Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch);
        }
        let identity = self.identity.clone();
        let delivery_identity = identity.clone();
        let reachability = Arc::clone(&state.reachability);
        let delivery_reachability = Arc::clone(&reachability);
        promote_reserved_commit(
            &mut state,
            &self.identity,
            self.slots.as_ref().expect("live reserved slots"),
            commit,
            self.publication.clone(),
            pins,
        );
        {
            let mut index = super::lock_index(&reachability);
            index.protect_reserved_slot(
                &self
                    .slots
                    .as_ref()
                    .expect("live reserved slots")
                    .reachability,
            );
            index
                .increment_reserved_protection(
                    &self
                        .slots
                        .as_ref()
                        .expect("live reserved slots")
                        .reachability,
                    &identity,
                )
                .expect("a new entry with one head protection has room for its delivery");
        }
        self.armed = false;
        self.slots = None;
        let head = ProductHeadHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                reachability,
                identity,
                HistoryProtectionClass::ProductHead,
            ),
        );
        let delivery_history = ExplicitCommitHistoryProtectionObligation::issued(
            CompositeHistoryProtectionObligation::new(
                delivery_reachability,
                delivery_identity,
                HistoryProtectionClass::ExplicitObligation,
            ),
        );
        let delivery = publication
            .claim_delivery(delivery_history)
            .expect("a newly installed publication has no earlier delivery claim");
        Ok((head, delivery))
    }

    fn validate_installation(
        &self,
        state: &CompositeHistoryCatalogState,
        commit: &CompositeRuntimeWorldCommit,
        pins: &crate::retention::HistoryRetentionObligation,
    ) -> Result<(), CompositeHistoryCatalogDenial> {
        if !pins.matches_basis(commit.basis()) {
            return Err(CompositeHistoryCatalogDenial::HistoryPinBasisMismatch);
        }
        if !self.armed {
            return Err(CompositeHistoryCatalogDenial::ReservationMissing);
        }
        let actual_charge = HistoryMetadataCharge::for_commit(commit)
            .and_then(|charge| charge.with_publication(self.publication.as_deref()))
            .map_err(|_| CompositeHistoryCatalogDenial::ArithmeticOverflow)?;
        let Some(reservation) = state.reservations.get(&self.identity) else {
            return Err(CompositeHistoryCatalogDenial::ReservationMissing);
        };
        if commit.identity() != &self.identity {
            return Err(CompositeHistoryCatalogDenial::ReservationCommitMismatch);
        }
        if commit.parent() != &reservation.parent || commit.parent() != &self.reservation.parent {
            return Err(CompositeHistoryCatalogDenial::ReservationParentMismatch);
        }
        if actual_charge != reservation.commit_charge
            || actual_charge != self.reservation.commit_charge
        {
            return Err(CompositeHistoryCatalogDenial::ReservationChargeMismatch);
        }
        Ok(())
    }
}

fn promote_reserved_commit(
    state: &mut CompositeHistoryCatalogState,
    identity: &crate::identity::CompositeCommitIdentity,
    slots: &ReservedHistorySlots,
    commit: Arc<CompositeRuntimeWorldCommit>,
    publication: Option<Arc<crate::history::CanonicalPublicationEnvelope>>,
    pins: crate::retention::HistoryRetentionObligation,
) -> Arc<CompositeRuntimeWorldCommit> {
    let reservation = state
        .reservations
        .remove(identity)
        .expect("validated live reservation");
    state
        .metadata
        .promote(reservation.reservation_charge, reservation.commit_charge);
    super::counters::lock_counters(&state.counters).record_metadata_promotion();
    if matches!(reservation.parent, super::CompositeCommitParent::Root) {
        state.root_reserved = false;
    }
    let result = Arc::clone(&commit);
    let entry = CompositeHistoryCatalogEntry {
        _pins: pins,
        commit,
        publication,
        metadata_charge: reservation.commit_charge,
    };
    slots.install(state, entry);
    result
}

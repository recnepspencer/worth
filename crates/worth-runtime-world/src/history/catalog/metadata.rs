use std::mem::size_of;
use std::sync::{Arc, Mutex, OnceLock};

use crate::identity::CompositeCommitIdentity;

use super::entry::CompositeHistoryCatalogEntry;
use super::reachability::HistoryReachabilityRecord;
use super::{CompositeCommitParent, CompositeRuntimeWorldCommit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HistoryMetadataCharge {
    commit_record: usize,
    arc_history: usize,
    lookup_index_key: usize,
    lookup_index_value: usize,
    reachability_key: usize,
    reachability_row: usize,
    slot_allocations: usize,
    publication_envelope: usize,
    publication_branch_name: usize,
    total: usize,
}

impl HistoryMetadataCharge {
    pub(super) fn for_parent(
        _parent: &CompositeCommitParent,
    ) -> Result<Self, HistoryMetadataArithmeticOverflow> {
        let commit_record = size_of::<CompositeRuntimeWorldCommit>();
        let arc_history = size_of::<Arc<CompositeRuntimeWorldCommit>>();
        let lookup_index_key = size_of::<CompositeCommitIdentity>();
        let lookup_index_value = size_of::<Arc<OnceLock<CompositeHistoryCatalogEntry>>>();
        let reachability_key = size_of::<CompositeCommitIdentity>();
        let reachability_row = size_of::<Arc<Mutex<Option<HistoryReachabilityRecord>>>>();
        let slot_allocations = checked_sum([
            size_of::<OnceLock<CompositeHistoryCatalogEntry>>(),
            size_of::<Mutex<Option<HistoryReachabilityRecord>>>(),
            4 * size_of::<usize>(),
        ])?;
        let total = checked_sum([
            commit_record,
            arc_history,
            lookup_index_key,
            lookup_index_value,
            reachability_key,
            reachability_row,
            slot_allocations,
        ])?;
        Ok(Self {
            commit_record,
            arc_history,
            lookup_index_key,
            lookup_index_value,
            reachability_key,
            reachability_row,
            slot_allocations,
            publication_envelope: 0,
            publication_branch_name: 0,
            total,
        })
    }

    pub(super) fn for_commit(
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<Self, HistoryMetadataArithmeticOverflow> {
        Self::for_parent(commit.parent())
    }

    pub(super) fn with_publication(
        mut self,
        publication: Option<&crate::history::CanonicalPublicationEnvelope>,
    ) -> Result<Self, HistoryMetadataArithmeticOverflow> {
        if let Some(publication) = publication {
            self.publication_envelope = checked_sum([
                size_of::<crate::history::CanonicalPublicationEnvelope>(),
                size_of::<Arc<crate::history::CanonicalPublicationEnvelope>>(),
            ])?;
            // A conservative per-envelope charge for its retained shared
            // name, not a claim to measure unique allocator-resident bytes.
            self.publication_branch_name = checked_sum([
                size_of::<Arc<str>>(),
                publication.branch_name().as_str().len(),
            ])?;
            self.total = checked_sum([
                self.total,
                self.publication_envelope,
                self.publication_branch_name,
            ])?;
        }
        Ok(self)
    }

    pub(super) const fn total(self) -> usize {
        self.total
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HistoryReservationCharge {
    reservation_key: usize,
    reservation_value: usize,
    held_identities: usize,
    reservation_handles: usize,
    total: usize,
}

impl HistoryReservationCharge {
    pub(super) fn for_parent(
        parent: &CompositeCommitParent,
    ) -> Result<Self, HistoryMetadataArithmeticOverflow> {
        let reservation_key = size_of::<CompositeCommitIdentity>();
        let reservation_value = size_of::<HistoryReservationMetadata>();
        let parent_identity = match parent {
            CompositeCommitParent::Root => 0,
            CompositeCommitParent::Ordinary(_) => size_of::<CompositeCommitIdentity>(),
        };
        let held_identities = checked_sum([size_of::<CompositeCommitIdentity>(), parent_identity])?;
        let reservation_handles = checked_sum([
            size_of::<Arc<Mutex<super::CompositeHistoryCatalogState>>>(),
            size_of::<super::slots::ReservedHistorySlots>(),
        ])?;
        let total = checked_sum([
            reservation_key,
            reservation_value,
            held_identities,
            reservation_handles,
        ])?;
        Ok(Self {
            reservation_key,
            reservation_value,
            held_identities,
            reservation_handles,
            total,
        })
    }

    pub(super) const fn total(self) -> usize {
        self.total
    }
}

/// The catalog's reservation value keeps the exact parent and both logical
/// charge plans. Installation transfers these values instead of recomputing
/// or reacquiring any capacity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::history) struct HistoryReservationMetadata {
    pub(super) parent: CompositeCommitParent,
    pub(super) commit_charge: HistoryMetadataCharge,
    pub(super) reservation_charge: HistoryReservationCharge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HistoryMetadataReservationPreview {
    pub(super) reservation_resident: usize,
    pub(super) promised_installation: usize,
    pub(super) total_occupancy: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct HistoryMetadataArithmeticOverflow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HistoryMetadataLedgerDenial {
    ArithmeticOverflow,
    Capacity {
        maximum: usize,
        used: usize,
        requested: usize,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HistoryMetadataLedger {
    installed_resident: usize,
    reservation_resident: usize,
    promised_installation: usize,
    total_occupancy: usize,
}

impl HistoryMetadataLedger {
    pub(super) fn preview_reservation(
        &self,
        reservation: HistoryReservationCharge,
        promised: HistoryMetadataCharge,
        maximum: usize,
    ) -> Result<HistoryMetadataReservationPreview, HistoryMetadataLedgerDenial> {
        let requested = reservation
            .total()
            .checked_add(promised.total())
            .ok_or(HistoryMetadataLedgerDenial::ArithmeticOverflow)?;
        let total_occupancy = self
            .total_occupancy
            .checked_add(requested)
            .ok_or(HistoryMetadataLedgerDenial::ArithmeticOverflow)?;
        let reservation_resident = self
            .reservation_resident
            .checked_add(reservation.total())
            .ok_or(HistoryMetadataLedgerDenial::ArithmeticOverflow)?;
        let promised_installation = self
            .promised_installation
            .checked_add(promised.total())
            .ok_or(HistoryMetadataLedgerDenial::ArithmeticOverflow)?;
        if total_occupancy > maximum {
            return Err(HistoryMetadataLedgerDenial::Capacity {
                maximum,
                used: self.total_occupancy,
                requested,
            });
        }
        Ok(HistoryMetadataReservationPreview {
            reservation_resident,
            promised_installation,
            total_occupancy,
        })
    }

    pub(super) fn reserve_confirmed(&mut self, preview: HistoryMetadataReservationPreview) {
        self.reservation_resident = preview.reservation_resident;
        self.promised_installation = preview.promised_installation;
        self.total_occupancy = preview.total_occupancy;
    }

    pub(super) fn promote(
        &mut self,
        reservation: HistoryReservationCharge,
        promised: HistoryMetadataCharge,
    ) {
        let reservation_total = reservation.total();
        let promised_total = promised.total();
        self.reservation_resident = self
            .reservation_resident
            .checked_sub(reservation_total)
            .expect("installed history promotion owns its reservation charge");
        self.promised_installation = self
            .promised_installation
            .checked_sub(promised_total)
            .expect("installed history promotion owns its promise charge");
        self.installed_resident = self
            .installed_resident
            .checked_add(promised_total)
            .expect("reserved history capacity makes promotion bounded");
        self.total_occupancy = self
            .total_occupancy
            .checked_sub(reservation_total)
            .expect("installed history promotion releases its reservation resident charge");
    }

    pub(super) fn release_reservation(&mut self, metadata: &HistoryReservationMetadata) {
        self.reservation_resident = self
            .reservation_resident
            .checked_sub(metadata.reservation_charge.total())
            .expect("a dropped slot owns its reservation resident charge");
        self.promised_installation = self
            .promised_installation
            .checked_sub(metadata.commit_charge.total())
            .expect("a dropped slot owns its promised installation charge");
        self.total_occupancy = self
            .total_occupancy
            .checked_sub(
                metadata
                    .reservation_charge
                    .total()
                    .checked_add(metadata.commit_charge.total())
                    .expect("reserved metadata charge was checked before installation"),
            )
            .expect("a dropped slot owns its total occupancy");
    }

    pub(super) fn release_installed(&mut self, charge: HistoryMetadataCharge) {
        self.installed_resident = self
            .installed_resident
            .checked_sub(charge.total())
            .expect("a reclaimed entry owns its installed resident charge");
        self.total_occupancy = self
            .total_occupancy
            .checked_sub(charge.total())
            .expect("a reclaimed entry owns its total occupancy");
    }

    pub const fn installed_resident(self) -> usize {
        self.installed_resident
    }

    pub const fn reservation_resident(self) -> usize {
        self.reservation_resident
    }

    pub const fn promised_installation(self) -> usize {
        self.promised_installation
    }

    pub const fn total_occupancy(self) -> usize {
        self.total_occupancy
    }
}

fn checked_sum<const N: usize>(
    parts: [usize; N],
) -> Result<usize, HistoryMetadataArithmeticOverflow> {
    parts.into_iter().try_fold(0usize, |total, part| {
        total
            .checked_add(part)
            .ok_or(HistoryMetadataArithmeticOverflow)
    })
}

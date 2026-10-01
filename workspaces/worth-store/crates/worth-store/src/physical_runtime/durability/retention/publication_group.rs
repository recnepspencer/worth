use std::sync::{Arc, Weak};

use super::{PhysicalPublicationAdmission, PhysicalRetentionGrowthDenial};
use crate::physical_runtime::RootNamespaceDurablePhysicalMutationMembers;

/// One physical WAL group; all members are reserved in the same segment.
#[derive(Clone, Copy)]
pub(in crate::physical_runtime) struct WalPublicationReservation {
    segment: u64,
    generation: u64,
    start: u64,
    end: u64,
    wal_bytes: u64,
    metadata_bytes: u64,
}

pub(super) struct RetainedWalPublication {
    reservation: WalPublicationReservation,
    settled: bool,
}

pub(in crate::physical_runtime) struct WalPublicationLease {
    admission: Weak<PhysicalPublicationAdmission>,
    start: u64,
}

impl WalPublicationReservation {
    pub(in crate::physical_runtime) fn new(
        segment: (u64, u64),
        range: worth_store_wal::WalLsnRange,
        wal_bytes: u64,
        metadata_bytes: u64,
    ) -> Option<Self> {
        wal_bytes.checked_add(metadata_bytes)?;
        (wal_bytes != 0 && metadata_bytes != 0).then_some(Self {
            segment: segment.0,
            generation: segment.1,
            start: range.start().get(),
            end: range.end_exclusive().get(),
            wal_bytes,
            metadata_bytes,
        })
    }

    fn total_bytes(self) -> u64 {
        // Construction checks the sum; settlement only decreases metadata.
        self.wal_bytes + self.metadata_bytes
    }
}

impl PhysicalPublicationAdmission {
    /// Conservative bootstrap bound: one independently allocated B-tree node
    /// per entry, including spare key/value slots, child edges and allocator
    /// overhead. The ordinary admission map and the reconstruction root set
    /// are simultaneously live; segment bookkeeping also keeps its Vec copy.
    pub(in crate::physical_runtime) fn reopened_metadata_memory_bound(
        groups: u64,
        segments: u64,
    ) -> Option<u64> {
        let node = |entry_bytes: usize| -> u64 {
            (12 * entry_bytes + 16 * std::mem::size_of::<usize>() + 64) as u64
        };
        let group_bytes = node(std::mem::size_of::<(u64, RetainedWalPublication)>())
            + node(std::mem::size_of::<u64>());
        let segment_bytes = node(std::mem::size_of::<((u64, u64), u64)>())
            + 3 * std::mem::size_of::<(u64, u64, u64)>() as u64
            + 64;
        groups
            .checked_mul(group_bytes)?
            .checked_add(segments.checked_mul(segment_bytes)?)
    }

    /// Reconstructs a verified retained group before this owner admits work.
    /// Missing publication evidence retains the full admission ceiling.
    pub(in crate::physical_runtime) fn restore_wal_publication(
        &self,
        mut reservation: WalPublicationReservation,
        published_metadata: Option<u64>,
    ) -> Result<(), ()> {
        if let Some(actual) = published_metadata {
            if actual > reservation.metadata_bytes {
                return Err(());
            }
            reservation.metadata_bytes = actual;
        }
        let mut state = self.lock();
        if state.wal_publications.contains_key(&reservation.start) {
            return Err(());
        }
        let total = state
            .charged_bytes
            .checked_add(reservation.total_bytes())
            .ok_or(())?;
        state.wal_publications.insert(
            reservation.start,
            RetainedWalPublication {
                reservation,
                settled: published_metadata.is_some(),
            },
        );
        state.charged_bytes = total;
        Ok(())
    }

    /// Registers the exact group while reserving its conservative ceiling.
    /// A no-effect lease drop removes both; any possible effect seals both.
    pub(in crate::physical_runtime) fn reserve_wal_publication(
        self: &Arc<Self>,
        reservation: WalPublicationReservation,
    ) -> Result<WalPublicationLease, PhysicalRetentionGrowthDenial> {
        let mut state = self.lock();
        let bytes = reservation.total_bytes();
        if bytes > state.remaining_bytes()
            || state.wal_publications.contains_key(&reservation.start)
        {
            return Err(PhysicalRetentionGrowthDenial {
                requested_bytes: bytes,
                requested_entries: 1,
                remaining_bytes: state.remaining_bytes(),
                remaining_entries: state.remaining_entries(),
            });
        }
        state.charged_bytes += bytes;
        state.wal_publications.insert(
            reservation.start,
            RetainedWalPublication {
                reservation,
                settled: false,
            },
        );
        Ok(WalPublicationLease {
            admission: Arc::downgrade(self),
            start: reservation.start,
        })
    }

    /// Only the namespace-durable publication can reduce its own reservation.
    /// The root owner calls this under its publication lock after validating
    /// the transition and before the infallible current-root installation.
    pub(in crate::physical_runtime) fn settle_wal_publication(
        &self,
        durable: &RootNamespaceDurablePhysicalMutationMembers,
    ) -> Result<(), ()> {
        let members = durable.settled_members();
        let start = members
            .first()
            .ok_or(())?
            .wal_member_basis()
            .lsn_range()
            .start()
            .get();
        let mut end = start;
        let mut wal_bytes = 0_u64;
        for member in members {
            let range = member.wal_member_basis().lsn_range();
            if range.start().get() != end {
                return Err(());
            }
            end = range.end_exclusive().get();
            wal_bytes = wal_bytes
                .checked_add(member.wal_append_settlement().range().byte_count())
                .ok_or(())?;
        }
        let metadata = durable.retained_routing_metadata_bytes().ok_or(())?;
        self.settle_exact_group(start, end, wal_bytes, metadata)
    }

    fn settle_exact_group(
        &self,
        start: u64,
        end: u64,
        wal_bytes: u64,
        metadata: u64,
    ) -> Result<(), ()> {
        let mut state = self.lock();
        let entry = state.wal_publications.get_mut(&start).ok_or(())?;
        if entry.settled || entry.reservation.end != end || entry.reservation.wal_bytes != wal_bytes
        {
            return Err(());
        }
        let refund = entry
            .reservation
            .metadata_bytes
            .checked_sub(metadata)
            .ok_or(())?;
        entry.reservation.metadata_bytes = metadata;
        entry.settled = true;
        state.charged_bytes = state
            .charged_bytes
            .checked_sub(refund)
            .expect("reserved group charge is retained");
        Ok(())
    }
}

impl super::AdmissionState {
    pub(super) fn release_wal_publication_groups(&mut self, segment: u64, generation: u64) -> u64 {
        let mut released = 0_u64;
        self.wal_publications.retain(|_, entry| {
            let charge = entry.reservation;
            if (charge.segment, charge.generation) != (segment, generation) {
                return true;
            }
            released = released
                .checked_add(charge.total_bytes())
                .expect("groups partition retained charge");
            false
        });
        released
    }
}

impl WalPublicationLease {
    pub(in crate::physical_runtime) fn seal(mut self) {
        self.admission = Weak::new();
    }
}

impl Drop for WalPublicationLease {
    fn drop(&mut self) {
        let Some(admission) = self.admission.upgrade() else {
            return;
        };
        let mut state = admission.lock();
        if let Some(entry) = state.wal_publications.remove(&self.start) {
            state.charged_bytes = state
                .charged_bytes
                .checked_sub(entry.reservation.total_bytes())
                .expect("a no-effect group releases its own reservation");
        }
    }
}

#[cfg(test)]
#[path = "publication_group_tests.rs"]
mod tests;

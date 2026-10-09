//! Native backing for qualified WAL paths, opens, listing, results and diagnostics.

use std::num::NonZeroU64;

use worth_foundational::LimitDimension;
use worth_store_buffer_pool::OperationAllocationGrant;
use worth_store_physical_backend::{
    BoundedRecoveryFilesystemDiscovery, GrantedRead, GrantedReadStop, ObservedWalArtifact,
    ReadGrant, RecoverySelectedWalReadOutcome as Outcome, RecoveryWalReadSelection,
};

use super::{PhysicalRecoveryObservationAllocationDenial, PhysicalRecoveryReadAllocation};
use crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial;

mod diagnostic_backing;
mod failure;
mod listing_backing;
mod source;
mod storage;
pub use failure::{
    FundedRecoveryWalReadFailure, RecoveryWalArtifactView, RecoveryWalDiscoveryFailureView,
    RecoveryWalReadFailureView,
};

#[derive(Debug)]
pub struct FundedRecoveryWalObservations {
    // The roster and its payloads must die before their shared reservation.
    artifacts: Vec<ObservedWalArtifact>,
    backing: Option<OperationAllocationGrant>,
    names: Option<OperationAllocationGrant>,
}

impl FundedRecoveryWalObservations {
    pub fn artifacts(&self) -> &[ObservedWalArtifact] {
        &self.artifacts
    }

    /// Internal freshness comparison may reorder the funded roster, not extract it.
    pub(in crate::physical_runtime) fn artifacts_mut(&mut self) -> &mut [ObservedWalArtifact] {
        &mut self.artifacts
    }

    pub fn charged_bytes(&self) -> u64 {
        let result = self
            .backing
            .as_ref()
            .map_or(0, OperationAllocationGrant::bytes);
        result
            + self
                .names
                .as_ref()
                .map_or(0, OperationAllocationGrant::bytes)
    }
}

/// What stopped a funded WAL read: the caller's grant, whose owner states it
/// as its own limit, or what an uncharged read would have met.
pub type FundedWalReadStop<D> = GrantedReadStop<D, FundedRecoveryWalReadFailure>;

impl PhysicalRecoveryReadAllocation<'_> {
    /// Admit the result roster before reserving its storage, then each known
    /// file length before constructing or reading its buffer. The files share
    /// `grant`. The cumulative charge survives this temporary Coordination
    /// borrow with the result.
    pub fn read_wal_payloads<D: LimitDimension>(
        &mut self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        maximum_segments: NonZeroU64,
        grant: ReadGrant<D>,
    ) -> Result<FundedRecoveryWalObservations, FundedWalReadStop<D>> {
        let source = source::WalReadSource::Recovery(discovery);
        match self.read_wal_source(source, maximum_segments, grant, None)? {
            Outcome::Observed(observed) => Ok(observed),
            Outcome::Mismatch(_) => unreachable!("unconstrained reads have no selected inventory"),
        }
    }

    /// A Serving observation of `observation_bytes`, which no caller budgets.
    #[cfg(test)]
    pub(in crate::physical_runtime) fn read_serving_wal_payloads(
        &mut self,
        media: &worth_store_physical_backend::QualifiedFilesystemMedia,
        maximum_segments: NonZeroU64,
        observation_bytes: u64,
    ) -> Result<FundedRecoveryWalObservations, FundedRecoveryWalReadFailure> {
        let source = serving_source(media, maximum_segments, observation_bytes)?;
        let read = self.read_wal_source(source, maximum_segments, ReadGrant::ceiling_only(), None);
        match read.map_err(GrantedReadStop::unread)? {
            Outcome::Observed(observed) => Ok(observed),
            Outcome::Mismatch(_) => unreachable!("unconstrained reads have no selected inventory"),
        }
    }

    /// The inventory `selection` declares, in a Serving observation of
    /// `observation_bytes`: each member's declared length is its ceiling.
    pub(in crate::physical_runtime) fn read_selected_serving_wal_payloads(
        &mut self,
        media: &worth_store_physical_backend::QualifiedFilesystemMedia,
        maximum_segments: NonZeroU64,
        observation_bytes: u64,
        selection: &dyn RecoveryWalReadSelection,
    ) -> Result<Outcome<FundedRecoveryWalObservations>, FundedRecoveryWalReadFailure> {
        let source = serving_source(media, maximum_segments, observation_bytes)?;
        let grant = ReadGrant::ceiling_only();
        self.read_wal_source(source, maximum_segments, grant, Some(selection))
            .map_err(GrantedReadStop::unread)
    }

    fn read_wal_source<D: LimitDimension>(
        &self,
        mut discovery: source::WalReadSource<'_, '_>,
        maximum_segments: NonZeroU64,
        grant: ReadGrant<D>,
        selection: Option<&dyn RecoveryWalReadSelection>,
    ) -> Result<Outcome<FundedRecoveryWalObservations>, FundedWalReadStop<D>> {
        use PhysicalRecoveryObservationAllocationDenial as Denial;
        let refused =
            |cause| GrantedReadStop::Unread(FundedRecoveryWalReadFailure::inline(0, cause));
        if discovery.store_identity() != self.store_identity() {
            return Err(refused(Denial::StoreMismatch));
        }
        if !discovery.listing_is_qualified() {
            return Err(refused(Denial::UnqualifiedListingStorage));
        }
        if !discovery.path_is_qualified() {
            return Err(refused(Denial::UnqualifiedPathStorage));
        }
        let mut storage =
            storage::NativeWalReadStorage::prepare(self).map_err(GrantedReadStop::Unread)?;
        match discovery
            .read(maximum_segments, grant, &mut storage, selection)
            .granted()
        {
            Ok(Outcome::Observed(artifacts)) => {
                Ok(Outcome::Observed(storage.finish_observations(artifacts)))
            }
            Ok(Outcome::Mismatch(mismatch)) => Ok(Outcome::Mismatch(mismatch)),
            // The grant's refusal holds no artifact: every buffer has unwound.
            Err(GrantedReadStop::PastGrant(overrun)) => Err(GrantedReadStop::PastGrant(overrun)),
            Err(GrantedReadStop::Unread(failure)) => {
                Err(GrantedReadStop::Unread(storage.finish_failure(failure)))
            }
        }
    }
}

/// A Serving WAL observation of `observation_bytes`.
fn serving_source<'media>(
    media: &'media worth_store_physical_backend::QualifiedFilesystemMedia,
    maximum_segments: NonZeroU64,
    observation_bytes: u64,
) -> Result<source::WalReadSource<'static, 'media>, FundedRecoveryWalReadFailure> {
    let observation = media
        .bounded_wal_observation(maximum_segments.get(), observation_bytes)
        .map_err(|cause| {
            FundedRecoveryWalReadFailure::inline(
                0,
                PhysicalRecoveryObservationAllocationDenial::WalObservationUnavailable(cause),
            )
        })?;
    Ok(source::WalReadSource::Serving(observation))
}

struct WalObservationBacking<'window, 'coordination> {
    window: &'window PhysicalRecoveryReadAllocation<'coordination>,
    backing: Option<OperationAllocationGrant>,
    retained: u64,
}

impl WalObservationBacking<'_, '_> {
    fn allocate_payload(
        &mut self,
        length: usize,
    ) -> Result<Vec<u8>, PhysicalRecoveryObservationAllocationDenial> {
        let mut bytes = self.allocate_vector(length)?;
        bytes.resize(length, 0);
        Ok(bytes)
    }

    /// Funds vector storage only; element-owned heaps require separate admission.
    fn allocate_vector<T>(
        &mut self,
        count: usize,
    ) -> Result<Vec<T>, PhysicalRecoveryObservationAllocationDenial> {
        use PhysicalRecoveryObservationAllocationDenial as Denial;
        let overflow = || {
            Denial::Residency(PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                admitted: self.window.recovery_byte_limit(),
            })
        };
        let requested = count
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        let required = self.retained.checked_add(requested).ok_or_else(overflow)?;
        match &mut self.backing {
            Some(grant) => grant
                .try_resize(required)
                .map_err(|denial| Denial::Residency(self.window.map_allocation_denial(denial)))?,
            None => {
                self.backing = self
                    .window
                    .reserve_owned(required)
                    .map_err(Denial::Residency)?
            }
        }
        let mut vector = Vec::new();
        vector.try_reserve_exact(count).map_err(|cause| {
            Denial::Residency(PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause })
        })?;
        let actual = vector
            .capacity()
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        if actual != requested {
            return Err(Denial::AllocatorExceededReservation { requested, actual });
        }
        self.retained = required;
        Ok(vector)
    }
}

#[cfg(test)]
mod tests;

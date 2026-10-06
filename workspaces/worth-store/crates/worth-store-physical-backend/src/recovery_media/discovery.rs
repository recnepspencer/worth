use super::refusal::ArtifactDamage;
use super::{AdmittedRecoveryFilesystemMedia, RecoveryFilesystemQualificationError};
use crate::filesystem_media::{ArtifactTreeDirectory, ArtifactTreeFailure};

mod addressed_payload;
mod artifact;
mod borrowed_record;
mod borrowed_wal;
mod charged_read;
#[cfg(test)]
mod count_tests;
mod media_backing;
mod observation_budget;
mod observed_artifact;
mod observed_wal;
mod resident_read;
mod resident_record;
mod selected_wal;
mod wal_artifacts;
mod wal_storage;
pub(crate) use artifact::record_artifact;
pub use artifact::RecoveryDiscoveryArtifact;
pub use borrowed_record::BorrowedRecordFilesystemObservation;
pub use borrowed_wal::BorrowedWalFilesystemObservation;
use media_backing::DiscoveryMediaBacking;
#[cfg(feature = "test-support")]
pub use observation_budget::filesystem_observation_limit_for_test;
use observation_budget::FilesystemObservationAllowance;
pub use observation_budget::{ExceededFilesystemObservationBound, FilesystemObservationBound};
pub use observed_artifact::ObservedRecoveryArtifact;
pub use observed_wal::{ObservedWalArtifact, RecoveryWalObservationIdentity};
pub use resident_read::RecoveryDiscoveryAllocationFailure;
pub use selected_wal::{
    RecoverySelectedWalReadOutcome, RecoveryWalReadSelection, RecoveryWalSelectionMismatch,
};
pub use wal_artifacts::WalInventoryOutcome;
pub use wal_storage::{RecoveryWalListingAllocationMode, RecoveryWalReadStorage};

pub type BoundedRecoveryFilesystemDiscovery =
    FilesystemObservation<crate::filesystem_media::recovery_qualification::AdmittedRecoveryParts>;

/// Sealed backing mechanics stay below the concrete owned and borrowed facades.
pub struct FilesystemObservation<M> {
    parts: M,
    remaining_entries: u64,
    maximum_entries: u64,
    remaining_bytes: u64,
    maximum_bytes: u64,
    discovery_incarnation: u64,
    wal_observations_issued: u64,
    counters: RecoveryDiscoveryCounters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecoveryDiscoveryCounters {
    pub fixed_slots_read: u64,
    pub addressed_artifacts_read: u64,
    pub directory_entries_observed: u64,
    pub bytes_read: u64,
    pub wal_bytes_read: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDiscoveryFailure {
    /// A bound this observation was admitted ran out. Nothing about the
    /// media follows from it.
    Limit(ExceededFilesystemObservationBound),
    /// What no bound of this observation can fix.
    Damage(ArtifactDamage),
}

/// The count that went past every count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryDiscoveryCount {
    /// The reads one observation issued, with the next.
    Reads,
    /// The bytes one observation spent, with the next read.
    ObservationBytes,
    /// The bytes every read returned, counted for the caller, or with the next
    /// read's length against the grant it spends.
    BytesRead,
    /// The WAL bytes one inventory read: counted for the caller, or with the
    /// next file's length against the inventory's grant.
    WalBytesRead,
    /// The WAL observations one observation issued, with the next.
    WalObservations,
}

impl BoundedRecoveryFilesystemDiscovery {
    pub const fn store_identity(
        &self,
    ) -> worth_store_physical_format::store_namespace::StableStoreIdentity {
        self.parts.store_identity
    }

    pub(crate) fn new(
        parts: crate::filesystem_media::recovery_qualification::AdmittedRecoveryParts,
        discovery_incarnation: u64,
        maximum_entries: u64,
        maximum_bytes: u64,
    ) -> Result<Self, RecoveryFilesystemQualificationError> {
        if maximum_entries == 0 {
            return Err(RecoveryFilesystemQualificationError::InvalidDiscoveryLimit);
        }
        Ok(Self {
            parts,
            remaining_entries: maximum_entries,
            maximum_entries,
            remaining_bytes: maximum_bytes,
            maximum_bytes,
            discovery_incarnation,
            wal_observations_issued: 0,
            counters: RecoveryDiscoveryCounters::default(),
        })
    }

    pub const fn counters(&self) -> RecoveryDiscoveryCounters {
        self.counters
    }

    pub fn finish(self) -> AdmittedRecoveryFilesystemMedia {
        AdmittedRecoveryFilesystemMedia::from_discovery(self.parts, self.discovery_incarnation)
    }
}

impl<M> FilesystemObservation<M> {
    /// The next read fits the reads this observation was admitted.
    fn admit_read(&self) -> Result<(), RecoveryDiscoveryFailure> {
        let next = (self.maximum_entries - self.remaining_entries)
            .checked_add(1)
            .ok_or(RecoveryDiscoveryFailure::overflow(
                RecoveryDiscoveryCount::Reads,
            ))?;
        FilesystemObservationAllowance::reads(self.maximum_entries)
            .admit(next)
            .map(drop)
            .map_err(RecoveryDiscoveryFailure::Limit)
    }

    /// Spends the `length` bytes a whole read returned from the bytes this
    /// observation was admitted.
    fn spend_read_bytes(&mut self, length: u64) -> Result<(), RecoveryDiscoveryFailure> {
        self.counters.bytes_read = self.counters.bytes_read.checked_add(length).ok_or(
            RecoveryDiscoveryFailure::overflow(RecoveryDiscoveryCount::BytesRead),
        )?;
        let spent = self.spent_with(length)?;
        self.remaining_bytes = self.maximum_bytes - spent;
        Ok(())
    }

    /// What this observation will have spent after `length` more bytes,
    /// within the bytes it was admitted.
    fn spent_with(&self, length: u64) -> Result<u64, RecoveryDiscoveryFailure> {
        let spent = (self.maximum_bytes - self.remaining_bytes)
            .checked_add(length)
            .ok_or(RecoveryDiscoveryFailure::overflow(
                RecoveryDiscoveryCount::ObservationBytes,
            ))?;
        FilesystemObservationAllowance::observation_bytes(self.maximum_bytes)
            .admit(spent)
            .map_err(RecoveryDiscoveryFailure::Limit)
    }
}

impl RecoveryDiscoveryFailure {
    pub(crate) fn invalid(artifact: RecoveryDiscoveryArtifact) -> Self {
        Self::Damage(ArtifactDamage::InvalidAddress { artifact })
    }

    pub(crate) fn media(artifact: RecoveryDiscoveryArtifact, failure: ArtifactTreeFailure) -> Self {
        Self::Damage(ArtifactDamage::Media { artifact, failure })
    }

    pub(crate) fn overflow(count: RecoveryDiscoveryCount) -> Self {
        Self::Damage(ArtifactDamage::CountOverflow(count))
    }
}

use super::refusal::ArtifactDamage;
use super::{AdmittedRecoveryFilesystemMedia, RecoveryFilesystemQualificationError};
use crate::filesystem_media::{
    ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile,
};

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
    /// One byte past a read ceiling, where the tree could not tell a length.
    ReadLength,
    /// The bytes one observation spent, with the next read.
    ObservationBytes,
    /// The bytes every read returned, counted for the caller.
    BytesRead,
    /// The WAL bytes one inventory read, counted for the caller or held
    /// beside the next read.
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

    pub fn read_current_checkpoint(
        &mut self,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        let context = RecoveryDiscoveryArtifact::CurrentCheckpoint;
        let artifact = ArtifactTreeDirectory::families()
            .file("checkpoint.current")
            .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        self.read_artifact(artifact, context, byte_limit, false)
    }

    pub const fn counters(&self) -> RecoveryDiscoveryCounters {
        self.counters
    }

    pub fn finish(self) -> AdmittedRecoveryFilesystemMedia {
        AdmittedRecoveryFilesystemMedia::from_discovery(self.parts, self.discovery_incarnation)
    }

    fn read_artifact(
        &mut self,
        artifact: ArtifactTreeFile,
        context: RecoveryDiscoveryArtifact,
        byte_limit: u64,
        fixed: bool,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        let effective_byte_limit = byte_limit.min(self.remaining_bytes);
        self.admit_read()?;
        self.remaining_entries -= 1;
        match self
            .parts
            .artifact_tree()
            .read_bounded(&artifact, effective_byte_limit)
        {
            Ok(bytes) => {
                self.spend_read_bytes(bytes.len() as u64)?;
                if !fixed {
                    self.counters.addressed_artifacts_read += 1;
                }
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity,
                    context,
                    0,
                    Some(bytes),
                ))
            }
            Err(failure) if failure.kind() == ArtifactTreeFailureKind::Absent => Ok(
                ObservedRecoveryArtifact::new(self.parts.store_identity, context, 0, None),
            ),
            Err(failure) => Err(self
                .whole_read_refused(&failure, byte_limit, effective_byte_limit)
                .unwrap_or_else(|| RecoveryDiscoveryFailure::media(context, failure))),
        }
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

    /// A read of `length` bytes refused within `byte_limit` and the bytes
    /// this reader has left: a whole artifact the tree would not read, or a
    /// range its caller asked for. One longer than `byte_limit` passed the
    /// ceiling its caller named for it, however little this reader has left;
    /// only one within that ceiling ran this reader out of bytes. Where the
    /// tree cannot tell a whole artifact's length it reports one byte past
    /// what it was asked for, which decides the same way. A length within
    /// both passes: no bound of this reader refused it.
    fn read_refused(&self, byte_limit: u64, length: u64) -> Result<(), RecoveryDiscoveryFailure> {
        FilesystemObservationAllowance::requested_bytes(byte_limit)
            .admit(length)
            .map_err(RecoveryDiscoveryFailure::Limit)?;
        self.spent_with(length).map(drop)
    }

    /// What a whole read the tree refused says about this observation: the
    /// limit one of its allowances refuses for the length the tree reported,
    /// read within `effective_byte_limit`. Every other failure is the tree's.
    fn whole_read_refused(
        &self,
        failure: &ArtifactTreeFailure,
        byte_limit: u64,
        effective_byte_limit: u64,
    ) -> Option<RecoveryDiscoveryFailure> {
        if failure.kind() != ArtifactTreeFailureKind::AccessLimitExceeded {
            return None;
        }
        let length = match failure.access_limit() {
            Some(limit) => limit.observed,
            None => match effective_byte_limit.checked_add(1) {
                Some(past) => past,
                None => {
                    return Some(RecoveryDiscoveryFailure::overflow(
                        RecoveryDiscoveryCount::ReadLength,
                    ))
                }
            },
        };
        self.read_refused(byte_limit, length).err()
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

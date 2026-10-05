use super::{AdmittedRecoveryFilesystemMedia, RecoveryFilesystemQualificationError};
use crate::filesystem_media::{
    ArtifactTreeDirectory, ArtifactTreeFailure, ArtifactTreeFailureKind, ArtifactTreeFile,
};
use worth_store_physical_format::RecordArtifactFile;

mod addressed_payload;
mod addressed_range;
mod artifact;
mod borrowed_record;
mod borrowed_wal;
mod media_backing;
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
    EntryLimitExceeded {
        observed: u64,
        admitted: u64,
    },
    ByteLimitExceeded {
        observed: u64,
        admitted: u64,
        scope: RecoveryDiscoveryByteLimitScope,
    },
    Media {
        artifact: RecoveryDiscoveryArtifact,
        failure: ArtifactTreeFailure,
    },
    InvalidAddress {
        artifact: RecoveryDiscoveryArtifact,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryDiscoveryByteLimitScope {
    Observation,
    Requested,
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
        if maximum_entries == 0 || maximum_bytes == 0 {
            return Err(RecoveryFilesystemQualificationError::InvalidDiscoveryLimit);
        }
        Ok(Self {
            parts,
            remaining_entries: maximum_entries,
            remaining_bytes: maximum_bytes,
            maximum_bytes,
            discovery_incarnation,
            wal_observations_issued: 0,
            counters: RecoveryDiscoveryCounters::default(),
        })
    }

    pub fn read_current_selector(
        &mut self,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_fixed(RecordArtifactFile::CurrentRootSelector, byte_limit)
    }

    pub fn read_bootstrap_catalog(
        &mut self,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_fixed(RecordArtifactFile::BootstrapCatalog, byte_limit)
    }

    pub fn read_previous_selector(
        &mut self,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_fixed(RecordArtifactFile::PreviousRootSelector, byte_limit)
    }

    pub fn read_root_manifest(
        &mut self,
        generation: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(RecordArtifactFile::RootManifest { generation }, byte_limit)
    }

    pub fn read_root_routing_block(
        &mut self,
        generation: u64,
        block: u64,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        self.read_addressed(
            RecordArtifactFile::RootRoutingBlock { generation, block },
            byte_limit,
        )
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

    fn read_fixed(
        &mut self,
        artifact: RecordArtifactFile,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        let context = RecoveryDiscoveryArtifact::Record(artifact);
        let artifact = record_artifact(artifact)?;
        let result = self.read_artifact(artifact, context, byte_limit, true)?;
        self.counters.fixed_slots_read += 1;
        Ok(result)
    }

    fn read_addressed(
        &mut self,
        artifact: RecordArtifactFile,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        let context = RecoveryDiscoveryArtifact::Record(artifact);
        let artifact = record_artifact(artifact)?;
        self.read_artifact(artifact, context, byte_limit, false)
    }

    fn read_artifact(
        &mut self,
        artifact: ArtifactTreeFile,
        context: RecoveryDiscoveryArtifact,
        byte_limit: u64,
        fixed: bool,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        let effective_byte_limit = byte_limit.min(self.remaining_bytes);
        if self.remaining_entries == 0 {
            return Err(RecoveryDiscoveryFailure::EntryLimitExceeded {
                observed: 1,
                admitted: 0,
            });
        }
        self.remaining_entries -= 1;
        match self
            .parts
            .artifact_tree()
            .read_bounded(&artifact, effective_byte_limit)
        {
            Ok(bytes) => {
                self.counters.bytes_read = self
                    .counters
                    .bytes_read
                    .checked_add(bytes.len() as u64)
                    .ok_or(RecoveryDiscoveryFailure::ByteLimitExceeded {
                        observed: u64::MAX,
                        admitted: self.maximum_bytes,
                        scope: RecoveryDiscoveryByteLimitScope::Observation,
                    })?;
                self.remaining_bytes = self.remaining_bytes.checked_sub(bytes.len() as u64).ok_or(
                    RecoveryDiscoveryFailure::ByteLimitExceeded {
                        observed: bytes.len() as u64,
                        admitted: self.maximum_bytes,
                        scope: RecoveryDiscoveryByteLimitScope::Observation,
                    },
                )?;
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
            Err(failure) if failure.kind() == ArtifactTreeFailureKind::AccessLimitExceeded => {
                let limit = failure.access_limit().unwrap_or(
                    crate::filesystem_media::ArtifactTreeAccessLimit {
                        observed: effective_byte_limit.saturating_add(1),
                        admitted: effective_byte_limit,
                    },
                );
                Err(self.read_refused(byte_limit, limit.observed))
            }
            Err(failure) => Err(RecoveryDiscoveryFailure::Media {
                artifact: context,
                failure,
            }),
        }
    }
}

fn map_media(
    failure: ArtifactTreeFailure,
    artifact: RecoveryDiscoveryArtifact,
) -> RecoveryDiscoveryFailure {
    if failure.kind() == ArtifactTreeFailureKind::AccessLimitExceeded {
        let limit =
            failure
                .access_limit()
                .unwrap_or(crate::filesystem_media::ArtifactTreeAccessLimit {
                    observed: 1,
                    admitted: 0,
                });
        RecoveryDiscoveryFailure::EntryLimitExceeded {
            observed: limit.observed,
            admitted: limit.admitted,
        }
    } else {
        RecoveryDiscoveryFailure::Media { artifact, failure }
    }
}

impl<M> FilesystemObservation<M> {
    /// A read of `length` bytes refused within `byte_limit` and the bytes
    /// this reader has left: a whole artifact the tree would not read, or a
    /// range its caller asked for. One longer than `byte_limit` passed the
    /// ceiling its caller named for it, however little this reader has left;
    /// only one within that ceiling ran this reader out of bytes. Where the
    /// tree cannot tell a whole artifact's length it reports one byte past
    /// what it was asked for, which decides the same way.
    fn read_refused(&self, byte_limit: u64, length: u64) -> RecoveryDiscoveryFailure {
        if length > byte_limit {
            RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed: length,
                admitted: byte_limit,
                scope: RecoveryDiscoveryByteLimitScope::Requested,
            }
        } else {
            RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed: self.counters.bytes_read.saturating_add(length),
                admitted: self.maximum_bytes,
                scope: RecoveryDiscoveryByteLimitScope::Observation,
            }
        }
    }

    /// Admits a range of `length` bytes before anything is opened. An empty
    /// range addresses nothing.
    fn admit_range(
        &self,
        artifact: &RecoveryDiscoveryArtifact,
        length: u64,
        byte_limit: u64,
    ) -> Result<(), RecoveryDiscoveryFailure> {
        if length == 0 {
            Err(RecoveryDiscoveryFailure::invalid(artifact.clone()))
        } else if length > self.remaining_bytes.min(byte_limit) {
            Err(self.read_refused(byte_limit, length))
        } else {
            Ok(())
        }
    }
}

impl RecoveryDiscoveryFailure {
    fn invalid(artifact: RecoveryDiscoveryArtifact) -> Self {
        Self::InvalidAddress { artifact }
    }
}

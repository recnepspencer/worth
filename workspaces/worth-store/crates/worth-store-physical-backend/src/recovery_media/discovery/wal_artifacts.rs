use std::ffi::{OsStr, OsString};

use worth_store_physical_format::store_namespace::{NamespaceEntryType, StableStoreIdentity};

use super::{
    map_media, ArtifactTreeDirectory, BoundedRecoveryFilesystemDiscovery,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedWalArtifact {
    store: StableStoreIdentity,
    observation: RecoveryWalObservationIdentity,
    name: OsString,
    entry_type: NamespaceEntryType,
    bytes: Option<Vec<u8>>,
}

/// Opaque identity of one C.4 WAL read within one admitted recovery-media generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecoveryWalObservationIdentity {
    media_generation: super::super::PhysicalRecoveryMediaGeneration,
    discovery_incarnation: u64,
    sequence: u64,
}

impl BoundedRecoveryFilesystemDiscovery {
    pub fn read_wal_artifacts(
        &mut self,
        maximum_segments: u64,
        byte_limit: u64,
    ) -> Result<Vec<ObservedWalArtifact>, RecoveryDiscoveryFailure> {
        let maximum_segments = admitted_segment_limit(maximum_segments)?;
        let directory_context = RecoveryDiscoveryArtifact::WalDirectory;
        let directory = ArtifactTreeDirectory::families()
            .child("wal")
            .map_err(|_| RecoveryDiscoveryFailure::invalid(directory_context.clone()))?;
        if !self
            .parts
            .artifact_tree()
            .directory_exists(&directory)
            .map_err(|failure| map_media(failure, directory_context.clone()))?
        {
            return Ok(Vec::new());
        }
        let entries = self
            .parts
            .artifact_tree()
            .list_bounded(&directory, maximum_segments)
            .map_err(|failure| map_media(failure, directory_context))?;
        self.counters.directory_entries_observed += entries.len() as u64;
        let mut observed = Vec::with_capacity(entries.len());
        let mut remaining_wal_bytes = byte_limit;
        for entry in entries {
            let name = entry.name().to_owned();
            let context = RecoveryDiscoveryArtifact::WalArtifact(name.clone());
            let bytes = if entry.entry_type() == NamespaceEntryType::RegularFile {
                let file = directory
                    .file(
                        name.to_str()
                            .ok_or_else(|| RecoveryDiscoveryFailure::invalid(context.clone()))?,
                    )
                    .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
                let bytes =
                    self.read_wal_artifact(file, context, remaining_wal_bytes, byte_limit)?;
                remaining_wal_bytes = remaining_wal_bytes.checked_sub(byte_count(&bytes)).ok_or(
                    RecoveryDiscoveryFailure::ByteLimitExceeded {
                        observed: byte_limit,
                        admitted: remaining_wal_bytes,
                        scope: RecoveryDiscoveryByteLimitScope::Requested,
                    },
                )?;
                self.counters.wal_bytes_read = self
                    .counters
                    .wal_bytes_read
                    .checked_add(byte_count(&bytes))
                    .ok_or(RecoveryDiscoveryFailure::ByteLimitExceeded {
                        observed: u64::MAX,
                        admitted: byte_limit,
                        scope: RecoveryDiscoveryByteLimitScope::Requested,
                    })?;
                bytes
            } else {
                None
            };
            observed.push(ObservedWalArtifact {
                store: self.parts.store_identity,
                observation: self.issue_wal_observation_identity()?,
                name,
                entry_type: entry.entry_type(),
                bytes,
            });
        }
        Ok(observed)
    }

    fn read_wal_artifact(
        &mut self,
        file: crate::filesystem_media::ArtifactTreeFile,
        context: RecoveryDiscoveryArtifact,
        remaining_wal_bytes: u64,
        byte_limit: u64,
    ) -> Result<Option<Vec<u8>>, RecoveryDiscoveryFailure> {
        match self.read_artifact(file, context, remaining_wal_bytes, false) {
            Ok(artifact) => Ok(artifact.into_bytes()),
            Err(RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed,
                admitted: _,
                scope: RecoveryDiscoveryByteLimitScope::Requested,
            }) => Err(RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed: byte_limit
                    .saturating_sub(remaining_wal_bytes)
                    .saturating_add(observed),
                admitted: byte_limit,
                scope: RecoveryDiscoveryByteLimitScope::Requested,
            }),
            Err(failure) => Err(failure),
        }
    }

    fn issue_wal_observation_identity(
        &mut self,
    ) -> Result<RecoveryWalObservationIdentity, RecoveryDiscoveryFailure> {
        self.wal_observations_issued = self.wal_observations_issued.checked_add(1).ok_or(
            RecoveryDiscoveryFailure::EntryLimitExceeded {
                observed: u64::MAX,
                admitted: u64::MAX - 1,
            },
        )?;
        Ok(RecoveryWalObservationIdentity {
            media_generation: self.parts.media_generation,
            discovery_incarnation: self.discovery_incarnation,
            sequence: self.wal_observations_issued,
        })
    }
}

impl ObservedWalArtifact {
    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }

    pub const fn observation_identity(&self) -> RecoveryWalObservationIdentity {
        self.observation
    }

    pub fn matches_media_generation(
        &self,
        generation: super::super::PhysicalRecoveryMediaGeneration,
    ) -> bool {
        self.observation.media_generation == generation
    }

    pub fn name(&self) -> &OsStr {
        &self.name
    }

    pub const fn entry_type(&self) -> NamespaceEntryType {
        self.entry_type
    }

    pub fn bytes(&self) -> Option<&[u8]> {
        self.bytes.as_deref()
    }
}

fn admitted_segment_limit(maximum_segments: u64) -> Result<usize, RecoveryDiscoveryFailure> {
    let admitted = usize::try_from(maximum_segments).map_err(|_| {
        RecoveryDiscoveryFailure::EntryLimitExceeded {
            observed: maximum_segments,
            admitted: usize::MAX as u64,
        }
    })?;
    if admitted == 0 {
        return Err(RecoveryDiscoveryFailure::EntryLimitExceeded {
            observed: 0,
            admitted: 0,
        });
    }
    Ok(admitted)
}

fn byte_count(bytes: &Option<Vec<u8>>) -> u64 {
    bytes.as_ref().map_or(0, |bytes| bytes.len() as u64)
}

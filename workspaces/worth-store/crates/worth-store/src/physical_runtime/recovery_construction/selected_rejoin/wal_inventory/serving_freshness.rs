//! Fresh Serving bytes are read through the same native-backed C4 WAL engine.

use super::{
    SelectedWalMediaFingerprint, WalSegmentArtifactIdentity, MAX_WAL_BYTES, MAX_WAL_SEGMENTS,
};
use crate::physical_runtime::{
    FundedRecoveryWalObservations, FundedRecoveryWalReadFailure, PhysicalRecoveryReadAllocation,
};
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use worth_store_physical_backend::{
    ArtifactTreeDirectoryEntry, QualifiedFilesystemMedia, RecoverySelectedWalReadOutcome,
    RecoveryWalReadSelection,
};
use worth_store_physical_format::store_namespace::NamespaceEntryType;

impl SelectedWalMediaFingerprint {
    pub(in crate::physical_runtime) fn matches_serving_media(
        &self,
        media: &QualifiedFilesystemMedia,
        window: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<bool, FundedRecoveryWalReadFailure> {
        match window.read_selected_serving_wal_payloads(
            media,
            MAX_WAL_SEGMENTS,
            MAX_WAL_BYTES,
            self,
        )? {
            RecoverySelectedWalReadOutcome::Observed(mut observed) => {
                Ok(self.matches_observations(&mut observed))
            }
            RecoverySelectedWalReadOutcome::Mismatch(_) => Ok(false),
        }
    }

    fn matches_observations(&self, observed: &mut FundedRecoveryWalObservations) -> bool {
        let artifacts = observed.artifacts_mut();
        if artifacts.len() != self.artifacts.len() {
            return false;
        }
        // Enumeration order is not authority. Reorder the already-funded roster
        // in place, retaining the existing exact duplicate-name rejection.
        artifacts.sort_unstable_by(|left, right| left.name().cmp(right.name()));
        if artifacts
            .windows(2)
            .any(|pair| pair[0].name() == pair[1].name())
        {
            return false;
        }
        let mut remaining = MAX_WAL_BYTES;
        for artifact in artifacts {
            if artifact.entry_type() != NamespaceEntryType::RegularFile {
                return false;
            }
            let Some(name) = artifact.name().to_str() else {
                return false;
            };
            let Some(identity) = WalSegmentArtifactIdentity::parse(name) else {
                return false;
            };
            let Ok(index) = self
                .artifacts
                .binary_search_by_key(&identity, |item| item.identity)
            else {
                return false;
            };
            let fingerprint = self.artifacts[index];
            let Some(next_remaining) = remaining.checked_sub(fingerprint.length) else {
                return false;
            };
            let Some(bytes) = artifact.bytes() else {
                return false;
            };
            if bytes.len() as u64 != fingerprint.length
                || <[u8; 32]>::from(Sha256::digest(bytes)) != fingerprint.sha256
            {
                return false;
            }
            remaining = next_remaining;
        }
        true
    }
}

impl RecoveryWalReadSelection for SelectedWalMediaFingerprint {
    fn matches_listing(&self, entries: &mut [ArtifactTreeDirectoryEntry]) -> bool {
        if entries.len() != self.artifacts.len() {
            return false;
        }
        if entries.iter().any(|entry| {
            entry.entry_type() != NamespaceEntryType::RegularFile
                || selected_identity(entry.name()).is_none()
        }) {
            return false;
        }
        // Sort the existing, funded listing only. This rejects duplicate typed
        // identities even when different filename spellings parse to one key.
        entries.sort_unstable_by_key(|entry| selected_identity(entry.name()));
        if entries
            .windows(2)
            .any(|pair| selected_identity(pair[0].name()) == selected_identity(pair[1].name()))
        {
            return false;
        }
        entries
            .iter()
            .try_fold(MAX_WAL_BYTES, |remaining, entry| {
                remaining.checked_sub(self.expected_file_length(entry.name())?)
            })
            .is_some()
    }

    fn expected_file_length(&self, name: &OsStr) -> Option<u64> {
        let identity = selected_identity(name)?;
        let index = self
            .artifacts
            .binary_search_by_key(&identity, |item| item.identity)
            .ok()?;
        Some(self.artifacts[index].length)
    }
}

fn selected_identity(name: &OsStr) -> Option<WalSegmentArtifactIdentity> {
    WalSegmentArtifactIdentity::parse(name.to_str()?)
}

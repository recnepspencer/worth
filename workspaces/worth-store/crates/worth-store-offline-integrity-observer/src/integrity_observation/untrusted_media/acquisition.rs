use std::collections::btree_map::Entry;
use std::fs::Metadata;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::{
    BoundedAcquisition, BoundedMediaWalk, CachedPhysicalFile, UnrecognizedEntryClassification,
};
use crate::integrity_observation::file_identity::{
    identity_from_file, open_observed_file, same_snapshot, PhysicalFileIdentity,
};
use crate::integrity_observation::{
    OfflineIndeterminatePhysicalReason, OfflineIntegrityOutcome,
    OfflineIntegrityReportCompleteness, OfflineUnknownPhysicalReason,
};

impl BoundedMediaWalk {
    pub(crate) fn acquire(
        &mut self,
        path: &Path,
        depth: u32,
    ) -> Result<BoundedAcquisition, OfflineIntegrityOutcome> {
        self.acquire_with_after_read(path, depth, || {})
    }

    pub(super) fn acquire_with_after_read(
        &mut self,
        path: &Path,
        depth: u32,
        after_read: impl FnOnce(),
    ) -> Result<BoundedAcquisition, OfflineIntegrityOutcome> {
        self.acquire_with_hooks(path, depth, || {}, after_read)
    }

    #[cfg(test)]
    pub(super) fn acquire_with_before_open(
        &mut self,
        path: &Path,
        depth: u32,
        before_open: impl FnOnce(),
    ) -> Result<BoundedAcquisition, OfflineIntegrityOutcome> {
        self.acquire_with_hooks(path, depth, before_open, || {})
    }

    fn acquire_with_hooks(
        &mut self,
        path: &Path,
        depth: u32,
        before_open: impl FnOnce(),
        after_read: impl FnOnce(),
    ) -> Result<BoundedAcquisition, OfflineIntegrityOutcome> {
        self.admit_acquisition_path(path, depth)?;
        let length = std::fs::metadata(path)
            .map_err(|_| self.indeterminate_io())?
            .len();
        if length > self.remaining_byte_budget() && !self.cached_lengths.contains(&length) {
            return Err(self.bound(OfflineIndeterminatePhysicalReason::ByteBoundExceeded));
        }
        before_open();
        let containment_guards = self.open_containment_guards(path)?;
        let mut file = self.open_identity_bound_file(path, containment_guards.len())?;
        let before = file.metadata().map_err(|_| self.indeterminate_io())?;
        if !before.is_file() {
            return Err(self.source_changed());
        }
        let physical_identity = self.identity_from_open_file(&file, path)?;
        self.verify_path_binding(&file, path, &physical_identity)?;
        let (physical_alias_of, cached_bytes) = self.cached_alias(&physical_identity, path);
        if let Some(bytes) = cached_bytes {
            let unchanged = self
                .seen_files
                .get(&physical_identity)
                .and_then(|cached| cached.snapshot.as_ref())
                .is_some_and(|snapshot| same_snapshot(snapshot, &before));
            if !unchanged {
                return Err(self.source_changed());
            }
            return Ok(BoundedAcquisition {
                byte_length: bytes.len(),
                bytes,
                physical_alias_of,
            });
        }
        let admitted = self.remaining_byte_budget();
        if before.len() > admitted {
            return Err(self.bound(OfflineIndeterminatePhysicalReason::ByteBoundExceeded));
        }
        let bytes = self.read_bounded_file(&mut file, &before, admitted)?;
        after_read();
        self.verify_stable_file(&file, path, &before, &physical_identity, bytes.len())?;
        let bytes: Arc<[u8]> = bytes.into();
        self.cache_bytes(physical_identity.clone(), path, Arc::clone(&bytes), before);
        Ok(BoundedAcquisition {
            byte_length: bytes.len(),
            bytes,
            physical_alias_of,
        })
    }

    pub(super) fn classify_unrecognized_entry(
        &mut self,
        path: &Path,
    ) -> UnrecognizedEntryClassification {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(_) => {
                return self.unrecognized_outcome(OfflineIntegrityOutcome::Unknown(
                    OfflineUnknownPhysicalReason::FilesystemEntryUnavailable,
                ));
            }
        };
        if metadata.is_dir() {
            return self.unrecognized_outcome(OfflineIntegrityOutcome::Unknown(
                OfflineUnknownPhysicalReason::UnrecognizedDirectory,
            ));
        }
        if !metadata.is_file() {
            return self.unrecognized_outcome(OfflineIntegrityOutcome::Unknown(
                OfflineUnknownPhysicalReason::UnrecognizedOtherEntry,
            ));
        }
        self.classify_unrecognized_file(path)
    }

    fn classify_unrecognized_file(&mut self, path: &Path) -> UnrecognizedEntryClassification {
        self.classify_unrecognized_file_with_hook(path, || {})
    }

    #[cfg(test)]
    pub(super) fn classify_unrecognized_with_before_open(
        &mut self,
        path: &Path,
        before_open: impl FnOnce(),
    ) -> UnrecognizedEntryClassification {
        if !path
            .symlink_metadata()
            .is_ok_and(|metadata| metadata.is_file())
        {
            return self.unrecognized_outcome(OfflineIntegrityOutcome::Unknown(
                OfflineUnknownPhysicalReason::FilesystemEntryUnavailable,
            ));
        }
        self.classify_unrecognized_file_with_hook(path, before_open)
    }

    fn classify_unrecognized_file_with_hook(
        &mut self,
        path: &Path,
        before_open: impl FnOnce(),
    ) -> UnrecognizedEntryClassification {
        before_open();
        let containment_guards = match self.open_containment_guards(path) {
            Ok(guards) => guards,
            Err(outcome) => return self.unrecognized_outcome(outcome),
        };
        let file = match self.open_identity_bound_file(path, containment_guards.len()) {
            Ok(file) => file,
            Err(outcome) => return self.unrecognized_outcome(outcome),
        };
        if !file.metadata().is_ok_and(|metadata| metadata.is_file()) {
            let outcome = self.source_changed();
            return self.unrecognized_outcome(outcome);
        }
        let identity = match self.identity_from_open_file(&file, path) {
            Ok(identity) => identity,
            Err(outcome) => return self.unrecognized_outcome(outcome),
        };
        if let Err(outcome) = self.verify_path_binding(&file, path, &identity) {
            return self.unrecognized_outcome(outcome);
        }
        let alias = self.register_identity_without_bytes(identity, path);
        let outcome =
            OfflineIntegrityOutcome::Unknown(OfflineUnknownPhysicalReason::UnrecognizedFile);
        self.record_outcome(&outcome);
        UnrecognizedEntryClassification {
            outcome,
            physical_alias_of: alias,
        }
    }

    fn admit_acquisition_path(
        &mut self,
        path: &Path,
        depth: u32,
    ) -> Result<(), OfflineIntegrityOutcome> {
        if let Some(reason) = self.depth_exhaustion(depth) {
            return Err(self.bound(reason));
        }
        if let Some(reason) = self.elapsed_exhaustion() {
            return Err(self.bound(reason));
        }
        if let Some(reason) = self
            .refused_path_reason(path)
            .map_err(|_| self.indeterminate_io())?
        {
            return Err(OfflineIntegrityOutcome::Indeterminate(reason));
        }
        Ok(())
    }

    fn open_identity_bound_file(
        &mut self,
        path: &Path,
        held_handles: usize,
    ) -> Result<std::fs::File, OfflineIntegrityOutcome> {
        if held_handles.saturating_add(1) > self.limits.maximum_open_files() as usize {
            return Err(self.bound(OfflineIndeterminatePhysicalReason::OpenFileBoundExceeded));
        }
        let file = open_observed_file(path).map_err(|_| {
            if self.path_is_plain_contained_file(path) {
                self.indeterminate_io()
            } else {
                self.source_changed()
            }
        })?;
        self.counters.files_opened += 1;
        self.counters.open_file_high_water = self
            .counters
            .open_file_high_water
            .max(held_handles.saturating_add(1) as u32);
        Ok(file)
    }

    fn identity_from_open_file(
        &mut self,
        file: &std::fs::File,
        path: &Path,
    ) -> Result<PhysicalFileIdentity, OfflineIntegrityOutcome> {
        let maximum_output_bytes = self.limits.maximum_bytes();
        let maximum_elapsed = self.remaining_elapsed_budget();
        identity_from_file(file, path, maximum_output_bytes, maximum_elapsed)
            .map_err(|_| self.identity_unavailable())
    }

    fn cached_alias(
        &mut self,
        identity: &PhysicalFileIdentity,
        path: &Path,
    ) -> (Option<PathBuf>, Option<Arc<[u8]>>) {
        let Some(cached) = self.seen_files.get(identity) else {
            return (None, None);
        };
        if cached.first_path == path {
            return (None, cached.bytes.as_ref().map(Arc::clone));
        }
        self.counters.duplicate_identities += 1;
        (
            Some(cached.first_path.clone()),
            cached.bytes.as_ref().map(Arc::clone),
        )
    }

    fn remaining_byte_budget(&self) -> u64 {
        self.limits
            .maximum_bytes()
            .saturating_sub(self.counters.bytes_read)
    }

    fn read_bounded_file(
        &mut self,
        file: &mut std::fs::File,
        metadata: &Metadata,
        admitted: u64,
    ) -> Result<Vec<u8>, OfflineIntegrityOutcome> {
        let mut bytes = Vec::new();
        let result = file
            .by_ref()
            .take(metadata.len().saturating_add(1).min(admitted))
            .read_to_end(&mut bytes);
        self.counters.bytes_read = self.counters.bytes_read.saturating_add(bytes.len() as u64);
        result.map_err(|_| self.indeterminate_io())?;
        Ok(bytes)
    }

    fn verify_stable_file(
        &mut self,
        file: &std::fs::File,
        path: &Path,
        before: &Metadata,
        expected_identity: &PhysicalFileIdentity,
        bytes_read: usize,
    ) -> Result<(), OfflineIntegrityOutcome> {
        if let Some(reason) = self.elapsed_exhaustion() {
            return Err(self.bound(reason));
        }
        let after = file.metadata().map_err(|_| self.indeterminate_io())?;
        self.verify_path_binding(file, path, expected_identity)?;
        if bytes_read as u64 != before.len() || !same_snapshot(before, &after) {
            return Err(self.source_changed());
        }
        Ok(())
    }

    fn cache_bytes(
        &mut self,
        identity: PhysicalFileIdentity,
        path: &Path,
        bytes: Arc<[u8]>,
        snapshot: Metadata,
    ) {
        self.cached_lengths.insert(bytes.len() as u64);
        match self.seen_files.entry(identity) {
            Entry::Vacant(entry) => {
                entry.insert(CachedPhysicalFile {
                    first_path: path.to_path_buf(),
                    bytes: Some(bytes),
                    snapshot: Some(snapshot),
                });
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().bytes = Some(bytes);
                entry.get_mut().snapshot = Some(snapshot);
            }
        }
    }

    fn register_identity_without_bytes(
        &mut self,
        identity: PhysicalFileIdentity,
        path: &Path,
    ) -> Option<PathBuf> {
        match self.seen_files.entry(identity) {
            Entry::Vacant(entry) => {
                entry.insert(CachedPhysicalFile {
                    first_path: path.to_path_buf(),
                    bytes: None,
                    snapshot: None,
                });
                None
            }
            Entry::Occupied(entry) if entry.get().first_path == path => None,
            Entry::Occupied(entry) => {
                self.counters.duplicate_identities += 1;
                Some(entry.get().first_path.clone())
            }
        }
    }

    pub(super) fn identity_unavailable(&mut self) -> OfflineIntegrityOutcome {
        self.completeness = OfflineIntegrityReportCompleteness::Indeterminate;
        OfflineIntegrityOutcome::Indeterminate(
            OfflineIndeterminatePhysicalReason::PhysicalIdentityUnavailable,
        )
    }

    pub(super) fn source_changed(&mut self) -> OfflineIntegrityOutcome {
        self.completeness = OfflineIntegrityReportCompleteness::Indeterminate;
        OfflineIntegrityOutcome::Indeterminate(OfflineIndeterminatePhysicalReason::SourceChanged)
    }
}

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use super::{BoundedAcquisition, BoundedMediaWalk, CachedPhysicalFile};
use crate::integrity_observation::file_identity::same_snapshot;
use crate::integrity_observation::{OfflineIndeterminatePhysicalReason, OfflineIntegrityOutcome};

impl BoundedMediaWalk {
    /// Acquires only the routed frame. Arena bytes are never retained in the
    /// whole-file cache; a stable physical snapshot binds successive reads.
    pub(crate) fn acquire_range(
        &mut self,
        path: &Path,
        depth: u32,
        offset: u64,
        length: u64,
    ) -> Result<BoundedAcquisition, OfflineIntegrityOutcome> {
        self.admit_acquisition_path(path, depth)?;
        if length > self.remaining_byte_budget() {
            return Err(self.bound(OfflineIndeterminatePhysicalReason::ByteBoundExceeded));
        }
        let guards = self.open_containment_guards(path)?;
        let mut file = self.open_identity_bound_file(path, guards.len())?;
        let before = file.metadata().map_err(|_| self.indeterminate_io())?;
        if !before.is_file() {
            return Err(self.source_changed());
        }
        let identity = self.identity_from_open_file(&file, path)?;
        self.verify_path_binding(&file, path, &identity)?;
        if self
            .seen_files
            .get(&identity)
            .and_then(|cached| cached.snapshot.as_ref())
            .is_some_and(|snapshot| !same_snapshot(snapshot, &before))
        {
            return Err(self.source_changed());
        }
        let (physical_alias_of, _) = self.cached_alias(&identity, path);
        let byte_length = usize::try_from(before.len())
            .map_err(|_| self.bound(OfflineIndeterminatePhysicalReason::ByteBoundExceeded))?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|_| self.indeterminate_io())?;
        let mut bytes = Vec::new();
        let result = file.by_ref().take(length).read_to_end(&mut bytes);
        self.counters.bytes_read = self.counters.bytes_read.saturating_add(bytes.len() as u64);
        result.map_err(|_| self.indeterminate_io())?;
        let after = file.metadata().map_err(|_| self.indeterminate_io())?;
        self.verify_path_binding(&file, path, &identity)?;
        if !same_snapshot(&before, &after) {
            return Err(self.source_changed());
        }
        if let Some(reason) = self.elapsed_exhaustion() {
            return Err(self.bound(reason));
        }
        self.seen_files
            .entry(identity)
            .and_modify(|cached| {
                cached.snapshot = Some(before.clone());
            })
            .or_insert_with(|| CachedPhysicalFile {
                first_path: path.to_path_buf(),
                bytes: None,
                snapshot: Some(before),
            });
        Ok(BoundedAcquisition {
            bytes: bytes.into(),
            byte_length,
            physical_alias_of,
        })
    }
}

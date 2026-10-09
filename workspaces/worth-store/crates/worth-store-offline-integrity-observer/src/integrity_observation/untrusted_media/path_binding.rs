use std::path::{Component, Path};

use super::{BoundedMediaWalk, PhysicalFileIdentity};
#[cfg(not(windows))]
use crate::integrity_observation::file_identity::identity_from_path;
use crate::integrity_observation::file_identity::open_directory_guard;
use crate::integrity_observation::{OfflineIndeterminatePhysicalReason, OfflineIntegrityOutcome};

impl BoundedMediaWalk {
    pub(super) fn open_containment_guards(
        &mut self,
        path: &Path,
    ) -> Result<Vec<std::fs::File>, OfflineIntegrityOutcome> {
        let relative = path
            .strip_prefix(&self.store_root)
            .map_err(|_| self.source_changed())?;
        let mut directory_paths = vec![self.store_root.clone()];
        let mut current = self.store_root.clone();
        for component in relative.parent().into_iter().flat_map(Path::components) {
            let Component::Normal(name) = component else {
                return Err(self.source_changed());
            };
            current.push(name);
            directory_paths.push(current.clone());
        }
        let mut guards = Vec::with_capacity(directory_paths.len());
        for directory in directory_paths {
            if guards.len().saturating_add(1) > self.limits.maximum_open_files() as usize {
                return Err(self.bound(OfflineIndeterminatePhysicalReason::OpenFileBoundExceeded));
            }
            let guard = open_directory_guard(&directory).map_err(|_| self.source_changed())?;
            self.counters.files_opened += 1;
            self.counters.open_file_high_water = self
                .counters
                .open_file_high_water
                .max(guards.len().saturating_add(1) as u32);
            let handle_is_directory = guard
                .metadata()
                .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink());
            let path_is_directory = directory
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink());
            let contained = directory
                .canonicalize()
                .is_ok_and(|resolved| resolved.starts_with(&self.store_root));
            if !handle_is_directory || !path_is_directory || !contained {
                return Err(self.source_changed());
            }
            guards.push(guard);
        }
        Ok(guards)
    }

    pub(super) fn verify_path_binding(
        &mut self,
        file: &std::fs::File,
        path: &Path,
        expected_identity: &PhysicalFileIdentity,
    ) -> Result<(), OfflineIntegrityOutcome> {
        let contained = self.path_is_plain_contained_file(path);
        let observed_identity = self.identity_from_live_path(file, path)?;
        if !contained || &observed_identity != expected_identity {
            return Err(self.source_changed());
        }
        self.bind_path_identity(path, expected_identity)
    }

    fn bind_path_identity(
        &mut self,
        path: &Path,
        identity: &PhysicalFileIdentity,
    ) -> Result<(), OfflineIntegrityOutcome> {
        if let Some(first_identity) = self.path_identities.get(path) {
            return if first_identity == identity {
                Ok(())
            } else {
                Err(self.source_changed())
            };
        }
        if self.path_identities.len() as u64 >= self.limits.maximum_entries() {
            return Err(self.entry_bound());
        }
        self.path_identities
            .insert(path.to_path_buf(), identity.clone());
        Ok(())
    }

    pub(super) fn path_is_plain_contained_file(&self, path: &Path) -> bool {
        path.symlink_metadata().is_ok_and(|metadata| {
            !metadata.file_type().is_symlink()
                && metadata.is_file()
                && path
                    .canonicalize()
                    .is_ok_and(|resolved| resolved.starts_with(&self.store_root))
        })
    }

    fn identity_from_live_path(
        &mut self,
        _file: &std::fs::File,
        path: &Path,
    ) -> Result<PhysicalFileIdentity, OfflineIntegrityOutcome> {
        #[cfg(windows)]
        {
            return self.identity_from_open_file(_file, path);
        }
        #[cfg(not(windows))]
        {
            let maximum_output_bytes = self.limits.maximum_bytes();
            let maximum_elapsed = self.remaining_elapsed_budget();
            identity_from_path(_file, path, maximum_output_bytes, maximum_elapsed)
                .map_err(|_| self.identity_unavailable())
        }
    }

    #[cfg(windows)]
    pub(super) fn charge_path_identity_lookup(
        &mut self,
        path: &Path,
    ) -> Result<(), OfflineIntegrityOutcome> {
        if self.limits.maximum_bytes()
            < crate::integrity_observation::file_identity::WINDOWS_HIGH_RES_IDENTITY_BYTES
            || self.remaining_elapsed_budget().is_zero()
        {
            return Err(self.identity_unavailable());
        }
        let relative = path
            .strip_prefix(&self.store_root)
            .map_err(|_| self.source_changed())?;
        // The path query opens one transient handle in addition to the held
        // directory guards and read handle. Charge the attempted open because
        // its safe API does not expose whether a failed query opened the file.
        let held_handles = relative
            .parent()
            .map_or(1, |parent| parent.components().count().saturating_add(1))
            .saturating_add(1);
        if held_handles.saturating_add(1) > self.limits.maximum_open_files() as usize {
            return Err(self.bound(OfflineIndeterminatePhysicalReason::OpenFileBoundExceeded));
        }
        self.counters.files_opened += 1;
        self.counters.open_file_high_water = self
            .counters
            .open_file_high_water
            .max(held_handles.saturating_add(1) as u32);
        Ok(())
    }
}

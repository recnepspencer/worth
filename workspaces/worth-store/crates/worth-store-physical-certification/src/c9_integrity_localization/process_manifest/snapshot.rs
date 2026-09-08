use super::*;
impl ProcessTreeSnapshot {
    pub(crate) fn require_only_files_delta(
        &self, root: &Path, allowed: &[PathBuf], target: &Path,
    ) -> Result<(Vec<u8>, Vec<u8>), ProcessManifestDenial> {
        let observed = Self::observe_with_lease(root, self.live_lease_payload_excluded)?;
        if self.directories != observed.directories || self.contents.keys().ne(observed.contents.keys()) {
            return Err(ProcessManifestDenial::MutationMismatch);
        }
        for (path, before) in &self.contents {
            if !allowed.contains(path) && observed.contents.get(path) != Some(before) {
                return Err(ProcessManifestDenial::MutationMismatch);
            }
        }
        Ok((self.contents.get(target).ok_or(ProcessManifestDenial::MutationMismatch)?.clone(),
            observed.contents.get(target).ok_or(ProcessManifestDenial::MutationMismatch)?.clone()))
    }
    pub(crate) fn require_only_file_delta(
        &self,
        root: &Path,
        target: &Path,
    ) -> Result<(Vec<u8>, Vec<u8>), ProcessManifestDenial> {
        let observed = Self::observe_with_lease(root, self.live_lease_payload_excluded)?;
        if self.directories != observed.directories
            || self.contents.keys().ne(observed.contents.keys())
        {
            return Err(ProcessManifestDenial::MutationMismatch);
        }
        for (path, before) in &self.contents {
            if path != target && observed.contents.get(path) != Some(before) {
                return Err(ProcessManifestDenial::MutationMismatch);
            }
        }
        let before = self
            .contents
            .get(target)
            .ok_or(ProcessManifestDenial::MutationMismatch)?
            .clone();
        let after = observed
            .contents
            .get(target)
            .ok_or(ProcessManifestDenial::MutationMismatch)?
            .clone();
        Ok((before, after))
    }
    pub(crate) fn observe(root: &Path) -> Result<Self, ProcessManifestDenial> {
        Self::observe_with_lease(root, false)
    }
    pub(crate) fn observe_live_diagnostic(root: &Path) -> Result<Self, ProcessManifestDenial> {
        Self::observe_with_lease(root, true)
    }
    fn observe_with_lease(
        root: &Path,
        live_lease_payload_excluded: bool,
    ) -> Result<Self, ProcessManifestDenial> {
        if live_lease_payload_excluded && !root.join("namespace/mutation.lock").is_file() {
            return Err(ProcessManifestDenial::TreeRead);
        }
        let (directories, files, contents) = observe_tree(root, live_lease_payload_excluded)?;
        Ok(Self {
            live_lease_payload_excluded,
            directories,
            files,
            contents,
        })
    }

    pub(crate) fn require_unchanged(&self, root: &Path) -> Result<(), ProcessManifestDenial> {
        let observed = Self::observe_with_lease(root, self.live_lease_payload_excluded)?;
        if observed == *self {
            Ok(())
        } else {
            Err(ProcessManifestDenial::TreeMismatch)
        }
    }

    pub(crate) fn require_exact_one_byte_delta(
        &self,
        root: &Path,
        target: &Path,
        offset: u64,
        xor_mask: u8,
    ) -> Result<([u8; 32], [u8; 32]), ProcessManifestDenial> {
        let observed = Self::observe_with_lease(root, self.live_lease_payload_excluded)?;
        if self.directories != observed.directories
            || self.contents.keys().ne(observed.contents.keys())
        {
            return Err(ProcessManifestDenial::MutationMismatch);
        }
        let target_offset =
            usize::try_from(offset).map_err(|_| ProcessManifestDenial::MutationMismatch)?;
        let mut target_digests = None;
        for (relative, before) in &self.contents {
            let after = observed
                .contents
                .get(relative)
                .ok_or(ProcessManifestDenial::MutationMismatch)?;
            if relative != target {
                if before != after {
                    return Err(ProcessManifestDenial::MutationMismatch);
                }
                continue;
            }
            if before.len() != after.len()
                || target_offset >= before.len()
                || before
                    .iter()
                    .zip(after)
                    .enumerate()
                    .filter(|(_, (left, right))| left != right)
                    .map(|(index, _)| index)
                    .ne([target_offset])
                || after[target_offset] != before[target_offset] ^ xor_mask
            {
                return Err(ProcessManifestDenial::MutationMismatch);
            }
            target_digests = Some((Sha256::digest(before).into(), Sha256::digest(after).into()));
        }
        target_digests.ok_or(ProcessManifestDenial::MutationMismatch)
    }
}

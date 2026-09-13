use std::path::Path;

use crate::OfflinePhysicalArtifactFamily;
use worth_store_physical_backend::{OfflineMediaConsistencyBasis, OfflineMediaFileIdentity};
use worth_store_physical_format::PhysicalGenerationOwner;

use super::OfflineInspectionCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineStructuralIdentification {
    /// The family is only a routing hint derived from the path. No owner
    /// decoder has admitted the artifact's structure or generation.
    FileNameHint,
    /// A physical-format owner decoded the artifact and bound its coordinates.
    OwnerDecoded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineWalkedFile {
    source_index: usize,
    source: OfflineMediaFileIdentity,
    family: OfflinePhysicalArtifactFamily,
    content_digest: [u8; 32],
    structural_identification: OfflineStructuralIdentification,
    generation: Option<u64>,
    physical_owner: Option<PhysicalGenerationOwner>,
}

impl OfflineWalkedFile {
    pub(crate) fn new(
        source_index: usize,
        source: OfflineMediaFileIdentity,
        family: OfflinePhysicalArtifactFamily,
        content_digest: [u8; 32],
    ) -> Self {
        Self {
            source_index,
            source,
            family,
            content_digest,
            structural_identification: OfflineStructuralIdentification::FileNameHint,
            generation: None,
            physical_owner: None,
        }
    }
    pub const fn source_index(&self) -> usize {
        self.source_index
    }
    pub fn path(&self) -> &Path {
        self.source.path()
    }
    pub const fn length(&self) -> u64 {
        self.source.length()
    }
    pub const fn family(&self) -> OfflinePhysicalArtifactFamily {
        self.family
    }
    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }
    pub const fn source(&self) -> &OfflineMediaFileIdentity {
        &self.source
    }
    pub const fn structural_identification(&self) -> OfflineStructuralIdentification {
        self.structural_identification
    }
    pub const fn generation(&self) -> Option<u64> {
        self.generation
    }
    pub const fn physical_owner(&self) -> Option<PhysicalGenerationOwner> {
        self.physical_owner
    }

    fn bind_owner(&mut self, binding: OwnerDecodedArtifactBinding) {
        self.family = binding.family;
        self.structural_identification = OfflineStructuralIdentification::OwnerDecoded;
        self.generation = Some(binding.generation);
        self.physical_owner = binding.physical_owner;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnerDecodedArtifactBinding {
    path: std::path::PathBuf,
    family: OfflinePhysicalArtifactFamily,
    generation: u64,
    physical_owner: Option<PhysicalGenerationOwner>,
}

impl OwnerDecodedArtifactBinding {
    pub(crate) fn new(
        path: std::path::PathBuf,
        family: OfflinePhysicalArtifactFamily,
        generation: u64,
    ) -> Option<Self> {
        (generation > 0).then_some(Self {
            path,
            family,
            generation,
            physical_owner: None,
        })
    }

    pub(crate) fn with_physical_owner(
        path: std::path::PathBuf,
        family: OfflinePhysicalArtifactFamily,
        generation: u64,
        physical_owner: PhysicalGenerationOwner,
    ) -> Option<Self> {
        (generation > 0 && physical_owner.generation().get() == generation).then_some(Self {
            path,
            family,
            generation,
            physical_owner: Some(physical_owner),
        })
    }

    pub(crate) fn owned_allocation_bytes(&self) -> Option<u64> {
        path_owned_allocation_bytes(&self.path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OwnerObservationBindingDenial {
    DuplicateSource,
    MissingSource,
}

#[cfg(windows)]
fn path_owned_allocation_bytes(path: &Path) -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    u64::try_from(path.as_os_str().encode_wide().count())
        .ok()?
        .checked_mul(2)
}

#[cfg(unix)]
fn path_owned_allocation_bytes(path: &Path) -> Option<u64> {
    use std::os::unix::ffi::OsStrExt;
    u64::try_from(path.as_os_str().as_bytes().len()).ok()
}

#[cfg(not(any(windows, unix)))]
fn path_owned_allocation_bytes(path: &Path) -> Option<u64> {
    u64::try_from(path.to_string_lossy().len()).ok()
}

#[derive(Debug)]
pub struct StructurallyWalkedMedia {
    consistency_basis: OfflineMediaConsistencyBasis,
    files: Vec<OfflineWalkedFile>,
    counters: OfflineInspectionCounters,
}

impl StructurallyWalkedMedia {
    pub(crate) fn new(
        consistency_basis: OfflineMediaConsistencyBasis,
        files: Vec<OfflineWalkedFile>,
        counters: OfflineInspectionCounters,
    ) -> Self {
        Self {
            consistency_basis,
            files,
            counters,
        }
    }
    pub const fn consistency_basis(&self) -> &OfflineMediaConsistencyBasis {
        &self.consistency_basis
    }
    pub fn files(&self) -> &[OfflineWalkedFile] {
        &self.files
    }
    pub const fn counters(&self) -> OfflineInspectionCounters {
        self.counters
    }
    pub fn admitted_bytes(&self) -> u64 {
        self.counters.bytes_read()
    }

    pub fn owned_allocation_bytes(&self) -> Option<u64> {
        let basis = self.consistency_basis.owned_allocation_bytes()?;
        let rows = u64::try_from(self.files.capacity())
            .ok()?
            .checked_mul(std::mem::size_of::<OfflineWalkedFile>() as u64)?;
        self.files
            .iter()
            .try_fold(basis.checked_add(rows)?, |total, file| {
                total.checked_add(file.source.owned_allocation_bytes()?)
            })
    }

    pub(crate) fn bind_owner_observations(
        &mut self,
        mut bindings_by_path: Vec<OwnerDecodedArtifactBinding>,
    ) -> Result<(), OwnerObservationBindingDenial> {
        bindings_by_path.sort_by(|left, right| left.path.cmp(&right.path));
        if bindings_by_path
            .windows(2)
            .any(|pair| pair[0].path == pair[1].path)
        {
            return Err(OwnerObservationBindingDenial::DuplicateSource);
        }
        if bindings_by_path.len() != self.files.len() {
            return Err(OwnerObservationBindingDenial::MissingSource);
        }
        for (file, binding) in self.files.iter_mut().zip(bindings_by_path) {
            if binding.path.as_path() != file.path() {
                return Err(OwnerObservationBindingDenial::MissingSource);
            }
            file.bind_owner(binding);
        }
        Ok(())
    }

    pub(crate) fn remove_auxiliary_components(
        &mut self,
        auxiliary: &[worth_store_physical_backend::OfflineMediaClosureEntry],
    ) -> bool {
        let before = self.files.len();
        self.files.retain(|file| {
            !auxiliary.iter().any(|entry| {
                entry.path().file_name() == file.path().file_name()
                    && entry.path().parent() == file.path().parent()
            })
        });
        before.saturating_sub(self.files.len()) == auxiliary.len()
    }
}

use worth_store_physical_backend::{ArtifactTreeDirectory, ArtifactTreeFailure};
use worth_store_physical_format::RecordArtifactFile;

use super::{PhysicalRecordArtifactTree, RecordFamilyCreationFailure, RecordFamilyInventory};

impl PhysicalRecordArtifactTree<'_> {
    pub(in crate::physical_runtime::record_serving) fn inventory(
        &self,
    ) -> Result<RecordFamilyInventory, ArtifactTreeFailure> {
        let family = self.tree.directory_exists(&self.record_family)?;
        let staging = self.tree.directory_exists(&self.record_staging)?;
        if !family && !staging {
            return Ok(RecordFamilyInventory::ProvenAbsent);
        }
        let catalog = self
            .tree
            .file_exists(&self.artifact(RecordArtifactFile::BootstrapCatalog))?;
        if family && staging && self.all_family_directories_exist()? && catalog {
            Ok(RecordFamilyInventory::Published)
        } else {
            Ok(RecordFamilyInventory::Residue)
        }
    }

    pub(in crate::physical_runtime::record_serving) fn create_record_family(
        &self,
    ) -> Result<(), RecordFamilyCreationFailure> {
        self.tree
            .create_directory(&self.record_family)
            .map_err(RecordFamilyCreationFailure::BeforeEffect)?;
        self.synchronize_created_directory_parent(&self.families)?;
        self.create_and_publish_directory(&self.record_staging, &self.staging)?;
        for directory in self.family_directories() {
            self.tree
                .create_directory(directory)
                .map_err(RecordFamilyCreationFailure::AfterEffect)?;
        }
        self.synchronize_created_directory_parent(&self.record_family)
    }

    fn all_family_directories_exist(&self) -> Result<bool, ArtifactTreeFailure> {
        for directory in self.family_directories() {
            if !self.tree.directory_exists(directory)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn family_directories(&self) -> [&ArtifactTreeDirectory; 5] {
        [
            &self.root_manifests,
            &self.page_segments,
            &self.segment_manifests,
            &self.arenas,
            &self.free_space_manifests,
        ]
    }

    fn create_and_publish_directory(
        &self,
        directory: &ArtifactTreeDirectory,
        parent: &ArtifactTreeDirectory,
    ) -> Result<(), RecordFamilyCreationFailure> {
        self.tree
            .create_directory(directory)
            .map_err(RecordFamilyCreationFailure::AfterEffect)?;
        self.synchronize_created_directory_parent(parent)
    }

    fn synchronize_created_directory_parent(
        &self,
        parent: &ArtifactTreeDirectory,
    ) -> Result<(), RecordFamilyCreationFailure> {
        self.tree
            .synchronize_directory(parent)
            .map_err(RecordFamilyCreationFailure::AfterEffect)
    }
}

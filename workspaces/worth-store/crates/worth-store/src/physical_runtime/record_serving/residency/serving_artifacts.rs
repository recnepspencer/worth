use worth_store_physical_backend::{ArtifactTreeFailure, QualifiedFilesystemMedia};
use worth_store_physical_format::RecordArtifactFile;

use super::{
    artifact_tree::{PhysicalRecordArtifactTree, RecordFamilyInventory},
    frame_loading::{ExactFrameSourceExtent, FrameLoadFailure, LoadedPhysicalFrame},
    frame_ports::FrameLoadPort,
    record_frame_reader::RecordFrameReader,
    PhysicalResidencyWorkPort,
};

pub(in crate::physical_runtime::record_serving) struct ServingRecordArtifacts<'media> {
    tree: PhysicalRecordArtifactTree<'media>,
    reader: RecordFrameReader<'media>,
}

impl<'media> ServingRecordArtifacts<'media> {
    pub(in crate::physical_runtime::record_serving) fn resident_admission_context(
        &self,
    ) -> Option<crate::physical_runtime::integrity::ResidentAdmissionContext<'_>> {
        self.reader.resident_admission_context()
    }

    pub(in crate::physical_runtime::record_serving) fn new(
        media: &'media QualifiedFilesystemMedia,
        loader: &'media (dyn FrameLoadPort + Send + Sync),
    ) -> Self {
        Self {
            tree: PhysicalRecordArtifactTree::new(media),
            reader: RecordFrameReader::bootstrap(media, loader),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn serving(
        media: &'media QualifiedFilesystemMedia,
        residency: PhysicalResidencyWorkPort,
    ) -> Self {
        Self {
            tree: PhysicalRecordArtifactTree::new(media),
            reader: RecordFrameReader::serving(residency),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn inventory(
        &self,
    ) -> Result<RecordFamilyInventory, ArtifactTreeFailure> {
        self.tree.inventory()
    }

    pub(in crate::physical_runtime::record_serving) fn has_staging_residue(
        &self,
    ) -> Result<bool, ArtifactTreeFailure> {
        self.tree.has_staging_residue()
    }

    pub(in crate::physical_runtime::record_serving) fn file_exists(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<bool, ArtifactTreeFailure> {
        self.tree.file_exists(artifact)
    }

    /// Bytes a displaced generation still occupies, or `None` once retirement
    /// removed every file it owned.
    pub(in crate::physical_runtime::record_serving) fn retained_bytes(
        &self,
        artifact: crate::physical_runtime::durability::RetiredArtifact,
    ) -> Result<Option<u64>, ArtifactTreeFailure> {
        let mut retained = None;
        for file in artifact.files() {
            if self.tree.file_exists(file)? {
                let bytes = self.tree.file_length(file)?;
                retained = Some(retained.unwrap_or(0_u64).saturating_add(bytes));
            }
        }
        Ok(retained)
    }

    pub(in crate::physical_runtime::record_serving) fn load_exact(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        artifact: RecordArtifactFile,
        offset: u64,
        length: u32,
        source_extent: ExactFrameSourceExtent,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        self.reader
            .load_exact(allocation, artifact, offset, length, source_extent)
    }

    pub(in crate::physical_runtime::record_serving) fn load_bounded(
        &self,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
        artifact: RecordArtifactFile,
        limit: u32,
    ) -> Result<LoadedPhysicalFrame, FrameLoadFailure> {
        self.reader.load_bounded(allocation, artifact, limit)
    }
}

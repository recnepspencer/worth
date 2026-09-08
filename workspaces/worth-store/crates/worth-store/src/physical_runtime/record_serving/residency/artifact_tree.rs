use worth_store_physical_backend::{
    ArtifactNewWriteRange, ArtifactRangeWriteDurabilityRequirement, ArtifactTreeDirectory,
    ArtifactTreeFailure, ArtifactTreeFile, ArtifactTreeMedia, ArtifactTreeReplacement,
    BackendQueueExecutionAdaptation, BackendQueueExecutionPlanBinding,
    BackendQueueSpeculativeScope, QualifiedFilesystemMedia, ScheduledArtifactMetadataReadOutcome,
    ScheduledArtifactNewWriteOutcome, ScheduledArtifactRangeReadOutcome,
    ScheduledArtifactRangeWriteOutcome, ScheduledArtifactTreePublicationEffectOutcome,
};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

mod family_lifecycle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum RecordFamilyInventory {
    ProvenAbsent,
    Published,
    Residue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::record_serving) enum RecordFamilyCreationFailure {
    BeforeEffect(ArtifactTreeFailure),
    AfterEffect(ArtifactTreeFailure),
}

pub(in crate::physical_runtime) struct PhysicalRecordArtifactTree<'media> {
    tree: ArtifactTreeMedia<'media>,
    families: ArtifactTreeDirectory,
    staging: ArtifactTreeDirectory,
    record_family: ArtifactTreeDirectory,
    root_manifests: ArtifactTreeDirectory,
    page_segments: ArtifactTreeDirectory,
    segment_manifests: ArtifactTreeDirectory,
    extents: ArtifactTreeDirectory,
    extent_manifests: ArtifactTreeDirectory,
    free_space_manifests: ArtifactTreeDirectory,
    record_staging: ArtifactTreeDirectory,
}

impl<'media> PhysicalRecordArtifactTree<'media> {
    pub(in crate::physical_runtime) fn new(media: &'media QualifiedFilesystemMedia) -> Self {
        let families = ArtifactTreeDirectory::families();
        let staging = ArtifactTreeDirectory::staging();
        let record_family = families.child("records").expect("portable Store layout");
        let root_manifests = record_family.child("roots").expect("portable Store layout");
        let page_segments = record_family
            .child("segments")
            .expect("portable Store layout");
        let segment_manifests = record_family
            .child("segment-manifests")
            .expect("portable Store layout");
        let extents = record_family
            .child("extents")
            .expect("portable Store layout");
        let extent_manifests = record_family
            .child("extent-manifests")
            .expect("portable Store layout");
        let free_space_manifests = record_family
            .child("free-space")
            .expect("portable Store layout");
        let record_staging = staging.child("records").expect("portable Store layout");
        Self {
            tree: media.artifact_tree(),
            families,
            staging,
            record_family,
            root_manifests,
            page_segments,
            segment_manifests,
            extents,
            extent_manifests,
            free_space_manifests,
            record_staging,
        }
    }

    pub(in crate::physical_runtime::record_serving) fn write_new(
        &self,
        artifact: RecordArtifactFile,
        bytes: &[u8],
    ) -> Result<(), ArtifactTreeFailure> {
        self.tree.write_new(&self.artifact(artifact), bytes)
    }

    pub(in crate::physical_runtime::record_serving) fn file_length(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<u64, ArtifactTreeFailure> {
        self.tree.file_length(&self.artifact(artifact))
    }

    pub(in crate::physical_runtime::record_serving) fn file_exists(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<bool, ArtifactTreeFailure> {
        self.tree.file_exists(&self.artifact(artifact))
    }

    pub(in crate::physical_runtime::record_serving) fn remove_file_durably(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<(), ArtifactTreeFailure> {
        self.tree.remove_file_durably(&self.artifact(artifact))
    }

    pub(in crate::physical_runtime::record_serving) fn read_exact_at(
        &self,
        artifact: RecordArtifactFile,
        offset: u64,
        target: &mut [u8],
    ) -> Result<(), ArtifactTreeFailure> {
        self.tree
            .read_exact_at(&self.artifact(artifact), offset, target)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime) fn write_scheduled_exact_at(
        &self,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        scope: BackendQueueSpeculativeScope,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.tree.write_scheduled_exact_at(
            &self.artifact(coordinate.artifact()),
            coordinate,
            bytes,
            binding,
            adaptation,
            scope,
            durability,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime) fn append_scheduled_writeback_at_eof(
        &self,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        scope: BackendQueueSpeculativeScope,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.tree.append_scheduled_writeback_at_eof(
            &self.artifact(coordinate.artifact()),
            coordinate,
            bytes,
            binding,
            adaptation,
            scope,
            durability,
        )
    }

    pub(in crate::physical_runtime) fn read_scheduled_exact_at(
        &self,
        coordinate: RecordFrameCoordinate,
        target: &mut [u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactRangeReadOutcome {
        self.tree.read_scheduled_exact_at(
            &self.artifact(coordinate.artifact()),
            coordinate,
            target,
            binding,
            adaptation,
        )
    }

    pub(in crate::physical_runtime) fn read_scheduled_file_length(
        &self,
        artifact: RecordArtifactFile,
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactMetadataReadOutcome {
        self.tree.read_scheduled_file_length(
            &self.artifact(artifact),
            artifact,
            binding,
            adaptation,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::physical_runtime) fn write_scheduled_foreground_exact_at(
        &self,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
        durability: ArtifactRangeWriteDurabilityRequirement,
    ) -> ScheduledArtifactRangeWriteOutcome {
        self.tree.write_scheduled_foreground_exact_at(
            &self.artifact(coordinate.artifact()),
            coordinate,
            bytes,
            binding,
            adaptation,
            durability,
        )
    }

    pub(in crate::physical_runtime) fn write_scheduled_new_exact(
        &self,
        coordinate: RecordFrameCoordinate,
        bytes: &[u8],
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactNewWriteOutcome {
        self.tree.write_scheduled_new_exact(
            &self.artifact(coordinate.artifact()),
            ArtifactNewWriteRange::new(u64::from(coordinate.length()))
                .expect("record frame coordinates are nonempty"),
            bytes,
            binding,
            adaptation,
        )
    }

    pub(in crate::physical_runtime) fn synchronize_scheduled_artifact(
        &self,
        artifact: RecordArtifactFile,
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        self.tree
            .synchronize_scheduled_file(&self.artifact(artifact), binding, adaptation)
    }

    pub(in crate::physical_runtime) fn synchronize_scheduled_artifact_parent(
        &self,
        artifact: RecordArtifactFile,
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        self.tree.synchronize_scheduled_directory(
            self.artifact_directory(artifact),
            binding,
            adaptation,
        )
    }

    pub(in crate::physical_runtime) fn replace_scheduled_catalog(
        &self,
        candidate: RecordArtifactFile,
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        let RecordArtifactFile::CatalogCandidate { publication } = candidate else {
            unreachable!("root protocol replacement requires its catalog candidate")
        };
        let previous_candidate = RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Previous,
            publication,
        };
        let current_candidate = RecordArtifactFile::RootSelectorCandidate {
            role: worth_store_physical_format::RootSelectorRole::Current,
            publication,
        };
        self.tree.replace_root_protocol_scheduled(
            ArtifactTreeReplacement::new(
                self.artifact(previous_candidate),
                self.artifact(RecordArtifactFile::PreviousRootSelector),
            ),
            ArtifactTreeReplacement::new(
                self.artifact(current_candidate),
                self.artifact(RecordArtifactFile::CurrentRootSelector),
            ),
            ArtifactTreeReplacement::new(
                self.artifact(candidate),
                self.artifact(RecordArtifactFile::BootstrapCatalog),
            ),
            binding,
            adaptation,
        )
    }

    pub(in crate::physical_runtime) fn synchronize_scheduled_record_family(
        &self,
        binding: BackendQueueExecutionPlanBinding,
        adaptation: BackendQueueExecutionAdaptation,
    ) -> ScheduledArtifactTreePublicationEffectOutcome {
        self.tree
            .synchronize_scheduled_directory(&self.record_family, binding, adaptation)
    }

    pub(in crate::physical_runtime::record_serving) fn synchronize_artifact(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<(), ArtifactTreeFailure> {
        self.tree.synchronize_file(&self.artifact(artifact))
    }

    pub(in crate::physical_runtime::record_serving) fn synchronize_artifact_parent(
        &self,
        artifact: RecordArtifactFile,
    ) -> Result<(), ArtifactTreeFailure> {
        let directory = self.artifact_directory(artifact);
        self.tree.synchronize_directory(directory)
    }

    pub(in crate::physical_runtime::record_serving) fn replace_catalog(
        &self,
        candidate: RecordArtifactFile,
    ) -> Result<(), ArtifactTreeFailure> {
        let RecordArtifactFile::CatalogCandidate { .. } = candidate else {
            unreachable!("only catalog candidates can become the bootstrap catalog");
        };
        self.tree.replace(
            &self.artifact(candidate),
            &self.artifact(RecordArtifactFile::BootstrapCatalog),
        )
    }

    pub(in crate::physical_runtime::record_serving) fn synchronize_record_family(
        &self,
    ) -> Result<(), ArtifactTreeFailure> {
        self.tree.synchronize_directory(&self.record_family)
    }

    pub(in crate::physical_runtime::record_serving) fn has_staging_residue(
        &self,
    ) -> Result<bool, ArtifactTreeFailure> {
        self.tree.directory_has_entries(&self.record_staging)
    }

    pub(in crate::physical_runtime) fn artifact(
        &self,
        artifact: RecordArtifactFile,
    ) -> ArtifactTreeFile {
        self.artifact_directory(artifact)
            .file(&artifact.file_name())
            .expect("encoded Store artifact names are portable")
    }

    fn artifact_directory(&self, artifact: RecordArtifactFile) -> &ArtifactTreeDirectory {
        match artifact {
            RecordArtifactFile::BootstrapCatalog
            | RecordArtifactFile::CurrentRootSelector
            | RecordArtifactFile::PreviousRootSelector => &self.record_family,
            RecordArtifactFile::CatalogCandidate { .. }
            | RecordArtifactFile::RootSelectorCandidate { .. } => &self.record_staging,
            RecordArtifactFile::RootManifest { .. }
            | RecordArtifactFile::RootRoutingBlock { .. } => &self.root_manifests,
            RecordArtifactFile::Segment { .. } => &self.page_segments,
            RecordArtifactFile::SegmentManifest { .. }
            | RecordArtifactFile::SegmentMembershipBlock { .. } => &self.segment_manifests,
            RecordArtifactFile::Extent { .. } => &self.extents,
            RecordArtifactFile::ExtentManifest { .. } => &self.extent_manifests,
            RecordArtifactFile::FreeSpaceManifest { .. }
            | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => &self.free_space_manifests,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackupBundleArtifactFamily {
    RootManifest,
    CheckpointManifest,
    WalSegment,
    Page,
    Extent,
    Index,
    BlobChunk,
    SecondaryRoot,
}

/// The owner decoder that gives a component's bytes structural meaning.
///
/// Family answers what role the component plays. Format answers which owner
/// can decode it. Keeping these axes separate prevents a valid hash from
/// laundering arbitrary bytes into a family role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackupBundleArtifactFormat {
    PhysicalRootManifestV1,
    RecoveryCheckpointManifestV1,
    WalSegmentV1,
    PhysicalDataPageV1,
    PhysicalExtentRecordV1,
    LayoutBTreeLeafV1,
    LayoutBTreeRootV1,
    BlobChunkV1,
    PhysicalSecondaryRootManifestV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupBundleArtifactCoverage {
    RootManifest {
        root_generation: u64,
    },
    CheckpointManifest {
        checkpoint_identity: String,
        manifest_generation: u64,
        durable_checkpoint_lsn: u64,
        authority_fingerprint: [u8; 32],
        frontier_digest: [u8; 32],
    },
    WalSegment {
        start_lsn: u64,
        end_exclusive_lsn: u64,
    },
    PhysicalReachability,
    SecondaryRoot {
        root_generation: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBundleArtifactManifestRow {
    family: BackupBundleArtifactFamily,
    format: BackupBundleArtifactFormat,
    identity: String,
    output_name: String,
    generation: u64,
    bytes: u64,
    content_digest: [u8; 32],
    coverage: BackupBundleArtifactCoverage,
    reclaim_owner: super::super::BackupBundlePhysicalOwner,
}

impl BackupBundleArtifactManifestRow {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        family: BackupBundleArtifactFamily,
        format: BackupBundleArtifactFormat,
        identity: impl Into<String>,
        output_name: impl Into<String>,
        generation: u64,
        bytes: u64,
        content_digest: [u8; 32],
        coverage: BackupBundleArtifactCoverage,
        reclaim_owner: super::super::BackupBundlePhysicalOwner,
    ) -> Option<Self> {
        let identity = identity.into();
        let output_name = output_name.into();
        if identity.trim().is_empty()
            || output_name.is_empty()
            || output_name.contains(['/', '\\'])
            || generation == 0
            || bytes == 0
            || !format.matches_family(family)
            || !coverage.matches_family(family)
            || !reclaim_owner.is_valid()
            || !reclaim_owner.matches_artifact(family, generation)
        {
            None
        } else {
            Some(Self {
                family,
                format,
                identity,
                output_name,
                generation,
                bytes,
                content_digest,
                coverage,
                reclaim_owner,
            })
        }
    }
    pub const fn family(&self) -> BackupBundleArtifactFamily {
        self.family
    }
    pub const fn format(&self) -> BackupBundleArtifactFormat {
        self.format
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn output_name(&self) -> &str {
        &self.output_name
    }
    pub const fn generation(&self) -> u64 {
        self.generation
    }
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }
    pub const fn coverage(&self) -> &BackupBundleArtifactCoverage {
        &self.coverage
    }
    pub const fn reclaim_owner(&self) -> super::super::BackupBundlePhysicalOwner {
        self.reclaim_owner
    }
    pub(super) fn is_valid(&self) -> bool {
        !self.identity.trim().is_empty()
            && !self.output_name.is_empty()
            && !self.output_name.contains(['/', '\\'])
            && self.generation > 0
            && self.bytes > 0
            && self.format.matches_family(self.family)
            && self.coverage.matches_family(self.family)
            && self.reclaim_owner.is_valid()
            && self
                .reclaim_owner
                .matches_artifact(self.family, self.generation)
    }
    pub(super) fn owned_allocation_bytes(&self) -> Option<u64> {
        let coverage = match &self.coverage {
            BackupBundleArtifactCoverage::CheckpointManifest {
                checkpoint_identity,
                ..
            } => u64::try_from(checkpoint_identity.capacity()).ok()?,
            _ => 0,
        };
        u64::try_from(self.identity.capacity())
            .ok()?
            .checked_add(u64::try_from(self.output_name.capacity()).ok()?)?
            .checked_add(coverage)
    }
}

impl BackupBundleArtifactFormat {
    pub const fn matches_family(self, family: BackupBundleArtifactFamily) -> bool {
        matches!(
            (self, family),
            (
                Self::PhysicalRootManifestV1,
                BackupBundleArtifactFamily::RootManifest
            ) | (
                Self::RecoveryCheckpointManifestV1,
                BackupBundleArtifactFamily::CheckpointManifest
            ) | (Self::WalSegmentV1, BackupBundleArtifactFamily::WalSegment)
                | (Self::PhysicalDataPageV1, BackupBundleArtifactFamily::Page)
                | (
                    Self::PhysicalExtentRecordV1,
                    BackupBundleArtifactFamily::Extent
                )
                | (
                    Self::LayoutBTreeLeafV1 | Self::LayoutBTreeRootV1,
                    BackupBundleArtifactFamily::Index
                )
                | (Self::BlobChunkV1, BackupBundleArtifactFamily::BlobChunk)
                | (
                    Self::PhysicalSecondaryRootManifestV1,
                    BackupBundleArtifactFamily::SecondaryRoot
                )
        )
    }
}

impl BackupBundleArtifactCoverage {
    pub fn matches_family(&self, family: BackupBundleArtifactFamily) -> bool {
        matches!(
            (self, family),
            (
                Self::RootManifest { .. },
                BackupBundleArtifactFamily::RootManifest
            ) | (
                Self::CheckpointManifest { .. },
                BackupBundleArtifactFamily::CheckpointManifest
            ) | (
                Self::WalSegment { .. },
                BackupBundleArtifactFamily::WalSegment
            ) | (
                Self::PhysicalReachability,
                BackupBundleArtifactFamily::Page
                    | BackupBundleArtifactFamily::Extent
                    | BackupBundleArtifactFamily::Index
                    | BackupBundleArtifactFamily::BlobChunk
            ) | (
                Self::SecondaryRoot { .. },
                BackupBundleArtifactFamily::SecondaryRoot
            )
        ) && !matches!(
            self,
            Self::WalSegment { start_lsn, end_exclusive_lsn } if start_lsn >= end_exclusive_lsn
        )
    }
}

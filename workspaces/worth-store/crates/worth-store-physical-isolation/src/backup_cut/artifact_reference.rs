use crate::CurrentGenerationPhysicalReference;

mod untrusted_claim;
pub use untrusted_claim::UntrustedBackupArtifactClaim;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BackupArtifactFamily {
    RootManifest,
    CheckpointManifest,
    WalSegment,
    Page,
    Extent,
    Index,
    BlobChunk,
    SecondaryRoot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupArtifactCoverage {
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
pub struct BackupArtifactReference {
    family: BackupArtifactFamily,
    format: worth_store_physical_format::BackupBundleArtifactFormat,
    identity: String,
    source_path: std::path::PathBuf,
    generation: u64,
    bytes: u64,
    content_digest: [u8; 32],
    physical_identity: [u8; 32],
    reclaim_reference: CurrentGenerationPhysicalReference,
    coverage: BackupArtifactCoverage,
}

impl BackupArtifactReference {
    /// Declares untrusted source bytes for later owner-semantic verification.
    ///
    /// This constructor proves only physical observation and field coherence.
    /// It never grants backup-cut authority; Operations must independently
    /// decode the source through its named owner before persisting a lease.
    pub fn declare_untrusted_physical_observation(
        claim: UntrustedBackupArtifactClaim,
        observation: worth_store_physical_backend::PhysicalBackupArtifactObservation,
        reclaim_reference: CurrentGenerationPhysicalReference,
    ) -> Option<Self> {
        let source_path = observation.path().to_path_buf();
        let bytes = observation.bytes();
        let content_digest = observation.content_digest();
        let physical_identity = observation.physical_identity();
        if claim.identity.trim().is_empty()
            || source_path.as_os_str().is_empty()
            || claim.generation == 0
            || bytes == 0
            || !claim.format.matches_family(bundle_family(claim.family))
            || reclaim_reference.generation().get() != claim.generation
            || !claim.coverage.matches_family(claim.family)
            || !reclaim_domain_matches_family(claim.family, reclaim_reference)
        {
            None
        } else {
            Some(Self {
                family: claim.family,
                format: claim.format,
                identity: claim.identity,
                source_path,
                generation: claim.generation,
                bytes,
                content_digest,
                physical_identity,
                reclaim_reference,
                coverage: claim.coverage,
            })
        }
    }
    pub const fn family(&self) -> BackupArtifactFamily {
        self.family
    }
    pub const fn format(&self) -> worth_store_physical_format::BackupBundleArtifactFormat {
        self.format
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn source_path(&self) -> &std::path::Path {
        &self.source_path
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
    pub const fn physical_identity(&self) -> [u8; 32] {
        self.physical_identity
    }
    pub const fn reclaim_reference(&self) -> CurrentGenerationPhysicalReference {
        self.reclaim_reference
    }
    pub const fn coverage(&self) -> &BackupArtifactCoverage {
        &self.coverage
    }
    pub fn portable_manifest_row(
        &self,
        output_name: impl Into<String>,
    ) -> Option<worth_store_physical_format::BackupBundleArtifactManifestRow> {
        worth_store_physical_format::BackupBundleArtifactManifestRow::new(
            bundle_family(self.family),
            self.format,
            &self.identity,
            output_name,
            self.generation,
            self.bytes,
            self.content_digest,
            portable_coverage(&self.coverage),
            worth_store_physical_format::BackupBundlePhysicalOwner::from_generation_owner(
                self.reclaim_reference.owner(),
            ),
        )
    }
}

const fn bundle_family(
    family: BackupArtifactFamily,
) -> worth_store_physical_format::BackupBundleArtifactFamily {
    use worth_store_physical_format::BackupBundleArtifactFamily as Bundle;
    match family {
        BackupArtifactFamily::RootManifest => Bundle::RootManifest,
        BackupArtifactFamily::CheckpointManifest => Bundle::CheckpointManifest,
        BackupArtifactFamily::WalSegment => Bundle::WalSegment,
        BackupArtifactFamily::Page => Bundle::Page,
        BackupArtifactFamily::Extent => Bundle::Extent,
        BackupArtifactFamily::Index => Bundle::Index,
        BackupArtifactFamily::BlobChunk => Bundle::BlobChunk,
        BackupArtifactFamily::SecondaryRoot => Bundle::SecondaryRoot,
    }
}

fn reclaim_domain_matches_family(
    family: BackupArtifactFamily,
    reference: CurrentGenerationPhysicalReference,
) -> bool {
    use worth_store_physical_format::PhysicalCellReuseDomain;

    matches!(
        (family, reference.owner().domain()),
        (
            BackupArtifactFamily::RootManifest | BackupArtifactFamily::SecondaryRoot,
            PhysicalCellReuseDomain::RootPublication
        ) | (
            BackupArtifactFamily::WalSegment,
            PhysicalCellReuseDomain::Segment
        ) | (
            BackupArtifactFamily::Extent,
            PhysicalCellReuseDomain::RecordExtentAllocation
        ) | (
            BackupArtifactFamily::Extent | BackupArtifactFamily::BlobChunk,
            PhysicalCellReuseDomain::ExtentAllocation
        ) | (
            BackupArtifactFamily::CheckpointManifest | BackupArtifactFamily::Index,
            PhysicalCellReuseDomain::SlotAllocation
        ) | (BackupArtifactFamily::Page, PhysicalCellReuseDomain::Page)
    )
}

fn portable_coverage(
    coverage: &BackupArtifactCoverage,
) -> worth_store_physical_format::BackupBundleArtifactCoverage {
    use worth_store_physical_format::BackupBundleArtifactCoverage as Bundle;
    match coverage {
        BackupArtifactCoverage::RootManifest { root_generation } => Bundle::RootManifest {
            root_generation: *root_generation,
        },
        BackupArtifactCoverage::CheckpointManifest {
            checkpoint_identity,
            manifest_generation,
            durable_checkpoint_lsn,
            authority_fingerprint,
            frontier_digest,
        } => Bundle::CheckpointManifest {
            checkpoint_identity: checkpoint_identity.clone(),
            manifest_generation: *manifest_generation,
            durable_checkpoint_lsn: *durable_checkpoint_lsn,
            authority_fingerprint: *authority_fingerprint,
            frontier_digest: *frontier_digest,
        },
        BackupArtifactCoverage::WalSegment {
            start_lsn,
            end_exclusive_lsn,
        } => Bundle::WalSegment {
            start_lsn: *start_lsn,
            end_exclusive_lsn: *end_exclusive_lsn,
        },
        BackupArtifactCoverage::PhysicalReachability => Bundle::PhysicalReachability,
        BackupArtifactCoverage::SecondaryRoot { root_generation } => Bundle::SecondaryRoot {
            root_generation: *root_generation,
        },
    }
}

impl BackupArtifactCoverage {
    pub fn root_manifest(root_generation: u64) -> Option<Self> {
        (root_generation > 0).then_some(Self::RootManifest { root_generation })
    }

    pub fn checkpoint_manifest(
        checkpoint_identity: impl Into<String>,
        manifest_generation: u64,
        durable_checkpoint_lsn: u64,
        authority_fingerprint: [u8; 32],
        frontier_digest: [u8; 32],
    ) -> Option<Self> {
        let checkpoint_identity = checkpoint_identity.into();
        (!checkpoint_identity.trim().is_empty() && manifest_generation > 0).then_some(
            Self::CheckpointManifest {
                checkpoint_identity,
                manifest_generation,
                durable_checkpoint_lsn,
                authority_fingerprint,
                frontier_digest,
            },
        )
    }

    pub const fn wal_segment(start_lsn: u64, end_exclusive_lsn: u64) -> Option<Self> {
        if start_lsn < end_exclusive_lsn {
            Some(Self::WalSegment {
                start_lsn,
                end_exclusive_lsn,
            })
        } else {
            None
        }
    }

    pub const fn physical_reachability() -> Self {
        Self::PhysicalReachability
    }

    pub const fn secondary_root(root_generation: u64) -> Option<Self> {
        if root_generation > 0 {
            Some(Self::SecondaryRoot { root_generation })
        } else {
            None
        }
    }

    pub(crate) const fn matches_family(&self, family: BackupArtifactFamily) -> bool {
        matches!(
            (self, family),
            (
                Self::RootManifest { .. },
                BackupArtifactFamily::RootManifest
            ) | (
                Self::CheckpointManifest { .. },
                BackupArtifactFamily::CheckpointManifest
            ) | (Self::WalSegment { .. }, BackupArtifactFamily::WalSegment)
                | (
                    Self::PhysicalReachability,
                    BackupArtifactFamily::Page
                        | BackupArtifactFamily::Extent
                        | BackupArtifactFamily::Index
                        | BackupArtifactFamily::BlobChunk
                )
                | (
                    Self::SecondaryRoot { .. },
                    BackupArtifactFamily::SecondaryRoot
                )
        )
    }
}

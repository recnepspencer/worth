use super::{BackupBundleArtifactManifestRow, BackupBundleManifest};
use worth_foundational::CanonicalDigestId;

/// Stable identity fields declared for a backup bundle manifest.
///
/// This is unvalidated format input. Constructing it grants no backup or
/// recovery authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBundleManifestIdentity {
    pub cut_identity: CanonicalDigestId,
    pub store_lineage: String,
    pub root_generation: u64,
    pub manifest_generation: u64,
}

/// Recovery frontier fields declared for a backup bundle manifest.
///
/// The half-open WAL interval and acknowledged frontier are validated when
/// the enclosing manifest declaration is admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBundleRecoveryCoordinates {
    pub checkpoint_identity: String,
    pub durable_checkpoint_lsn: u64,
    pub wal_half_open_interval: (u64, u64),
    pub acknowledged_frontier: u64,
}

/// Complete unvalidated input to canonical backup manifest construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupBundleManifestDeclaration {
    pub(super) identity: BackupBundleManifestIdentity,
    pub(super) recovery: BackupBundleRecoveryCoordinates,
    pub(super) security_scope_fingerprint: u64,
    pub(super) artifacts: Vec<BackupBundleArtifactManifestRow>,
}

impl BackupBundleManifestDeclaration {
    pub fn new(
        identity: BackupBundleManifestIdentity,
        recovery: BackupBundleRecoveryCoordinates,
        security_scope_fingerprint: u64,
        artifacts: Vec<BackupBundleArtifactManifestRow>,
    ) -> Self {
        Self {
            identity,
            recovery,
            security_scope_fingerprint,
            artifacts,
        }
    }

    pub fn from_manifest_with_artifacts(
        manifest: &BackupBundleManifest,
        artifacts: Vec<BackupBundleArtifactManifestRow>,
    ) -> Self {
        Self::new(
            BackupBundleManifestIdentity {
                cut_identity: CanonicalDigestId::new(manifest.cut_identity()),
                store_lineage: manifest.store_lineage().to_owned(),
                root_generation: manifest.root_generation(),
                manifest_generation: manifest.manifest_generation(),
            },
            BackupBundleRecoveryCoordinates {
                checkpoint_identity: manifest.checkpoint_identity().to_owned(),
                durable_checkpoint_lsn: manifest.durable_checkpoint_lsn(),
                wal_half_open_interval: manifest.wal_half_open_interval(),
                acknowledged_frontier: manifest.acknowledged_frontier(),
            },
            manifest.security_scope_fingerprint(),
            artifacts,
        )
    }
}

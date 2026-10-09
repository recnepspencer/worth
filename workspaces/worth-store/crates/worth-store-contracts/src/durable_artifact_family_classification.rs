#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactFamilyAuthorityClass {
    Authoritative,
    Derived,
    Diagnostic,
    Terminal,
    CertificationEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactFamilyAccessLane {
    HotPath,
    MaintenancePath,
    VerifierPath,
    TerminalPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactFamilyLifecycleClass {
    CoreState,
    RecoveryState,
    DerivedState,
    OperationalSupport,
    TransferBoundary,
    EvidenceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableArtifactOwningBoundary {
    WorthStorePhysicalFormat,
    WorthStoreWal,
    WorthStoreRecoveryPhysics,
    WorthStoreBufferPool,
    WorthStorePhysicalIntegrity,
    WorthStorePhysicalIsolation,
    WorthStoreIoScheduler,
    WorthStoreBlobChunks,
    WorthStoreSecurity,
    WorthStoreOperations,
    WorthStoreMaintenance,
    WorthStoreRetention,
    WorthStoreTiering,
    WorthStoreSnapshots,
    WorthStoreBranchDeltas,
    WorthStoreOfflineVerifier,
}

impl DurableArtifactOwningBoundary {
    pub const fn crate_name(self) -> &'static str {
        match self {
            Self::WorthStorePhysicalFormat => "worth-store-physical-format",
            Self::WorthStoreWal => "worth-store-wal",
            Self::WorthStoreRecoveryPhysics => "worth-store-recovery-physics",
            Self::WorthStoreBufferPool => "worth-store-buffer-pool",
            Self::WorthStorePhysicalIntegrity => "worth-store-physical-integrity",
            Self::WorthStorePhysicalIsolation => "worth-store-physical-isolation",
            Self::WorthStoreIoScheduler => "worth-store-io-scheduler",
            Self::WorthStoreBlobChunks => "worth-store-blob-chunks",
            Self::WorthStoreSecurity => "worth-store-security",
            Self::WorthStoreOperations => "worth-store-operations",
            Self::WorthStoreMaintenance => "worth-store-maintenance",
            Self::WorthStoreRetention => "worth-store-retention",
            Self::WorthStoreTiering => "worth-store-tiering",
            Self::WorthStoreSnapshots => "worth-store-snapshots",
            Self::WorthStoreBranchDeltas => "worth-store-branch-deltas",
            Self::WorthStoreOfflineVerifier => "worth-store-offline-verifier",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableArtifactRebuildPosture {
    NoRebuild,
    RebuildFromAuthoritativeState,
    ReplayRebuildable,
    PartialRebuildOnly,
    QuarantineOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableArtifactMigrationPosture {
    StableNoMigration,
    VersionedReadmission,
    VersionedMigration,
    RollbackCapable,
    OfflineImportOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableArtifactProjectionClass {
    TerminalReport,
    OfflineObservation,
    CertificationEvidence,
    JsonExport,
    CounterSummary,
    TestFixture,
}

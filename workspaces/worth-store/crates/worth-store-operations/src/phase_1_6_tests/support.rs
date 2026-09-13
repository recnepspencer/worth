pub(super) use super::control_selection_provider::TestControlStoreFencingProvider;
pub(super) use crate::{
    admit_backup_for_production_restore, inspect_control_store_copies, qualify_backup_custody,
    record_independent_backup_verification, recover_online_backups, BackupMaterializationDenial,
    ConfiguredFailureDomainId, ControlStoreAvailabilityDenial, ControlStoreTrustPosture,
    OnlineBackupAdmissionDenial, OnlineBackupIntent, OnlineBackupReadmissionFailure,
    OperationalControlAppendDenial, OperationalControlLocation, OperationalControlRecord,
    OperationalControlReplayBudget, OperationalControlReplayResource, OperationalControlStore,
    OperationalControlStoreOpenDenial, OperationalControlStorePort, OperationalOperationId,
    OperationalTransitionId, OperationalWorkflowKind, ProtectedOperationalMediaLocation,
};
pub(super) use worth_store_authority::{
    BackupRestoreAdmissionPolicy, ControlStoreFencingAuthority, ControlStoreGeneration,
};
pub(super) use worth_store_offline_verifier::{
    verify_materialized_backup, BackupStructuralVerificationDenial,
    BackupVerificationAllocationPhase, BackupVerificationDefect, BackupVerificationReadAccounting,
    OfflineInspectionBudget, OfflineInspectionDenial,
};
pub(super) use worth_store_physical_backend::observe_physical_backup_artifact;
pub(super) use worth_store_physical_format::{
    BackupBundleArtifactFormat, BackupBundleFormatAuthority, BackupBundleManifest,
    PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalRecordSlot, PhysicalReferenceAuthority, PhysicalRootReference, PhysicalSegmentId,
    PhysicalStoreIdentity,
};
pub(super) use worth_store_physical_isolation::{
    BackupArtifactCoverage, BackupArtifactFamily, BackupArtifactReference,
    BackupCutAdmissionDenial, BackupCutCoordinates, BackupCutManifest, BackupCutManifestDenial,
    BackupCutReadmissionDenial, BackupReachabilityLeaseRegistry,
    CurrentGenerationPhysicalReference, ExecutedReachabilityEvidence,
    GenerationCountedPhysicalReference, HazardLeaseTable, HazardLeaseTableCapacity, ReclaimDenial,
    ReclaimEligibilityProof, UntrustedBackupArtifactClaim,
};

pub(crate) struct BackupScenario {
    pub(super) _directory: tempfile::TempDir,
    pub(super) source: std::path::PathBuf,
    pub(crate) target: std::path::PathBuf,
    pub(super) control: std::path::PathBuf,
    pub(super) references: Vec<BackupArtifactReference>,
    pub(crate) leases: BackupReachabilityLeaseRegistry,
    authority: worth_store_authority::StoreCurrentAuthorityWitness,
    checkpoint_identity: String,
    root_generation: u64,
}

impl BackupScenario {
    pub(crate) fn new(case: &str) -> Self {
        Self::at_root_generation(case, 1)
    }

    pub(super) fn at_root_generation(case: &str, root_generation: u64) -> Self {
        let directory = tempfile::tempdir().expect("temp directory");
        let source = directory.path().join("source");
        let target = directory.path().join("target");
        let control = directory.path().join("control").join("operations.log");
        std::fs::create_dir_all(&source).expect("source directory");
        std::fs::create_dir_all(&target).expect("target directory");
        let authority = crate::backup::export::current_authority("store.physical.default_instance");
        let store_identity =
            PhysicalStoreIdentity::from_aspect_identity(authority.identity().clone());
        let artifacts =
            crate::certification_scenario::backup_artifacts::canonical_backup_artifacts_at_root_generation(
                case,
                &source,
                root_generation,
                store_identity,
            );
        Self::from_artifacts(
            directory,
            source,
            target,
            control,
            artifacts,
            root_generation,
            authority,
        )
    }

    pub(super) fn paired_across_root_publication(case: &str) -> (Self, Self) {
        let older_directory = tempfile::tempdir().expect("older temp directory");
        let newer_directory = tempfile::tempdir().expect("newer temp directory");
        let older_source = older_directory.path().join("source");
        let newer_source = newer_directory.path().join("source");
        std::fs::create_dir_all(&older_source).expect("older source directory");
        std::fs::create_dir_all(&newer_source).expect("newer source directory");
        let authority = crate::backup::export::current_authority("store.physical.default_instance");
        let store_identity =
            PhysicalStoreIdentity::from_aspect_identity(authority.identity().clone());
        let (older_artifacts, newer_artifacts) =
            crate::certification_scenario::backup_artifacts::canonical_backup_artifacts_across_one_root_publication(
                case,
                &older_source,
                &newer_source,
                store_identity,
            );
        let older = Self::from_artifacts(
            older_directory,
            older_source,
            std::path::PathBuf::from("target"),
            std::path::PathBuf::from("control/operations.log"),
            older_artifacts,
            1,
            authority.clone(),
        );
        let newer = Self::from_artifacts(
            newer_directory,
            newer_source,
            std::path::PathBuf::from("target"),
            std::path::PathBuf::from("control/operations.log"),
            newer_artifacts,
            2,
            authority,
        );
        (older, newer)
    }

    fn from_artifacts(
        directory: tempfile::TempDir,
        source: std::path::PathBuf,
        target: std::path::PathBuf,
        control: std::path::PathBuf,
        artifacts: crate::certification_scenario::backup_artifacts::CanonicalBackupArtifacts,
        root_generation: u64,
        authority: worth_store_authority::StoreCurrentAuthorityWitness,
    ) -> Self {
        let target = if target.is_absolute() {
            target
        } else {
            directory.path().join(target)
        };
        let control = if control.is_absolute() {
            control
        } else {
            directory.path().join(control)
        };
        std::fs::create_dir_all(&target).expect("target directory");
        Self {
            _directory: directory,
            source,
            target,
            control,
            references: artifacts.references,
            leases: BackupReachabilityLeaseRegistry::for_store_runtime(),
            authority,
            checkpoint_identity: artifacts.checkpoint_identity,
            root_generation,
        }
    }
    pub(crate) fn control_store(&self) -> OperationalControlStore {
        OperationalControlStore::open_with_certified_topology(
            OperationalControlLocation::new(&self.control, failure_domain("control-media")),
            [
                ProtectedOperationalMediaLocation::source(
                    &self.source,
                    failure_domain("source-media"),
                ),
                ProtectedOperationalMediaLocation::backup_target(
                    &self.target,
                    failure_domain("target-media"),
                ),
            ],
        )
        .expect("independent control store")
    }
    pub(crate) fn source_root(&self) -> &std::path::Path {
        &self.source
    }
    pub(crate) fn authority(&self) -> worth_store_authority::StoreCurrentAuthorityWitness {
        self.authority.clone()
    }
    pub(crate) fn cut_manifest(&self) -> BackupCutManifest {
        BackupCutManifest::canonical(self.references.clone()).expect("complete canonical cut")
    }
    pub(crate) fn references(&self) -> &[BackupArtifactReference] {
        &self.references
    }
    pub(crate) fn total_bytes(&self) -> u64 {
        self.references
            .iter()
            .map(BackupArtifactReference::bytes)
            .sum()
    }
    pub(crate) fn coordinates(&self) -> BackupCutCoordinates {
        BackupCutCoordinates::new(
            "lineage-a",
            self.root_generation,
            1,
            &self.checkpoint_identity,
            10,
            10,
            12,
            12,
            "worth-physical-format-v1",
            "posix-file-fsync-dir-sync",
        )
        .expect("coherent cut coordinates")
    }
    pub(super) fn checkpoint_identity(&self) -> &str {
        &self.checkpoint_identity
    }
}

pub(super) const fn artifact_format(family: BackupArtifactFamily) -> BackupBundleArtifactFormat {
    match family {
        BackupArtifactFamily::RootManifest => BackupBundleArtifactFormat::PhysicalRootManifestV1,
        BackupArtifactFamily::CheckpointManifest => {
            BackupBundleArtifactFormat::RecoveryCheckpointManifestV1
        }
        BackupArtifactFamily::WalSegment => BackupBundleArtifactFormat::WalSegmentV1,
        BackupArtifactFamily::Page => BackupBundleArtifactFormat::PhysicalDataPageV1,
        BackupArtifactFamily::Extent => BackupBundleArtifactFormat::PhysicalExtentRecordV1,
        BackupArtifactFamily::Index => BackupBundleArtifactFormat::LayoutBTreeLeafV1,
        BackupArtifactFamily::BlobChunk => BackupBundleArtifactFormat::BlobChunkV1,
        BackupArtifactFamily::SecondaryRoot => {
            BackupBundleArtifactFormat::PhysicalSecondaryRootManifestV1
        }
    }
}

pub(super) fn artifact_coverage(family: BackupArtifactFamily) -> BackupArtifactCoverage {
    match family {
        BackupArtifactFamily::RootManifest => {
            BackupArtifactCoverage::root_manifest(1).expect("root coverage")
        }
        BackupArtifactFamily::CheckpointManifest => {
            BackupArtifactCoverage::checkpoint_manifest("checkpoint-a", 1, 10, [1; 32], [2; 32])
                .expect("checkpoint coverage")
        }
        BackupArtifactFamily::WalSegment => {
            BackupArtifactCoverage::wal_segment(10, 12).expect("WAL coverage")
        }
        BackupArtifactFamily::Page
        | BackupArtifactFamily::Extent
        | BackupArtifactFamily::Index
        | BackupArtifactFamily::BlobChunk => BackupArtifactCoverage::physical_reachability(),
        BackupArtifactFamily::SecondaryRoot => {
            BackupArtifactCoverage::secondary_root(1).expect("secondary root coverage")
        }
    }
}

pub(super) fn reclaim_reference(
    family: BackupArtifactFamily,
    coordinate: u16,
) -> CurrentGenerationPhysicalReference {
    let generation = PhysicalGeneration::from_raw(1).expect("generation");
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let counted = match family {
        BackupArtifactFamily::RootManifest | BackupArtifactFamily::SecondaryRoot => {
            let cell = generations
                .root_publication_cell(
                    PhysicalRootReference::from_raw(u64::from(coordinate)).expect("root"),
                )
                .with_root_publication_generation(generation);
            GenerationCountedPhysicalReference::from_admitted_reference(
                references.admit_root_publication(cell),
            )
        }
        BackupArtifactFamily::WalSegment => {
            let cell = generations
                .segment_cell(PhysicalSegmentId::from_raw(u64::from(coordinate)).expect("segment"))
                .with_segment_generation(generation);
            GenerationCountedPhysicalReference::from_segment_cell(cell)
        }
        BackupArtifactFamily::Extent | BackupArtifactFamily::BlobChunk => {
            let cell = generations
                .extent_cell(
                    PhysicalSegmentId::from_raw(100).expect("segment"),
                    PhysicalExtentId::from_raw(u64::from(coordinate)).expect("extent"),
                )
                .with_extent_generation(generation);
            GenerationCountedPhysicalReference::from_admitted_reference(
                references.admit_extent(cell),
            )
        }
        BackupArtifactFamily::CheckpointManifest | BackupArtifactFamily::Index => {
            let cell = generations
                .slot_cell(
                    PhysicalSegmentId::from_raw(200).expect("segment"),
                    PhysicalPageId::from_raw(1).expect("page"),
                    PhysicalRecordSlot::from_raw(coordinate).expect("slot"),
                )
                .with_slot_generation(generation);
            GenerationCountedPhysicalReference::from_admitted_reference(
                references.admit_page_slot(cell),
            )
        }
        BackupArtifactFamily::Page => {
            let cell = generations
                .page_cell(
                    PhysicalSegmentId::from_raw(200).expect("segment"),
                    PhysicalPageId::from_raw(u64::from(coordinate)).expect("page"),
                )
                .with_page_generation(generation);
            GenerationCountedPhysicalReference::from_page_cell(cell)
        }
    };
    counted
        .require_current_generation(generation)
        .expect("owner-issued current generation reference")
}

pub(super) fn failure_domain(value: &str) -> ConfiguredFailureDomainId {
    ConfiguredFailureDomainId::new(value).expect("failure domain")
}

pub(crate) use super::backup_custody_fixture::backup_custody;
pub(super) use super::control_fault_fixture::{FailingControlStore, ObserveReservedLeaseThenFail};

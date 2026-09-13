use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AbortedRuntime, AdmissionError, AdmittedPhysicalDurabilityPolicy,
    AdmittedRecordPlacementPolicy, CheckpointMemoryLimit, ClosedRuntime, FilesystemMediaAdmission,
    GroupCommitDelay, GroupCommitLimit, IdempotencyRetentionGenerations,
    LiveIdempotencyBindingLimit, MediaAdmissionInspectionCause, MediaOwnedPhysicalRuntime,
    PendingUnresolvedMutationLimit, PhysicalCheckpointPolicy, PhysicalDurabilityDeclaration,
    PhysicalDurabilityPolicyDenial, PhysicalIdempotencyPolicy, PhysicalRecordInitialization,
    PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy, RecordBootstrapDenial,
    RecordBootstrapFailure, RecordServingRebindReason, RecordServingStaleReason,
    RetainedWalTailLimit, ServingPhysicalRuntime, ServingShutdownOutcome, WalSegmentByteLimit,
    WalSegmentInventoryLimit,
};
use worth_store_physical_backend::{
    BackendCapabilityAdmissionDenial, FilesystemAccessPosture, MediaQualificationDeferred,
    MediaQualificationDenial, MediaQualificationRebindRequired, MediaQualificationStale,
};

use crate::TemporaryDirectory;

use super::configuration::{
    dense_recovery_planning_configuration, record_publication_configuration,
    recovery_planning_configuration, PhysicalResidencyStoreConfiguration,
};

#[derive(Debug)]
pub enum PhysicalResidencyStoreWorldConstructionFailure {
    Directory(std::io::Error),
    Runtime(AdmissionError),
    MediaDenied(MediaQualificationDenial),
    MediaDeferred(MediaQualificationDeferred),
    MediaStale(MediaQualificationStale),
    MediaRebindRequired(MediaQualificationRebindRequired),
    MediaInspectionRequired(MediaAdmissionInspectionCause),
    DurabilityBasis(BackendCapabilityAdmissionDenial),
    DurabilityPolicy(PhysicalDurabilityPolicyDenial),
    RecordDenied(RecordBootstrapDenial),
    RecordStale(RecordServingStaleReason),
    RecordRebindRequired(RecordServingRebindReason),
    RecordInspectionRequired(RecordBootstrapFailure),
}

pub struct PhysicalResidencyStoreWorld {
    root: TemporaryDirectory,
    serving: Option<ServingPhysicalRuntime>,
    pub(super) placement: AdmittedRecordPlacementPolicy,
}

impl PhysicalResidencyStoreWorld {
    pub fn initialize(label: &str) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        Self::initialize_with_configuration(
            label,
            record_publication_configuration(),
            NonZeroU64::new(16 * 1024 * 1024).unwrap(),
        )
    }

    pub fn initialize_for_recovery(
        label: &str,
    ) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        Self::initialize_with_configuration(
            label,
            recovery_planning_configuration(),
            NonZeroU64::new(16 * 1024 * 1024).unwrap(),
        )
    }

    pub fn initialize_for_recovery_with_segment_pages(
        label: &str,
        segment_pages: u32,
    ) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        Self::initialize_with_configuration(
            label,
            dense_recovery_planning_configuration(segment_pages),
            NonZeroU64::new(16 * 1024 * 1024).unwrap(),
        )
    }

    pub fn initialize_for_recovery_with_manifest_capacity(
        label: &str,
        manifest_capacity: u16,
    ) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        Self::initialize_with_configuration(
            label,
            super::configuration::compact_recovery_planning_configuration(1, manifest_capacity),
            NonZeroU64::new(16 * 1024 * 1024).unwrap(),
        )
    }

    pub fn initialize_for_recovery_with_wal_segment_bytes(
        label: &str,
        wal_segment_bytes: NonZeroU64,
    ) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        Self::initialize_with_configuration(
            label,
            recovery_planning_configuration(),
            wal_segment_bytes,
        )
    }

    fn initialize_with_configuration(
        label: &str,
        configuration: PhysicalResidencyStoreConfiguration,
        wal_segment_bytes: NonZeroU64,
    ) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        let root = TemporaryDirectory::create(label)
            .map_err(PhysicalResidencyStoreWorldConstructionFailure::Directory)?;
        let runtime = PhysicalStore::admit(
            PhysicalRuntimeAdmission::new(root.path())
                .map_err(PhysicalResidencyStoreWorldConstructionFailure::Runtime)?,
        )
        .map_err(PhysicalResidencyStoreWorldConstructionFailure::Runtime)?;
        let media = admit_media(runtime)?;
        let durability = admit_durability(&media, wal_segment_bytes)?;
        let request = PhysicalRecordInitialization::new(
            configuration.format,
            configuration.placement,
            configuration.access,
            durability,
        )
        .with_residency_policy(configuration.residency);
        let serving = admit_record_store(media.initialize_record_store(request))?;
        Ok(Self {
            root,
            serving: Some(serving),
            placement: configuration.placement,
        })
    }

    pub fn root(&self) -> &Path {
        self.root.path()
    }

    pub fn retained_root(&self) -> crate::TemporaryDirectory {
        self.root.clone()
    }

    pub fn serving(&self) -> &ServingPhysicalRuntime {
        self.serving
            .as_ref()
            .expect("a live fixture retains its serving runtime")
    }

    pub fn placement(&self) -> AdmittedRecordPlacementPolicy {
        self.placement
    }

    pub fn close(mut self) -> ServingShutdownOutcome<ClosedRuntime> {
        self.serving
            .take()
            .expect("a live fixture closes its serving runtime exactly once")
            .close()
    }
}

fn admit_durability(
    media: &MediaOwnedPhysicalRuntime,
    wal_segment_bytes: NonZeroU64,
) -> Result<AdmittedPhysicalDurabilityPolicy, PhysicalResidencyStoreWorldConstructionFailure> {
    let basis = media
        .physical_durability_admission_basis()
        .map_err(PhysicalResidencyStoreWorldConstructionFailure::DurabilityBasis)?;
    match PhysicalDurabilityDeclaration::builder()
        .group_commit(
            GroupCommitLimit::new(NonZeroU32::new(32).unwrap()),
            GroupCommitDelay::new(NonZeroU64::new(1).unwrap()),
        )
        .wal(PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(wal_segment_bytes),
            WalSegmentInventoryLimit::new(NonZeroU32::new(1_024).unwrap()),
        ))
        .idempotency(PhysicalIdempotencyPolicy::new(
            IdempotencyRetentionGenerations::new(NonZeroU64::new(4).unwrap()),
            PendingUnresolvedMutationLimit::new(NonZeroU32::new(1_024).unwrap()),
            LiveIdempotencyBindingLimit::new(NonZeroU32::new(4_096).unwrap()),
        ))
        .checkpoint(PhysicalCheckpointPolicy::fuzzy(
            CheckpointMemoryLimit::new(NonZeroU64::new(16 * 1024 * 1024).unwrap()),
            RetainedWalTailLimit::new(NonZeroU64::new(64 * 1024 * 1024).unwrap()),
        ))
        .admit(basis)
        .into_raw()
    {
        TransitionOutcome::Success(policy) => Ok(policy),
        TransitionOutcome::Denied(denial) => {
            Err(PhysicalResidencyStoreWorldConstructionFailure::DurabilityPolicy(denial))
        }
        TransitionOutcome::Deferred(deferred) => match deferred {},
        TransitionOutcome::Stale(stale) => match stale {},
        TransitionOutcome::RebindRequired(rebind) => match rebind {},
        TransitionOutcome::Failed(failure) => match failure {},
    }
}

impl Drop for PhysicalResidencyStoreWorld {
    fn drop(&mut self) {
        if let Some(serving) = self.serving.take() {
            let _: ServingShutdownOutcome<AbortedRuntime> = serving.abort();
        }
    }
}

fn admit_media(
    runtime: worth_store::physical_runtime::AdmittedPhysicalRuntime,
) -> Result<
    worth_store::physical_runtime::MediaOwnedPhysicalRuntime,
    PhysicalResidencyStoreWorldConstructionFailure,
> {
    match runtime
        .try_admit_filesystem_media(FilesystemMediaAdmission::production(
            FilesystemAccessPosture::CoordinatedServiceAccount,
        ))
        .into_raw()
    {
        TransitionOutcome::Success(media) => Ok(media),
        TransitionOutcome::Denied(denial) => {
            let reason = denial.reason().clone();
            denial.into_runtime().close();
            Err(PhysicalResidencyStoreWorldConstructionFailure::MediaDenied(
                reason,
            ))
        }
        TransitionOutcome::Deferred(deferred) => {
            let reason = deferred.reason();
            deferred.into_runtime().close();
            Err(PhysicalResidencyStoreWorldConstructionFailure::MediaDeferred(reason))
        }
        TransitionOutcome::Stale(stale) => {
            let reason = stale.reason();
            stale.into_runtime().close();
            Err(PhysicalResidencyStoreWorldConstructionFailure::MediaStale(
                reason,
            ))
        }
        TransitionOutcome::RebindRequired(rebind) => {
            let reason = rebind.reason();
            rebind.into_runtime().close();
            Err(PhysicalResidencyStoreWorldConstructionFailure::MediaRebindRequired(reason))
        }
        TransitionOutcome::Failed(inspection) => Err(
            PhysicalResidencyStoreWorldConstructionFailure::MediaInspectionRequired(
                match inspection.cause() {
                    MediaAdmissionInspectionCause::BackendFailure(failure) => {
                        MediaAdmissionInspectionCause::BackendFailure(failure.clone())
                    }
                },
            ),
        ),
    }
}

fn admit_record_store(
    outcome: worth_store::physical_runtime::RecordStoreInitializationOutcome,
) -> Result<ServingPhysicalRuntime, PhysicalResidencyStoreWorldConstructionFailure> {
    match outcome.into_raw() {
        TransitionOutcome::Success(serving) => Ok(serving),
        TransitionOutcome::Denied(denial) => {
            let reason = denial.reason();
            denial.into_runtime().close();
            Err(PhysicalResidencyStoreWorldConstructionFailure::RecordDenied(reason))
        }
        TransitionOutcome::Deferred(deferred) => match deferred {},
        TransitionOutcome::Stale(stale) => {
            let reason = stale.reason();
            stale.into_runtime().close();
            Err(PhysicalResidencyStoreWorldConstructionFailure::RecordStale(
                reason,
            ))
        }
        TransitionOutcome::RebindRequired(rebind) => {
            let reason = rebind.reason();
            rebind.into_runtime().close();
            Err(PhysicalResidencyStoreWorldConstructionFailure::RecordRebindRequired(reason))
        }
        TransitionOutcome::Failed(inspection) => Err(
            PhysicalResidencyStoreWorldConstructionFailure::RecordInspectionRequired(
                inspection.cause(),
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use worth_store::physical_runtime::PhysicalOperationAllocationScope as Scope;

    use super::PhysicalResidencyStoreWorld;
    use crate::harness::physical_residency::{
        configuration::successor_scope_pressure_configuration, SUCCESSOR_SCOPE_ALLOCATION_BYTES,
    };

    #[test]
    fn real_store_world_mints_and_releases_every_exact_successor_scope() {
        let world = PhysicalResidencyStoreWorld::initialize_with_configuration(
            "physical-residency-scopes",
            successor_scope_pressure_configuration(),
            NonZeroU64::new(16 * 1024 * 1024).unwrap(),
        )
        .unwrap();
        assert!(world.root().is_dir());
        let serving = world.serving();
        let allocations = serving.physical_allocations();
        let bytes = NonZeroU64::new(SUCCESSOR_SCOPE_ALLOCATION_BYTES).unwrap();
        let recovery = allocations.admit_recovery(bytes).unwrap();
        let scrub = allocations.admit_scrub(bytes).unwrap();
        let maintenance = allocations.admit_maintenance(bytes).unwrap();
        let verification = allocations.admit_verification(bytes).unwrap();
        let blob = allocations.admit_blob(bytes).unwrap();
        for runtime in [
            recovery.runtime_identity(),
            scrub.runtime_identity(),
            maintenance.runtime_identity(),
            verification.runtime_identity(),
            blob.runtime_identity(),
        ] {
            assert_eq!(runtime, serving.runtime_identity());
        }
        for active_scope in [
            Scope::Recovery,
            Scope::Scrub,
            Scope::Maintenance,
            Scope::Verification,
            Scope::Blob,
        ] {
            assert_eq!(
                serving
                    .residency_observation()
                    .counters()
                    .active_operation_bytes_for(active_scope),
                SUCCESSOR_SCOPE_ALLOCATION_BYTES,
            );
        }
        let pressure = allocations
            .admit_recovery(bytes)
            .expect_err("the five live successor allocations exhaust the global envelope")
            .pressure()
            .expect("global operation exhaustion is Store pressure");
        assert_eq!(
            pressure.dimension(),
            worth_store::physical_runtime::PhysicalResidencyDimension::OperationBytes,
        );
        assert_eq!(pressure.admitted(), SUCCESSOR_SCOPE_ALLOCATION_BYTES * 5,);
        drop((recovery, scrub, maintenance, verification, blob));
        assert_eq!(
            serving
                .residency_observation()
                .counters()
                .active_operation_bytes(),
            0,
        );
        assert!(!world.close().residency().requires_inspection());
    }
}

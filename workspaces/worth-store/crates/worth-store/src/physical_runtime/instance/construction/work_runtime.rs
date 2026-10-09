use std::sync::Arc;

use worth_store_physical_backend::QualifiedFilesystemMedia;

use crate::physical_runtime::record_serving::{RecordWorkAdmission, ServingHealth};
use crate::physical_runtime::runtime::PhysicalRuntimeCore;
use crate::physical_runtime::work::{
    PhysicalWorkAdmissionAuthority, PhysicalWorkSubmissionFoundation, PhysicalWorkSubmissionOwner,
};
use crate::physical_runtime::{
    PhysicalDurabilityObservation, PhysicalSignalProfileIdentity, PhysicalWorkProfileDeclaration,
};

use super::super::{
    PhysicalSchedulerAdmissionOwner, PhysicalSignalConstructionFailure, PhysicalStoreWorkRuntime,
    PhysicalWorkExecutor, PhysicalWorkSignalOwner,
};

pub(super) struct PreparedPhysicalWorkRuntime {
    record_work: Arc<RecordWorkAdmission>,
    scheduler: PhysicalSchedulerAdmissionOwner,
    admission: PhysicalWorkAdmissionAuthority,
    submission: PhysicalWorkSubmissionOwner,
    signal: PhysicalWorkSignalOwner,
    health: ServingHealth,
    recovery: crate::physical_runtime::work::PhysicalEffectRecoveryInventory,
}

pub(super) struct InstalledPhysicalWorkRuntime {
    pub(super) runtime: Arc<PhysicalStoreWorkRuntime>,
    pub(super) record_work: Arc<RecordWorkAdmission>,
    pub(super) scheduler: PhysicalSchedulerAdmissionOwner,
    pub(super) admission: PhysicalWorkAdmissionAuthority,
}

pub(super) fn prepare_work_runtime(
    media: &QualifiedFilesystemMedia,
    core: &PhysicalRuntimeCore,
    work_profile: PhysicalWorkProfileDeclaration,
    durability: PhysicalDurabilityObservation,
) -> Result<PreparedPhysicalWorkRuntime, PhysicalSignalConstructionFailure> {
    let (record_work, work_profile) = RecordWorkAdmission::install(work_profile, durability)
        .map_err(PhysicalSignalConstructionFailure::ProfileRejected)?;
    let work_capacity = work_profile.capacity();
    let recovery = PhysicalWorkExecutor::inspect_recovery(media, work_capacity.commands());
    let health = ServingHealth::new(recovery.requires_inspection());
    let lifecycle = core.lifecycle_generation();
    let signal = PhysicalWorkSignalOwner::build_foundation(lifecycle, work_profile)?;
    let submission = PhysicalWorkSubmissionOwner::new(PhysicalWorkSubmissionFoundation {
        store: media.store_identity(),
        runtime: core.runtime_identity(),
        generation: lifecycle,
        lifecycle: core.lifecycle_state(),
        lifecycle_phase: crate::physical_runtime::lifecycle::ObservedLifecyclePhase::RecordServing,
        signal_profile: signal.profile(),
        bindings: signal.bindings(),
        signal_admission: signal.admission_status(),
        abandonment: signal.abandonment_publisher(),
    });
    let scheduler = PhysicalSchedulerAdmissionOwner::new(media, work_capacity)
        .map_err(PhysicalSignalConstructionFailure::SchedulerCapabilityRejected)?;
    let admission = PhysicalWorkAdmissionAuthority::from_qualified_instance(
        media,
        core.runtime_identity(),
        lifecycle,
    );
    Ok(PreparedPhysicalWorkRuntime {
        record_work: Arc::new(record_work),
        scheduler,
        admission,
        submission,
        signal,
        health,
        recovery,
    })
}

impl PreparedPhysicalWorkRuntime {
    pub(super) fn admit_publication_residue(
        &self,
        admission: super::retirement_residue::PublicationResidueAdmission,
    ) {
        if matches!(
            admission,
            super::retirement_residue::PublicationResidueAdmission::InspectionRequired
        ) {
            self.health.revoke();
        }
    }
    pub(super) fn signal_profile(&self) -> PhysicalSignalProfileIdentity {
        self.signal.profile()
    }

    pub(super) fn install(self, media: QualifiedFilesystemMedia) -> InstalledPhysicalWorkRuntime {
        InstalledPhysicalWorkRuntime {
            runtime: PhysicalStoreWorkRuntime::new(
                self.submission,
                self.signal,
                PhysicalWorkExecutor::new(media),
                self.health,
                self.recovery,
            ),
            record_work: self.record_work,
            scheduler: self.scheduler,
            admission: self.admission,
        }
    }
}

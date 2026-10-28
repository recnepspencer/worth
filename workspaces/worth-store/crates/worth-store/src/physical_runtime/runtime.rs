use super::{
    availability::InstalledCapabilityStatus,
    diagnostics::RuntimeDiagnostics,
    observation::ObservationHandle,
    resource_lifecycle::ResourceLifecycle,
    root_admission::RootAdmission,
    shutdown::{AbortedRuntime, ClosedRuntime, ShutdownCoordinator},
    DeclaredStoreRoot, PhysicalCapability, RuntimeCounterSnapshot, RuntimeIdentity,
};

/// Sole move-only owner of the responsibilities installed at admission.
pub struct AdmittedPhysicalRuntime {
    core: PhysicalRuntimeCore,
}

pub(super) struct PhysicalRuntimeCore {
    runtime_identity: RuntimeIdentity,
    resource_lifecycle: ResourceLifecycle,
    diagnostics: RuntimeDiagnostics,
    shutdown: ShutdownCoordinator,
}

impl AdmittedPhysicalRuntime {
    pub(crate) fn from_admission(
        runtime_identity: RuntimeIdentity,
        root_admission: RootAdmission,
    ) -> Self {
        let diagnostics = RuntimeDiagnostics::admitted(runtime_identity);
        let resource_lifecycle = ResourceLifecycle::new(diagnostics.counter_cells());
        let shutdown = ShutdownCoordinator::admitted(root_admission, diagnostics.counter_cells());
        Self {
            core: PhysicalRuntimeCore {
                runtime_identity,
                resource_lifecycle,
                diagnostics,
                shutdown,
            },
        }
    }

    pub const fn runtime_identity(&self) -> RuntimeIdentity {
        self.core.runtime_identity
    }

    pub fn declared_store_root(&self) -> &DeclaredStoreRoot {
        self.core.shutdown.declared_root()
    }

    pub fn observe(&self) -> ObservationHandle {
        ObservationHandle::new(
            self.core.runtime_identity,
            self.core.shutdown.lifecycle_state(),
            self.core.resource_lifecycle.acquire_observation(),
        )
    }

    pub fn installed_capabilities(&self) -> InstalledCapabilityStatus {
        self.core
            .diagnostics
            .record_capability_observations(PhysicalCapability::FAMILY_COUNT);
        InstalledCapabilityStatus::c3()
    }

    pub fn counters(&self) -> RuntimeCounterSnapshot {
        let lifecycle = self.core.shutdown.lifecycle_snapshot();
        self.core.diagnostics.snapshot(lifecycle.generation)
    }

    pub fn close(self) -> ClosedRuntime {
        self.core.close()
    }

    pub fn abort(self) -> AbortedRuntime {
        self.core.abort()
    }

    pub fn try_admit_filesystem_media(
        self,
        admission: super::FilesystemMediaAdmission,
    ) -> super::MediaAdmissionOutcome {
        super::media_ownership::try_admit(self, admission)
    }

    pub(super) fn into_core(self) -> PhysicalRuntimeCore {
        self.core
    }

    pub(super) const fn from_core(core: PhysicalRuntimeCore) -> Self {
        Self { core }
    }
}

impl PhysicalRuntimeCore {
    pub(super) const fn runtime_identity(&self) -> RuntimeIdentity {
        self.runtime_identity
    }

    pub(super) fn lifecycle_generation(&self) -> super::LifecycleGeneration {
        self.shutdown.lifecycle_snapshot().generation
    }

    pub(super) fn lifecycle_state(&self) -> std::sync::Arc<super::lifecycle::LifecycleState> {
        self.shutdown.lifecycle_state()
    }

    pub(super) fn declared_store_root(&self) -> &DeclaredStoreRoot {
        self.shutdown.declared_root()
    }

    pub(super) fn progress_to_media_owned(&self) {
        self.shutdown.progress_to_media_owned();
    }

    pub(super) fn progress_to_record_serving(&self) {
        self.shutdown.progress_to_record_serving();
    }

    pub(super) fn termination_guard(&self) -> super::lifecycle::LifecycleTerminationGuard {
        super::lifecycle::LifecycleTerminationGuard::new(self.shutdown.lifecycle_state())
    }

    pub(super) fn media_observation_parts(
        &self,
    ) -> (
        std::sync::Arc<super::lifecycle::LifecycleState>,
        super::resource_lifecycle::ObservationLease,
    ) {
        (
            self.shutdown.lifecycle_state(),
            self.resource_lifecycle.acquire_observation(),
        )
    }

    pub(super) fn close(self) -> ClosedRuntime {
        let Self {
            runtime_identity,
            resource_lifecycle,
            diagnostics,
            shutdown,
        } = self;
        let closed = shutdown.close(runtime_identity);
        drop((resource_lifecycle, diagnostics));
        closed
    }

    pub(super) fn abort(self) -> AbortedRuntime {
        let Self {
            runtime_identity,
            resource_lifecycle,
            diagnostics,
            shutdown,
        } = self;
        let aborted = shutdown.abort(runtime_identity);
        drop((resource_lifecycle, diagnostics));
        aborted
    }
}

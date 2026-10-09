use std::num::NonZeroU64;

#[cfg(feature = "certification-test-authority")]
mod certification;
#[cfg(feature = "certification-test-authority")]
mod certification_faults;
mod resident_admission;
mod signal_binding;
pub use resident_admission::PhysicalRecoveryRejoinResidentAdmissionDenial;
use resident_admission::RecoveryRejoinResidentState;

#[cfg(feature = "certification-test-authority")]
use certification_faults::RecoveryCoordinationCertificationFaults;

use crate::physical_runtime::work::{
    PhysicalWorkAdmissionAuthority, PhysicalWorkSubmissionFoundation, PhysicalWorkSubmissionOwner,
};
use crate::physical_runtime::{
    instance::{PhysicalSchedulerAdmissionOwner, PhysicalWorkSignalOwner},
    AdmittedRecoveryFilesystemMedia, LifecycleGeneration,
    PhysicalRecoveryRegisteredSessionAuthority, PhysicalSignalShutdownOutcome, RuntimeIdentity,
};

use super::{semantics, PhysicalRecoveryCoordinationCapacity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryCoordinationAdmissionError {
    Residency(worth_store_buffer_pool::PhysicalResidencyDenial),
    FreshnessMediaMismatch,
    SignalUnavailable,
    SignalBindingMismatch,
    SchedulerUnavailable,
    DiscoveryReservationDenied,
    RuntimeIdentityUnavailable,
}

pub struct PhysicalRecoveryCoordination {
    pub(super) residency: crate::physical_runtime::instance::PhysicalResidencyOwner,
    pub(super) store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    pub(super) media_generation: worth_store_physical_backend::PhysicalRecoveryMediaGeneration,
    _registered_session: PhysicalRecoveryRegisteredSessionAuthority,
    pub(super) signal: PhysicalWorkSignalOwner,
    pub(super) submission: PhysicalWorkSubmissionOwner,
    pub(super) admission: PhysicalWorkAdmissionAuthority,
    pub(super) scheduler: PhysicalSchedulerAdmissionOwner,
    pub(super) scheduler_security: worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    pub(super) work_security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
    pub(super) bases: [crate::physical_runtime::PhysicalWorkSemanticBasis; 5],
    pub(super) construction: crate::physical_runtime::PhysicalRecoveryConstructionAuthority,
    pub(super) cleanup_capacity: PhysicalRecoveryCoordinationCapacity,
    recovery_allocation: Option<crate::physical_runtime::PhysicalRecoveryAllocationAdmission>,
    rejoin_resident: RecoveryRejoinResidentState,
    pub(super) root_protocol_counters: crate::physical_runtime::RootProtocolRouteCounterCells,
    pub(super) checkpoint_selection: super::selected_checkpoint::RecoveryCheckpointSelection,
    runtime: RuntimeIdentity,
    yieldpoint: Option<crate::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
    #[cfg(feature = "certification-test-authority")]
    certification_faults: RecoveryCoordinationCertificationFaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryQuiescenceObservation {
    live_commands: u64,
    live_scheduler_reservations: u64,
    pending_signal_reconciliations: u64,
    signal_reconciliation_overflow: u64,
    signal_available: bool,
}

impl PhysicalRecoveryCoordination {
    pub(super) fn admit(
        media: &AdmittedRecoveryFilesystemMedia,
        session: PhysicalRecoveryRegisteredSessionAuthority,
        capacity: PhysicalRecoveryCoordinationCapacity,
        policy: crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy,
        yieldpoint: Option<crate::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
    ) -> Result<Self, PhysicalRecoveryCoordinationAdmissionError> {
        let freshness = session.freshness();
        if !freshness.matches_media_generation(media.media_generation()) {
            return Err(PhysicalRecoveryCoordinationAdmissionError::FreshnessMediaMismatch);
        }
        let mut residency = crate::physical_runtime::instance::PhysicalResidencyOwner::admit(
            media.store_identity(),
            policy,
        )
        .map_err(PhysicalRecoveryCoordinationAdmissionError::Residency)?;
        let cleanup_capacity = capacity;
        let recovery_allocation = capacity.recovery_allocation_bytes().map(|bytes| {
            crate::physical_runtime::PhysicalRecoveryAllocationAdmission::new(
                media.store_identity(),
                bytes,
            )
        });
        if let Some(allocation) = recovery_allocation {
            residency
                .restrict_recovery_allocation(
                    allocation,
                    lifecycle_from(session.session_identity_bytes()),
                )
                .map_err(PhysicalRecoveryCoordinationAdmissionError::Residency)?;
        }
        let capacity = capacity.work_capacity();
        let semantics = semantics::install(
            media.store_identity(),
            session.session_identity_bytes(),
            capacity,
        );
        let lifecycle = lifecycle_from(session.session_identity_bytes());
        let construction = crate::physical_runtime::PhysicalRecoveryConstructionAuthority::issue(
            media.store_identity(),
            media.media_generation(),
            session.session_identity_bytes(),
        );
        let runtime = RuntimeIdentity::generate()
            .ok_or(PhysicalRecoveryCoordinationAdmissionError::RuntimeIdentityUnavailable)?;
        let signal = PhysicalWorkSignalOwner::build_foundation(lifecycle, semantics.profile)
            .map_err(|_| PhysicalRecoveryCoordinationAdmissionError::SignalUnavailable)?;
        if !signal_binding::bindings_match(
            &signal,
            media.store_identity(),
            session.session_identity_bytes(),
        ) {
            return Err(PhysicalRecoveryCoordinationAdmissionError::SignalBindingMismatch);
        }
        let scheduler = PhysicalSchedulerAdmissionOwner::new_recovery(media, capacity)
            .map_err(|_| PhysicalRecoveryCoordinationAdmissionError::SchedulerUnavailable)?;
        let (reservation, _) = scheduler
            .record_metadata(&semantics.scheduler_security)
            .map_err(|_| PhysicalRecoveryCoordinationAdmissionError::DiscoveryReservationDenied)?;
        drop(reservation);
        let submission = PhysicalWorkSubmissionOwner::new(PhysicalWorkSubmissionFoundation {
            store: media.store_identity(),
            runtime,
            generation: lifecycle,
            lifecycle: crate::physical_runtime::lifecycle::LifecycleState::recovery_active(
                lifecycle,
            ),
            lifecycle_phase: crate::physical_runtime::lifecycle::ObservedLifecyclePhase::MediaOwned,
            signal_profile: signal.profile(),
            bindings: signal.bindings(),
            signal_admission: signal.admission_status(),
            abandonment: signal.abandonment_publisher(),
        });
        let admission =
            PhysicalWorkAdmissionAuthority::from_recovery_media(media, runtime, lifecycle);
        Ok(Self {
            residency,
            store: media.store_identity(),
            media_generation: media.media_generation(),
            _registered_session: session,
            signal,
            submission,
            admission,
            scheduler,
            scheduler_security: semantics.scheduler_security,
            work_security: semantics.work_security,
            bases: semantics.bases,
            construction,
            cleanup_capacity,
            recovery_allocation,
            rejoin_resident: RecoveryRejoinResidentState::Unadmitted,
            root_protocol_counters: crate::physical_runtime::RootProtocolRouteCounterCells::default(
            ),
            checkpoint_selection:
                super::selected_checkpoint::RecoveryCheckpointSelection::Unobserved,
            runtime,
            yieldpoint,
            #[cfg(feature = "certification-test-authority")]
            certification_faults: RecoveryCoordinationCertificationFaults::new(),
        })
    }

    pub fn is_ready(&self) -> bool {
        let observation = self.quiescence_observation();
        let capacity = self.scheduler.capacity_snapshot();
        observation.signal_available
            && observation.live_commands == 0
            && observation.live_scheduler_reservations == 0
            && observation.pending_signal_reconciliations == 0
            && observation.signal_reconciliation_overflow == 0
            && capacity.admitted_reservations() >= 1
            && capacity.admitted_reservations() == capacity.released_reservations()
            && capacity.denied_reservations() == 0
            && capacity.active_reservations() == 0
            && capacity.available() == capacity.configured()
    }

    pub(in crate::physical_runtime) const fn cleanup_capacity(
        &self,
    ) -> PhysicalRecoveryCoordinationCapacity {
        self.cleanup_capacity
    }

    /// The immutable recovery resource admission bound to this coordination owner.
    pub const fn recovery_allocation_admission(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalRecoveryAllocationAdmission> {
        self.recovery_allocation
    }

    pub fn root_protocol_counters(&self) -> crate::physical_runtime::RootProtocolRouteCounters {
        self.root_protocol_counters.snapshot()
    }

    pub fn quiescence_observation(&self) -> PhysicalRecoveryQuiescenceObservation {
        let work = self.submission.counters();
        let live_commands = [
            crate::physical_runtime::PhysicalWorkCounterStage::Blocked,
            crate::physical_runtime::PhysicalWorkCounterStage::Ready,
            crate::physical_runtime::PhysicalWorkCounterStage::Queued,
            crate::physical_runtime::PhysicalWorkCounterStage::Dispatched,
            crate::physical_runtime::PhysicalWorkCounterStage::Settling,
        ]
        .into_iter()
        .map(|stage| work.total(stage))
        .sum();
        let (pending, overflow) = self.signal.reconciliation_counts();
        PhysicalRecoveryQuiescenceObservation::quiescence(
            live_commands,
            self.scheduler.capacity_snapshot().active_reservations(),
            pending as u64,
            overflow,
            self.signal.admission_status().is_available(),
        )
    }

    pub(in crate::physical_runtime) fn reconcile_signal_settlements(&self) -> u64 {
        self.signal
            .reconcile_settlements()
            .into_iter()
            .map(|(identity, outcome)| {
                self.submission
                    .record_reconciled_derived_completion(identity, outcome);
                1_u64
            })
            .sum()
    }

    pub fn shutdown_is_quiescent(self) -> bool {
        self.into_quiescent_parts().is_some()
    }

    pub(in crate::physical_runtime) fn into_quiescent_recovery_parts(
        self,
    ) -> Option<(
        crate::physical_runtime::instance::PhysicalResidencyOwner,
        super::selected_checkpoint::RecoveryCheckpointOwnership,
    )> {
        let (residency, checkpoint) = self.into_quiescent_parts()?;
        Some((residency, checkpoint.into_ownership()?))
    }

    fn into_quiescent_parts(
        self,
    ) -> Option<(
        crate::physical_runtime::instance::PhysicalResidencyOwner,
        super::selected_checkpoint::RecoveryCheckpointSelection,
    )> {
        let _ = self.reconcile_signal_settlements();
        if !self.is_ready() {
            return None;
        }
        let work = self
            .submission
            .stop(crate::physical_runtime::work::PhysicalWorkStopKind::Close);
        let quiescent = work.residual() == 0
            && work.unaccounted_terminal() == 0
            && work.ready() == 0
            && work.blocked() == 0
            && work.queued() == 0
            && work.dispatched() == 0
            && work.settling() == 0
            && self.signal.dispose() == PhysicalSignalShutdownOutcome::Disposed
            && self.scheduler.capacity_snapshot().active_reservations() == 0;
        quiescent.then_some((self.residency, self.checkpoint_selection))
    }

    pub(in crate::physical_runtime) const fn freshness(
        &self,
    ) -> &crate::physical_runtime::PhysicalRecoveryFreshnessAuthority {
        self._registered_session.freshness()
    }

    pub(in crate::physical_runtime) fn session_identity(&self) -> [u8; 16] {
        self._registered_session.session_identity_bytes()
    }

    pub(in crate::physical_runtime) const fn runtime_identity(&self) -> RuntimeIdentity {
        self.runtime
    }

    pub fn pause_at(
        &self,
        stage: crate::physical_runtime::PhysicalRecoveryYieldpointStage,
    ) -> crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult {
        if let Some(yieldpoint) = &self.yieldpoint {
            return yieldpoint.pause_after(stage);
        }
        crate::physical_runtime::PhysicalRecoveryYieldpointWaitResult::NotArmed
    }

    pub(in crate::physical_runtime) const fn construction_authority(
        &self,
    ) -> &crate::physical_runtime::PhysicalRecoveryConstructionAuthority {
        &self.construction
    }
}

impl PhysicalRecoveryQuiescenceObservation {
    const fn quiescence(
        live_commands: u64,
        live_scheduler_reservations: u64,
        pending_signal_reconciliations: u64,
        signal_reconciliation_overflow: u64,
        signal_available: bool,
    ) -> Self {
        Self {
            live_commands,
            live_scheduler_reservations,
            pending_signal_reconciliations,
            signal_reconciliation_overflow,
            signal_available,
        }
    }
    pub const fn live_commands(self) -> u64 {
        self.live_commands
    }
    pub const fn live_scheduler_reservations(self) -> u64 {
        self.live_scheduler_reservations
    }
    pub const fn pending_signal_reconciliations(self) -> u64 {
        self.pending_signal_reconciliations
    }
    pub const fn signal_reconciliation_overflow(self) -> u64 {
        self.signal_reconciliation_overflow
    }
    pub const fn signal_available(self) -> bool {
        self.signal_available
    }
}

impl PhysicalRecoveryRegisteredSessionAuthority {
    /// Uses this registered Store session to admit the matching native recovery
    /// coordination owner after persisted Store admission.
    pub fn admit_coordination(
        self,
        media: &AdmittedRecoveryFilesystemMedia,
        capacity: PhysicalRecoveryCoordinationCapacity,
        policy: crate::physical_runtime::record_serving::AdmittedPhysicalRecordResidencyPolicy,
        yieldpoint: Option<crate::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
    ) -> Result<PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationAdmissionError> {
        PhysicalRecoveryCoordination::admit(media, self, capacity, policy, yieldpoint)
    }
}

fn lifecycle_from(session: [u8; 16]) -> LifecycleGeneration {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&session[..8]);
    // Lifecycle encoding reserves three low state bits after shifting the
    // generation. Bound session-derived generations to the representable
    // domain before constructing the lifecycle identity.
    let generation = u64::from_le_bytes(bytes) & (u64::MAX >> 3);
    let generation = NonZeroU64::new(generation).unwrap_or(NonZeroU64::MIN);
    LifecycleGeneration::from_reopened(generation)
}

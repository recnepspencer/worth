use std::num::NonZeroU64;

#[cfg(feature = "certification-test-authority")]
mod certification;
#[cfg(feature = "certification-test-authority")]
mod certification_faults;

#[cfg(feature = "certification-test-authority")]
use certification_faults::RecoveryCoordinationCertificationFaults;

use crate::physical_runtime::work::{
    PhysicalWorkAdmissionAuthority, PhysicalWorkSubmissionFoundation, PhysicalWorkSubmissionOwner,
};
use crate::physical_runtime::{
    instance::{PhysicalSchedulerAdmissionOwner, PhysicalWorkSignalOwner},
    AdmittedRecoveryFilesystemMedia, LifecycleGeneration,
    PhysicalRecoveryRegisteredSessionAuthority, PhysicalSignalAspectRole,
    PhysicalSignalShutdownOutcome, PhysicalWorkSignalFamily, PhysicalWorkSignalFamilySet,
    RuntimeIdentity,
};

use super::{semantics, PhysicalRecoveryCoordinationCapacity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryCoordinationAdmissionError {
    FreshnessMediaMismatch,
    SignalUnavailable,
    SignalBindingMismatch,
    SchedulerUnavailable,
    DiscoveryReservationDenied,
    RuntimeIdentityUnavailable,
}

pub struct PhysicalRecoveryCoordination {
    pub(super) store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    pub(super) media_generation: worth_store_physical_backend::PhysicalRecoveryMediaGeneration,
    _registered_session: PhysicalRecoveryRegisteredSessionAuthority,
    pub(super) signal: PhysicalWorkSignalOwner,
    pub(super) submission: PhysicalWorkSubmissionOwner,
    pub(super) admission: PhysicalWorkAdmissionAuthority,
    pub(super) scheduler: PhysicalSchedulerAdmissionOwner,
    pub(super) scheduler_security: worth_store_io_scheduler::IoSchedulerSecurityScopeAdmission,
    pub(super) work_security: worth_store_security::StoreAuthorityBoundSecurityScopeReceipt,
    pub(super) bases: [crate::physical_runtime::PhysicalWorkSemanticBasis; 4],
    pub(super) construction: crate::physical_runtime::PhysicalRecoveryConstructionAuthority,
    pub(super) cleanup_capacity: PhysicalRecoveryCoordinationCapacity,
    pub(super) root_protocol_counters: crate::physical_runtime::RootProtocolRouteCounterCells,
    checkpoint_binding_basis: Option<crate::physical_runtime::StoreRecoveryCheckpointBindingBasis>,
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
        yieldpoint: Option<crate::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
    ) -> Result<Self, PhysicalRecoveryCoordinationAdmissionError> {
        let freshness = session.freshness();
        if !freshness.matches_media_generation(media.media_generation()) {
            return Err(PhysicalRecoveryCoordinationAdmissionError::FreshnessMediaMismatch);
        }
        let cleanup_capacity = capacity;
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
        if !bindings_match(
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
            root_protocol_counters: crate::physical_runtime::RootProtocolRouteCounterCells::default(
            ),
            checkpoint_binding_basis: None,
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

    pub fn root_protocol_counters(&self) -> crate::physical_runtime::RootProtocolRouteCounters {
        self.root_protocol_counters.snapshot()
    }

    pub fn install_checkpoint_binding_basis(
        &mut self,
        basis: crate::physical_runtime::StoreRecoveryCheckpointBindingBasis,
    ) -> bool {
        if self.checkpoint_binding_basis.is_some() {
            return false;
        }
        self.checkpoint_binding_basis = Some(basis);
        true
    }

    pub(in crate::physical_runtime) fn checkpoint_binding_basis(
        &self,
    ) -> Option<&crate::physical_runtime::StoreRecoveryCheckpointBindingBasis> {
        self.checkpoint_binding_basis.as_ref()
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
        let _ = self.reconcile_signal_settlements();
        if !self.is_ready() {
            return false;
        }
        let work = self
            .submission
            .stop(crate::physical_runtime::work::PhysicalWorkStopKind::Close);
        work.residual() == 0
            && work.unaccounted_terminal() == 0
            && work.ready() == 0
            && work.blocked() == 0
            && work.queued() == 0
            && work.dispatched() == 0
            && work.settling() == 0
            && self.signal.dispose() == PhysicalSignalShutdownOutcome::Disposed
            && self.scheduler.capacity_snapshot().active_reservations() == 0
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

fn bindings_match(
    signal: &PhysicalWorkSignalOwner,
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    session: [u8; 16],
) -> bool {
    let observations = signal.binding_observations();
    let expected = [
        (
            "store.physical.recovery.discovery-basis",
            PhysicalSignalAspectRole::Dependency,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::ReadFault),
            expected_partition(store, session, "discovery"),
        ),
        (
            "store.physical.recovery.redo-basis",
            PhysicalSignalAspectRole::DependencyAndOutput,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::ExactWriteback)
                .with(PhysicalWorkSignalFamily::Publication),
            expected_partition(store, session, "redo"),
        ),
        (
            "store.physical.recovery.publication-basis",
            PhysicalSignalAspectRole::DependencyAndOutput,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::RootPublication),
            expected_partition(store, session, "publication"),
        ),
        (
            "store.physical.recovery.cleanup-basis",
            PhysicalSignalAspectRole::DependencyAndOutput,
            PhysicalWorkSignalFamilySet::only(PhysicalWorkSignalFamily::WalReclamation),
            expected_partition(store, session, "cleanup"),
        ),
    ];
    observations.len() == expected.len()
        && expected.iter().all(|(key, role, families, partition)| {
            observations.iter().any(|observation| {
                observation.identity().aspect_key().as_str() == *key
                    && observation.role() == *role
                    && observation.families() == *families
                    && observation.partition().is_some_and(|actual| {
                        actual.partition.0 == *partition && actual.detail.is_none()
                    })
            })
        })
}

fn expected_partition(
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    session: [u8; 16],
    stage: &str,
) -> String {
    let store = store
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let session = session
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("store.physical.recovery/{store}/{session}/{stage}")
}

impl PhysicalRecoveryRegisteredSessionAuthority {
    /// Uses this registered Store session to admit the matching native recovery
    /// coordination owner after persisted Store admission.
    pub fn admit_coordination(
        self,
        media: &AdmittedRecoveryFilesystemMedia,
        capacity: PhysicalRecoveryCoordinationCapacity,
        yieldpoint: Option<crate::physical_runtime::PhysicalRecoveryProcessYieldpoint>,
    ) -> Result<PhysicalRecoveryCoordination, PhysicalRecoveryCoordinationAdmissionError> {
        PhysicalRecoveryCoordination::admit(media, self, capacity, yieldpoint)
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

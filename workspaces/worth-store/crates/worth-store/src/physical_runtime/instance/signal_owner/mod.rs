mod availability;
mod graph;
mod lifecycle_join;
mod outcome;
mod reconciliation;
mod route;
mod wake;
mod worker;
#[cfg(feature = "certification-test-authority")]
mod yieldpoint;

use std::sync::Arc;

use crate::physical_runtime::{
    work::{
        PhysicalSignalAspectBindingDigest, PhysicalSignalAspectBindingSet, PhysicalWorkAspectDelta,
    },
    LifecycleGeneration, PhysicalSignalProfileIdentity, PhysicalWorkProfileDeclaration,
};

pub use outcome::{
    PhysicalSignalClockObservation, PhysicalSignalClockObservationFailure,
    PhysicalSignalConstructionFailure, PhysicalSignalDeltaApplicationFailure,
    PhysicalSignalObservation, PhysicalSignalRuntimeIdentity, PhysicalSignalShutdownOutcome,
};
use route::PhysicalSignalRouteOwner;
use worker::PhysicalSignalGraphWorker;

pub(in crate::physical_runtime) use availability::PhysicalSignalAdmissionStatus;
#[cfg(feature = "certification-test-authority")]
pub use yieldpoint::CertificationPhysicalSignalPauseGate;

pub(in crate::physical_runtime) struct PhysicalWorkSignalOwner {
    runtime_identity: PhysicalSignalRuntimeIdentity,
    profile: PhysicalSignalProfileIdentity,
    bindings: Arc<PhysicalSignalAspectBindingSet>,
    graph_worker: PhysicalSignalGraphWorker,
    admission_status: PhysicalSignalAdmissionStatus,
    reconciliation: reconciliation::PhysicalSignalReconciliation,
    _lifecycle_generation: LifecycleGeneration,
}

impl PhysicalWorkSignalOwner {
    pub(in crate::physical_runtime) fn build_foundation(
        lifecycle_generation: LifecycleGeneration,
        profile: PhysicalWorkProfileDeclaration,
    ) -> Result<Self, PhysicalSignalConstructionFailure> {
        let runtime_identity = new_runtime_identity()?;
        let bindings = Arc::new(PhysicalSignalAspectBindingSet::install(profile));
        let profile = bindings.profile();
        let admission_status = PhysicalSignalAdmissionStatus::available();
        let graph_worker =
            PhysicalSignalGraphWorker::spawn(Arc::clone(&bindings), admission_status.clone())?;
        let reconciliation =
            reconciliation::PhysicalSignalReconciliation::bounded(bindings.capacity().commands());
        Ok(Self {
            runtime_identity,
            profile,
            bindings,
            graph_worker,
            admission_status,
            reconciliation,
            _lifecycle_generation: lifecycle_generation,
        })
    }

    pub(in crate::physical_runtime) const fn profile(&self) -> PhysicalSignalProfileIdentity {
        self.profile
    }

    pub(in crate::physical_runtime) fn bindings(&self) -> Arc<PhysicalSignalAspectBindingSet> {
        Arc::clone(&self.bindings)
    }

    pub(in crate::physical_runtime) fn binding_observations(
        &self,
    ) -> Box<[crate::physical_runtime::PhysicalSignalAspectBindingObservation]> {
        self.bindings.observations()
    }

    pub(in crate::physical_runtime) fn admission_status(&self) -> PhysicalSignalAdmissionStatus {
        self.admission_status.clone()
    }

    pub(in crate::physical_runtime) fn abandonment_publisher(
        &self,
    ) -> crate::physical_runtime::work::PhysicalWorkAbandonmentPublisher {
        self.graph_worker.abandonment_publisher()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pause_after_dequeue_for_certification(
        &self,
    ) -> CertificationPhysicalSignalPauseGate {
        self.graph_worker.pause_after_dequeue_for_certification()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn pause_abandonment_after_dequeue_for_certification(
        &self,
    ) -> CertificationPhysicalSignalPauseGate {
        self.graph_worker
            .pause_abandonment_after_dequeue_for_certification()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn fail_next_abandonment_for_certification(&self) {
        self.graph_worker.fail_next_abandonment_for_certification();
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn route_depth_for_certification(
        &self,
        route: PhysicalSignalAspectBindingDigest,
    ) -> Option<usize> {
        self.graph_worker.route_depth_for_certification(route)
    }

    pub(in crate::physical_runtime) const fn runtime_identity(
        &self,
    ) -> PhysicalSignalRuntimeIdentity {
        self.runtime_identity
    }

    pub(in crate::physical_runtime) fn clock_observation(
        &self,
    ) -> Result<PhysicalSignalClockObservation, PhysicalSignalClockObservationFailure> {
        self.require_available()
            .map_err(|_| PhysicalSignalClockObservationFailure::OwnerUnavailable)?;
        let basis = self
            .graph_worker
            .clock_basis()
            .ok_or(PhysicalSignalClockObservationFailure::OwnerUnavailable)?;
        Ok(PhysicalSignalClockObservation::new(
            basis.current_tick().get(),
            basis.last_advance_ordinal().get(),
        ))
    }

    pub(in crate::physical_runtime) fn apply_delta(
        &self,
        delta: PhysicalWorkAspectDelta,
    ) -> Result<(), PhysicalSignalDeltaApplicationFailure> {
        self.require_available()
            .map_err(|_| PhysicalSignalDeltaApplicationFailure::OwnerUnavailable)?;
        let route = self
            .route(delta.binding())
            .ok_or(PhysicalSignalDeltaApplicationFailure::BindingNotInstalled)?;
        route.apply_delta(delta)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn apply_delta_for_certification(
        &self,
        delta: PhysicalWorkAspectDelta,
    ) -> Result<(), PhysicalSignalDeltaApplicationFailure> {
        let binding = self
            .bindings
            .bindings()
            .iter()
            .find(|binding| binding.digest() == delta.binding())
            .ok_or(PhysicalSignalDeltaApplicationFailure::BindingNotInstalled)?;
        let delta = delta
            .rebind_for_certification(binding)
            .map_err(PhysicalSignalDeltaApplicationFailure::SemanticBasisRejected)?;
        self.apply_delta(delta)
    }

    pub(in crate::physical_runtime) fn apply_projection_failure_delta(
        &self,
        settled: &crate::physical_runtime::SettledPhysicalWork,
    ) -> Result<(), PhysicalSignalDeltaApplicationFailure> {
        let Some(delta) =
            self.projection_failure_delta(settled.intent(), settled.signal_binding())?
        else {
            return Ok(());
        };
        self.apply_delta(delta)
    }

    pub(in crate::physical_runtime) fn admit_projection_failure(
        &self,
        work: &crate::physical_runtime::ResourceAdmittedPhysicalWork,
    ) -> Result<Option<PhysicalWorkAspectDelta>, PhysicalSignalDeltaApplicationFailure> {
        self.projection_failure_delta(work.intent(), work.consumer_handle().route())
    }

    fn projection_failure_delta(
        &self,
        intent: &crate::physical_runtime::PhysicalWorkIntent,
        signal_binding: PhysicalSignalAspectBindingDigest,
    ) -> Result<Option<PhysicalWorkAspectDelta>, PhysicalSignalDeltaApplicationFailure> {
        let Some(fact) = intent.semantic_basis().projection_fact() else {
            return Ok(None);
        };
        let binding = self
            .bindings
            .binding_for_identity(intent.semantic_basis().aspect_identity())
            .filter(|binding| binding.digest() == signal_binding)
            .ok_or(PhysicalSignalDeltaApplicationFailure::BindingNotInstalled)?;
        let delta =
            PhysicalWorkAspectDelta::from_boundary_fact(binding, fact, intent.scope().clone())
                .map_err(PhysicalSignalDeltaApplicationFailure::SemanticBasisRejected)?;
        Ok(Some(delta))
    }

    pub(in crate::physical_runtime) fn revoke_derived_admission(&self) {
        self.admission_status.revoke();
    }

    pub(in crate::physical_runtime) fn observation(
        &self,
    ) -> Result<PhysicalSignalObservation, PhysicalSignalClockObservationFailure> {
        let route = self
            .bindings
            .bindings()
            .first()
            .and_then(|binding| self.route(binding.digest()))
            .ok_or(PhysicalSignalClockObservationFailure::OwnerUnavailable)?;
        let observation = route
            .observation()
            .map_err(|_| PhysicalSignalClockObservationFailure::OwnerUnavailable)?;
        let clock = observation.clock();
        Ok(PhysicalSignalObservation::new(
            self.profile,
            outcome::PhysicalSignalTopologyObservation {
                graph_owner_count: 1,
                aspect_binding_count: u16::try_from(self.bindings.len())
                    .expect("Signal aspect capacity fits u16"),
                locality_owner_count: u16::try_from(self.graph_worker.len())
                    .expect("Signal locality owner capacity fits u16"),
                active_locality_count: observation.active_locality_count(),
                active_graph_node_count: observation.active_graph_node_count(),
                active_in_flight_count: observation.resource().active_in_flight_node_count(),
                request_admission_count: observation.request_admission_count(),
                async_family_count: crate::physical_runtime::work::PHYSICAL_ASYNC_CAPABILITIES.len()
                    as u8,
                aspect_invalidation_count: observation.aspect_invalidation_count(),
            },
            PhysicalSignalClockObservation::new(
                clock.current_tick().get(),
                clock.last_advance_ordinal().get(),
            ),
        ))
    }

    pub(in crate::physical_runtime) fn request(
        &self,
        admitted: crate::physical_runtime::AdmittedPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkReadiness,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.require_available()?;
        let route = admitted.authority().binding();
        self.route(route)
            .ok_or(crate::physical_runtime::PhysicalWorkPreEffectDenial::CapabilityAbsent)?
            .request(admitted)
    }

    pub(in crate::physical_runtime) fn revalidate(
        &self,
        ready: crate::physical_runtime::ReadyPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkReadiness,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.require_available()?;
        let route = ready.authority().binding();
        self.route(route)
            .ok_or(crate::physical_runtime::PhysicalWorkPreEffectDenial::CapabilityAbsent)?
            .revalidate_ready(ready)
    }

    pub(in crate::physical_runtime) fn revalidate_blocked(
        &self,
        blocked: crate::physical_runtime::BlockedPhysicalWork,
    ) -> Result<
        crate::physical_runtime::PhysicalWorkReadiness,
        crate::physical_runtime::PhysicalWorkPreEffectDenial,
    > {
        self.require_available()?;
        let route = blocked.authority().binding();
        let (admitted, active) = blocked
            .into_revalidation_parts()
            .ok_or(crate::physical_runtime::PhysicalWorkPreEffectDenial::DependencyBlocked)?;
        self.route(route)
            .ok_or(crate::physical_runtime::PhysicalWorkPreEffectDenial::CapabilityAbsent)?
            .revalidate_blocked(admitted, active)
    }

    pub(in crate::physical_runtime) fn dispose(mut self) -> PhysicalSignalShutdownOutcome {
        let (pending, overflow) = self.reconciliation.counts();
        let owner_was_available = self.admission_status.is_available();
        self.admission_status.revoke();
        self.graph_worker.stop();
        if pending != 0 || overflow != 0 {
            PhysicalSignalShutdownOutcome::DerivedReconciliationPending { pending, overflow }
        } else if owner_was_available {
            PhysicalSignalShutdownOutcome::Disposed
        } else {
            PhysicalSignalShutdownOutcome::OwnerRevoked
        }
    }

    #[cfg(feature = "recovery-runtime-owner")]
    pub(in crate::physical_runtime) fn reconciliation_counts(&self) -> (usize, u64) {
        self.reconciliation.counts()
    }

    fn route(&self, route: PhysicalSignalAspectBindingDigest) -> Option<&PhysicalSignalRouteOwner> {
        self.graph_worker.route(route)
    }

    fn require_available(
        &self,
    ) -> Result<(), crate::physical_runtime::PhysicalWorkPreEffectDenial> {
        self.admission_status
            .is_available()
            .then_some(())
            .ok_or(crate::physical_runtime::PhysicalWorkPreEffectDenial::SignalOwnerUnavailable)
    }
}

impl Drop for PhysicalWorkSignalOwner {
    fn drop(&mut self) {
        self.admission_status.revoke();
    }
}

fn new_runtime_identity() -> Result<PhysicalSignalRuntimeIdentity, PhysicalSignalConstructionFailure>
{
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes)
        .map_err(|_| PhysicalSignalConstructionFailure::IdentityEntropyUnavailable)?;
    Ok(PhysicalSignalRuntimeIdentity::new(bytes))
}

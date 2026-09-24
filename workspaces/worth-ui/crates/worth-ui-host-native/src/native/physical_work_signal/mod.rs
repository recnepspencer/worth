mod completion_reconciliation;
mod construction;
mod counters;
mod declarations;
mod identity;
mod lifecycle_observation;
mod locality;
mod observation;
mod readiness_handoff;
mod routing;
mod shutdown;
mod temporal_progression;
mod transition_observation;
mod wake_delivery;
mod worker;
mod worker_graph;

#[cfg(test)]
mod tests;

use worth_ui_host_contract::UiGlyphRasterTransactionPending;

use self::construction::UiNativePhysicalSignalConstruction;
use self::counters::UiNativePhysicalSignalCounters;
use self::routing::UiNativePhysicalSignalRoute;
use self::shutdown::UiNativePhysicalSignalLifecycle;
use self::wake_delivery::UiNativePhysicalSignalWakeDelivery;

pub(crate) use completion_reconciliation::UiNativePhysicalSignalSettlement;
pub(crate) use identity::{
    UiNativePhysicalAtlasRequestIdentity, UiNativePhysicalAtlasRequestInput,
    UiNativePhysicalAtlasUploadIdentity, UiNativePhysicalPresentationBasis,
    UiNativePhysicalPresentationIdentity,
};
pub use lifecycle_observation::UiNativePhysicalSignalLifecycleObservation;
#[cfg(test)]
pub(crate) use observation::UiNativePhysicalRecoveryPosture;
pub(crate) use observation::UiNativePhysicalSignalObservation;
pub(crate) use readiness_handoff::UiNativePhysicalSignalReadyAttempt;
pub(crate) use routing::UiNativePhysicalSignalRequestToken;
pub(crate) use routing::UiNativePhysicalSignalWork;
pub(crate) use routing::{
    UiNativePhysicalSignalExternalBasis, UiNativePhysicalSignalExternalObservation,
    UiNativePhysicalSignalExternalStatus as UiNativePhysicalSignalStatus,
};
pub use transition_observation::{
    UiNativePhysicalSignalExternalStatusClass, UiNativePhysicalSignalObservationOriginClass,
    UiNativePhysicalSignalSettlementClass, UiNativePhysicalSignalTransitionObservation,
    UiNativePhysicalSignalWorkClass,
};

const PHYSICAL_SIGNAL_OBSERVATION_CAPACITY: usize = 64;

pub(crate) struct UiNativePhysicalSignalOwner {
    runtime_identity: identity::UiNativePhysicalSignalRuntimeIdentity,
    #[cfg(test)]
    declarations: declarations::UiNativePhysicalSignalDeclarations,
    route: UiNativePhysicalSignalRoute,
    worker: Option<worker::UiNativePhysicalSignalWorker>,
    terminal_telemetry: worth_signal::facade::adapters::RuntimeTelemetry,
    terminal_performed_transitions: u64,
    terminal_performed_nodes: u64,
    counters: UiNativePhysicalSignalCounters,
    wake: UiNativePhysicalSignalWakeDelivery,
    next_presentation_sequence: u64,
    next_atlas_sequence: u64,
    lifecycle: UiNativePhysicalSignalLifecycle,
    transition_observations: Vec<UiNativePhysicalSignalTransitionObservation>,
    transition_observation_overflowed: bool,
}

impl UiNativePhysicalSignalOwner {
    pub(crate) fn new() -> Self {
        let built = UiNativePhysicalSignalConstruction::build();
        let terminal_telemetry = built.worker.telemetry();
        Self {
            runtime_identity: built.runtime_identity,
            #[cfg(test)]
            declarations: built.declarations,
            route: built.route,
            worker: Some(built.worker),
            terminal_telemetry,
            terminal_performed_transitions: 0,
            terminal_performed_nodes: 0,
            counters: UiNativePhysicalSignalCounters::default(),
            wake: UiNativePhysicalSignalWakeDelivery::new(),
            next_presentation_sequence: 1,
            next_atlas_sequence: 1,
            lifecycle: UiNativePhysicalSignalLifecycle::Running,
            transition_observations: Vec::new(),
            transition_observation_overflowed: false,
        }
    }

    pub(crate) fn admit_atlas_planning(
        &mut self,
        presentation_basis: UiNativePhysicalPresentationBasis,
        demands: &[worth_ui_host_contract::UiGlyphRasterDemandBatchView<'_>],
        pins: worth_ui_host_contract::UiGlyphRasterPinTransitionView<'_>,
    ) -> Result<UiNativePhysicalAtlasRequestIdentity, ()> {
        let identity =
            UiNativePhysicalAtlasRequestIdentity::from_inputs(UiNativePhysicalAtlasRequestInput {
                runtime: self.runtime_identity,
                sequence: self.next_atlas_sequence,
                presentation_basis,
                demands,
                pins,
            });
        let next = self.next_atlas_sequence.checked_add(1).ok_or(())?;
        self.admit(
            declarations::UiNativePhysicalSignalOperation::AtlasUpload,
            UiNativePhysicalSignalWork::AtlasPlanning(identity),
        )?;
        self.next_atlas_sequence = next;
        Ok(identity)
    }

    pub(crate) fn take_ready_atlas_planning(
        &mut self,
        identity: UiNativePhysicalAtlasRequestIdentity,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        self.take_ready_work(UiNativePhysicalSignalWork::AtlasPlanning(identity))
    }

    pub(crate) fn bind_atlas_upload(
        &mut self,
        token: UiNativePhysicalSignalRequestToken,
        pending: UiGlyphRasterTransactionPending,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        let UiNativePhysicalSignalWork::AtlasPlanning(request) = token.work() else {
            return Err(());
        };
        let successor = UiNativePhysicalSignalWork::AtlasUpload(
            UiNativePhysicalAtlasUploadIdentity::new(request, pending),
        );
        let performed = self
            .worker_mut()?
            .replace_work(token.handle(), token.work(), successor)
            .ok_or(())?;
        if !self.route.replace_work(token, successor) {
            return Err(());
        }
        self.wake.remove(token.work());
        self.publish_performed(performed)?;
        self.take_ready_atlas_upload(pending)
    }

    pub(crate) fn admit_presentation(
        &mut self,
        basis: UiNativePhysicalPresentationBasis,
    ) -> Result<UiNativePhysicalPresentationIdentity, ()> {
        if self.route.contains_presentation_basis(basis) {
            return Err(());
        }
        let identity = UiNativePhysicalPresentationIdentity::new(
            self.runtime_identity,
            self.next_presentation_sequence,
            basis,
        );
        let next = self.next_presentation_sequence.checked_add(1).ok_or(())?;
        self.admit(
            declarations::UiNativePhysicalSignalOperation::PresentationReadback,
            UiNativePhysicalSignalWork::Presentation(identity),
        )?;
        self.next_presentation_sequence = next;
        Ok(identity)
    }

    fn begin_work(
        &mut self,
        work: UiNativePhysicalSignalWork,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        let token = self
            .route
            .token_for(self.runtime_identity, work)
            .map_err(|_| ())?;
        if !self.worker()?.contains(work) {
            return Err(());
        }
        Ok(token)
    }

    #[cfg(test)]
    pub(crate) fn begin(
        &mut self,
        pending: UiGlyphRasterTransactionPending,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        let identity = self.route.atlas_upload(pending).ok_or(())?;
        self.begin_work(UiNativePhysicalSignalWork::AtlasUpload(identity))
    }

    pub(crate) fn take_ready_atlas_upload(
        &mut self,
        pending: UiGlyphRasterTransactionPending,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        let identity = self.route.atlas_upload(pending).ok_or(())?;
        self.take_ready_work(UiNativePhysicalSignalWork::AtlasUpload(identity))
    }

    pub(crate) fn take_ready_presentation(
        &mut self,
        identity: UiNativePhysicalPresentationIdentity,
        retained: UiNativePhysicalSignalRequestToken,
    ) -> Result<UiNativePhysicalSignalReadyAttempt, ()> {
        let work = UiNativePhysicalSignalWork::Presentation(identity);
        if retained.work() != work {
            return Err(());
        }
        let current = match self.begin_work(work) {
            Ok(current) => current,
            Err(()) => {
                return Err(());
            }
        };
        if self.route.owner_handle(work) != Some(retained.handle()) {
            return Err(());
        }
        let handoff = if retained == current {
            UiNativePhysicalSignalReadyAttempt::Current(current)
        } else {
            UiNativePhysicalSignalReadyAttempt::Successor {
                predecessor: retained,
                successor: current,
            }
        };
        if !self.wake.take(work) {
            return Err(());
        }
        if !self
            .route
            .acknowledge_owner_handle(work, retained.handle(), current.handle())
        {
            return Err(());
        }
        Ok(handoff)
    }

    pub(crate) fn take_initial_presentation(
        &mut self,
        identity: UiNativePhysicalPresentationIdentity,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        self.take_ready_work(UiNativePhysicalSignalWork::Presentation(identity))
    }

    pub(crate) fn observation(&self) -> UiNativePhysicalSignalObservation {
        UiNativePhysicalSignalObservation::from_parts(
            observation::UiNativePhysicalSignalObservationInput {
                runtime: self.runtime_identity,
                active_requests: self.route.len(),
                wake: &self.wake,
                counters: self.counters,
                runtime_owned: self.worker.is_some(),
                accepting_admissions: self.lifecycle == UiNativePhysicalSignalLifecycle::Running,
                active_recoveries: self
                    .worker
                    .as_ref()
                    .map(|worker| {
                        worker.active_operation_count(
                            declarations::UiNativePhysicalSignalOperation::Recovery,
                        )
                    })
                    .unwrap_or(0),
                telemetry: self
                    .worker
                    .as_ref()
                    .map(worker::UiNativePhysicalSignalWorker::telemetry)
                    .unwrap_or(self.terminal_telemetry),
                performed_transitions: self
                    .worker
                    .as_ref()
                    .map(worker::UiNativePhysicalSignalWorker::performed_transitions)
                    .unwrap_or(self.terminal_performed_transitions),
                performed_nodes: self
                    .worker
                    .as_ref()
                    .map(worker::UiNativePhysicalSignalWorker::performed_nodes)
                    .unwrap_or(self.terminal_performed_nodes),
                last_performed: self
                    .worker
                    .as_ref()
                    .and_then(worker::UiNativePhysicalSignalWorker::last_performed),
                retained_transition_observations: self.transition_observations.len(),
            },
        )
    }

    pub(crate) fn next_ready_work(&self) -> Option<UiNativePhysicalSignalWork> {
        self.wake.next()
    }

    pub(crate) fn transition_observations(&self) -> &[UiNativePhysicalSignalTransitionObservation] {
        &self.transition_observations
    }

    pub(crate) const fn lifecycle_observation(&self) -> UiNativePhysicalSignalLifecycleObservation {
        UiNativePhysicalSignalLifecycleObservation::from_counters(self.counters)
    }

    pub(crate) const fn transition_observation_trace_complete(&self) -> bool {
        !self.transition_observation_overflowed
    }

    fn record_transition_observation(
        &mut self,
        observation: UiNativePhysicalSignalTransitionObservation,
    ) {
        if self.transition_observations.len() == PHYSICAL_SIGNAL_OBSERVATION_CAPACITY {
            self.transition_observations.remove(0);
            self.transition_observation_overflowed = true;
        }
        self.transition_observations.push(observation);
    }

    #[cfg(test)]
    pub(crate) fn declarations(&self) -> declarations::UiNativePhysicalSignalDeclarations {
        self.declarations
    }

    fn admit(
        &mut self,
        operation: declarations::UiNativePhysicalSignalOperation,
        work: UiNativePhysicalSignalWork,
    ) -> Result<(), ()> {
        if self.lifecycle != UiNativePhysicalSignalLifecycle::Running {
            return Err(());
        }
        let (handle, performed) = self.worker_mut()?.admit(operation, work)?;
        if self.route.record(work, handle).is_err() {
            let _ = self
                .worker_mut()
                .and_then(|worker| worker.cancel_handle(handle));
            return Err(());
        }
        self.counters.admissions = self.counters.admissions.saturating_add(1);
        self.publish_performed(performed)?;
        Ok(())
    }

    fn publish_performed(
        &mut self,
        performed: worker_graph::UiNativePhysicalSignalPerformed,
    ) -> Result<(), ()> {
        if performed.evaluated_nodes() == 0 || !self.worker()?.contains(performed.work()) {
            return Err(());
        }
        self.wake.request(performed.work());
        Ok(())
    }

    fn take_ready_work(
        &mut self,
        work: UiNativePhysicalSignalWork,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        let token = self.begin_work(work)?;
        if !self.wake.take(work) {
            return Err(());
        }
        Ok(token)
    }

    #[cfg(test)]
    pub(crate) fn take_ready_token(
        &mut self,
        expected: UiNativePhysicalSignalRequestToken,
    ) -> Result<UiNativePhysicalSignalRequestToken, ()> {
        let current = self.begin_work(expected.work())?;
        if current != expected || !self.wake.take(expected.work()) {
            return Err(());
        }
        Ok(expected)
    }

    pub(crate) fn token_uses_recovery(&self, token: UiNativePhysicalSignalRequestToken) -> bool {
        self.worker.as_ref().is_some_and(|worker| {
            worker.request_uses_operation(
                token.handle(),
                token.work(),
                declarations::UiNativePhysicalSignalOperation::Recovery,
            )
        })
    }

    fn worker(&self) -> Result<&worker::UiNativePhysicalSignalWorker, ()> {
        self.worker.as_ref().ok_or(())
    }

    fn worker_mut(&mut self) -> Result<&mut worker::UiNativePhysicalSignalWorker, ()> {
        self.worker.as_mut().ok_or(())
    }
}

//! The same physical publication endpoint is checked out for one caller step.
use super::owner::HomeCheckpointAttempt;
use std::sync::Arc;
use worth_store::physical_runtime::*;

pub(super) enum PublicationEndpoint {
    Ready(AnchoredRecordPublicationOperation),
    InCaller,
    Stopped(AnchoredRecordStoppedPublication),
    Failed(AnchoredRecordPublicationFailureCustody),
    Settled,
}

#[derive(Default)]
pub(super) struct HomePhysicalPublication {
    driver: DriverPhase,
    endpoint: Option<PublicationEndpoint>,
    recovery: Option<AnchoredRecordPublicationRecovery>,
    stopping: Option<AnchoredRecordStoppedPublication>,
    pub(super) registration: Option<Arc<AnchoredRecordStepRegistration>>,
}

#[derive(Default)]
enum DriverPhase {
    #[default]
    Available,
    Loaned,
}

/// Exclusive ownership of caller progression, without holding a mutex through
/// callbacks. Read-only Native medium queries never acquire this loan; the
/// original caller keeps it through WAL admission and its final handoff.
pub(super) struct PublicationDriverLoan {
    attempt: Arc<HomeCheckpointAttempt>,
}

impl Drop for PublicationDriverLoan {
    fn drop(&mut self) {
        let mut physical = self
            .attempt
            .physical
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        physical.stop_after_caller();
        physical.driver = DriverPhase::Available;
    }
}

pub(super) enum PublicationProgress {
    Pending,
    Selected(AnchoredRecordPublished),
    Failed(AnchoredRecordPublicationFailureCustody),
    Stopped(Vec<AnchoredRecordMemberLocator>),
    InspectionRequired,
}

impl HomePhysicalPublication {
    pub(super) fn install(
        &mut self,
        operation: AnchoredRecordPublicationOperation,
        recovery: AnchoredRecordPublicationRecovery,
    ) {
        self.recovery = Some(recovery);
        self.endpoint = Some(PublicationEndpoint::Ready(operation));
    }
    pub(super) fn unsettled(&self) -> bool {
        matches!(
            self.endpoint,
            Some(
                PublicationEndpoint::Ready(_)
                    | PublicationEndpoint::InCaller
                    | PublicationEndpoint::Stopped(_)
                    | PublicationEndpoint::Failed(_)
            )
        )
    }
    pub(super) fn completed_prefix(&self) -> Vec<AnchoredRecordMemberLocator> {
        self.recovery.as_ref().map_or_else(Vec::new, |recovery| {
            recovery.observe().completed_prefix().to_vec()
        })
    }
    pub(super) fn stop_after_caller(&mut self) {
        self.stop();
        if matches!(self.endpoint, Some(PublicationEndpoint::InCaller)) {
            self.endpoint = self.stopping.take().map(PublicationEndpoint::Stopped);
        }
    }
    pub(super) fn stop(&mut self) {
        self.endpoint = match self.endpoint.take() {
            Some(PublicationEndpoint::Ready(operation)) => {
                Some(PublicationEndpoint::Stopped(operation.stop()))
            }
            Some(PublicationEndpoint::InCaller) => {
                if self.stopping.is_none() {
                    self.stopping = self.recovery.take().map(|recovery| recovery.stop());
                }
                Some(PublicationEndpoint::InCaller)
            }
            other => other,
        };
    }
    pub(super) fn terminal(&self) -> Option<Arc<AnchoredRecordStepTerminal>> {
        self.registration
            .as_ref()
            .and_then(|registration| registration.terminal())
    }
    pub(super) fn clear(&mut self) {
        self.endpoint.take();
        self.recovery.take();
        self.registration.take();
        self.stopping.take();
    }
}

impl HomeCheckpointAttempt {
    pub(super) fn try_driver(self: &Arc<Self>) -> Option<PublicationDriverLoan> {
        let mut physical = self
            .physical
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match physical.driver {
            DriverPhase::Available => {
                physical.driver = DriverPhase::Loaned;
                Some(PublicationDriverLoan {
                    attempt: self.clone(),
                })
            }
            DriverPhase::Loaned => None,
        }
    }
    pub(super) fn stop_advancement(&self) {
        self.physical
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .stop();
    }

    pub(super) fn advance_physical(
        self: &Arc<Self>,
        records: &AnchoredRecords<'_>,
        observe: Option<&super::step_custody::StepObserver<'_>>,
    ) -> PublicationProgress {
        let endpoint = {
            let mut physical = self
                .physical
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if !matches!(
                &physical.endpoint,
                Some(
                    PublicationEndpoint::Ready(_)
                        | PublicationEndpoint::Stopped(_)
                        | PublicationEndpoint::Failed(_)
                )
            ) {
                return PublicationProgress::Pending;
            }
            physical.endpoint.replace(PublicationEndpoint::InCaller)
        };
        match endpoint {
            Some(PublicationEndpoint::Ready(operation)) => {
                let port = super::step_custody::HomeCheckpointStepPort::new(self, observe);
                match operation.step(records, &port) {
                    AnchoredRecordPublicationStep::Pending(operation)
                    | AnchoredRecordPublicationStep::Ready(operation) => {
                        let mut physical = self
                            .physical
                            .lock()
                            .unwrap_or_else(|error| error.into_inner());
                        if physical.stopping.take().is_some() {
                            physical.endpoint =
                                Some(PublicationEndpoint::Stopped(operation.stop()));
                        } else {
                            physical.endpoint = Some(PublicationEndpoint::Ready(operation));
                        }
                        PublicationProgress::Pending
                    }
                    AnchoredRecordPublicationStep::Completed(receipt) => {
                        self.physical
                            .lock()
                            .unwrap_or_else(|error| error.into_inner())
                            .endpoint = Some(PublicationEndpoint::Settled);
                        PublicationProgress::Selected(receipt)
                    }
                    AnchoredRecordPublicationStep::Failed(failure) => {
                        self.preserve_failure(failure)
                    }
                }
            }
            Some(PublicationEndpoint::Stopped(stopped)) => self.settle_stopped(stopped, records),
            Some(PublicationEndpoint::Failed(failure)) => self.preserve_failure(failure),
            None | Some(PublicationEndpoint::InCaller | PublicationEndpoint::Settled) => {
                unreachable!("only owned actionable endpoints are checked out")
            }
        }
    }

    fn settle_stopped(
        &self,
        stopped: AnchoredRecordStoppedPublication,
        records: &AnchoredRecords<'_>,
    ) -> PublicationProgress {
        match stopped.settle(records) {
            AnchoredRecordStoppedPoll::Pending(stopped) => {
                self.physical
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .endpoint = Some(PublicationEndpoint::Stopped(stopped));
                PublicationProgress::Pending
            }
            AnchoredRecordStoppedPoll::Settled(observation) => {
                self.physical
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .endpoint = Some(PublicationEndpoint::Settled);
                if let Some(receipt) = observation.terminal() {
                    PublicationProgress::Selected(receipt.clone())
                } else {
                    PublicationProgress::Stopped(observation.completed_prefix().to_vec())
                }
            }
            AnchoredRecordStoppedPoll::Failed(failure) => self.preserve_failure(failure),
        }
    }

    fn preserve_failure(
        &self,
        failure: AnchoredRecordPublicationFailureCustody,
    ) -> PublicationProgress {
        let observation = failure.observe();
        let known_terminal = observation.step_terminal().is_some_and(|terminal| {
            terminal.settlement_failure().is_none()
                && matches!(
                    terminal.outcome(),
                    PhysicalMutationOutcome::ProvenNoEffect(_)
                )
        });
        if known_terminal {
            self.physical
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .endpoint = Some(PublicationEndpoint::Settled);
            PublicationProgress::Failed(failure)
        } else {
            self.physical
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .endpoint = Some(PublicationEndpoint::Failed(failure));
            PublicationProgress::InspectionRequired
        }
    }
}

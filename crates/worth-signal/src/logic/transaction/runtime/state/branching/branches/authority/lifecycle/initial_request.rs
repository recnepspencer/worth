//! Initial request reports retain their exact displaced or unused timeout.
use super::*;
use crate::branch::owner_services::conditional_execution::{
    SignalOwnedAsyncManagedAdmissionState, SignalOwnedAsyncManagedWakeCleanup,
    SignalOwnedAsyncQueueDenial,
};

impl<D, I, T> BranchState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub(crate) fn admit_owned_async_request_handoff(
        &mut self,
        node: ResourceNodeId,
        allow_coalescing: bool,
    ) -> Result<SignalOwnedAsyncManagedAdmissionState, SignalConditionalServiceExecutionDenial>
    {
        self.admit_owned_async_source(node)?;
        // Resolve the real registered descriptor before scheduling. Both owners
        // remain under this cell lock, so no fallible descriptor lookup follows
        // creation of the timeout wake.
        let descriptor = self
            .derived
            .resource
            .descriptor_for_node(node)
            .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?
            .descriptor_id();
        let prior_timeout = self.derived.resource.active_timeout_wake_for_node(node);
        let tick = self.derived.temporal.clock_basis().current_tick();
        let timeout = self.schedule_owned_async_timeout(node, tick)?;
        let new_timeout = timeout.as_ref().map(|(wake, _)| wake.id());
        let timeout_admission = timeout.map(|(_, admission)| admission);
        let capture = self.captures_telemetry();
        let branch = self.branch_id();
        let report = self
            .derived
            .resource
            .admit_resource_request_with_descriptor(
                ResourceRequestIntent::new(node),
                descriptor,
                branch,
                tick,
                allow_coalescing,
                timeout_admission,
                capture.then_some(&mut self.derived.telemetry.resource),
            );
        let (wake, reason) = if report.intent_equivalence_coalescing().is_some() {
            (new_timeout, TemporalWakeRetirementReason::Disposed)
        } else if report.superseded_request().is_some() {
            (prior_timeout, TemporalWakeRetirementReason::Superseded)
        } else {
            (None, TemporalWakeRetirementReason::Disposed)
        };
        let (cleanup, refusal) = match self.retire_optional_wake(wake, reason) {
            Ok(()) => (None, None),
            Err(cause) => (
                wake.map(|wake| SignalOwnedAsyncManagedWakeCleanup::Retirement { wake, reason }),
                Some(SignalOwnedAsyncQueueDenial::Service(cause)),
            ),
        };
        match self.require_in_flight(report.admitted_request().handle()) {
            Ok(in_flight) if refusal.is_none() => {
                Ok(SignalOwnedAsyncManagedAdmissionState::Ready {
                    request: SignalOwnedAsyncRequestAdmission::new(report, in_flight),
                    queue: None,
                })
            }
            result => {
                let (in_flight, cause) = match result {
                    Ok(in_flight) => (Some(in_flight), refusal.expect("retained cleanup refusal")),
                    Err(cause) => (
                        None,
                        refusal.unwrap_or(SignalOwnedAsyncQueueDenial::Service(cause)),
                    ),
                };
                Ok(SignalOwnedAsyncManagedAdmissionState::CleanupRequired {
                    report,
                    in_flight,
                    queue: None,
                    cleanup,
                    cause,
                })
            }
        }
    }
}

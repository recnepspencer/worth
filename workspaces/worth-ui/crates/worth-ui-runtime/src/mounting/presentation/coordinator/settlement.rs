use super::super::outcome::UiMountedPresentationOutcome;
use super::super::state::{
    UiMountedPresentationCompletionDenial, UiMountedPresentationInFlight,
    UiMountedPresentationInFlightState,
};
use super::super::terminal::UiIndeterminatePresentationEvidence;
use super::super::terminal::{aggregate_affected, rejected_outcome};
use super::pending_completion::{observe_pending_surface, PendingCompletionContext};
use super::UiMountedPresentationCoordinator;
use crate::facade::UiHostEffectPort;

impl UiMountedPresentationCoordinator {
    pub fn complete(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
        host: UiHostEffectPort<'_>,
        now: u64,
    ) -> Result<UiMountedPresentationOutcome, UiMountedPresentationCompletionDenial> {
        let attempt_identity = in_flight.attempt();
        let Some(state) = self.in_flight.remove(&attempt_identity) else {
            return Err(UiMountedPresentationCompletionDenial::UnknownAttempt);
        };
        if state.deadline.expired_at(now) {
            return Ok(self.finish_expired_completion(state, host));
        }
        Ok(self.poll_pending_completion(state, host))
    }

    fn finish_expired_completion(
        &mut self,
        state: UiMountedPresentationInFlightState,
        host: UiHostEffectPort<'_>,
    ) -> UiMountedPresentationOutcome {
        let UiMountedPresentationInFlightState {
            frame,
            retention,
            attempt,
            pending,
            rejected,
            completed,
            superseded_costs: _,
            ..
        } = state;
        let affected = aggregate_affected(&completed, &pending, &rejected);
        let stopped = super::cancellation::cancel_all(pending, host);
        let cancellation = super::cancellation_settlement::settle(
            stopped,
            self.presentation_async.as_mut(),
            worth_ui_host_contract::UiHostSurfacePresentationDenial::DeadlineExpired,
        );
        let requires_indeterminate = cancellation.requires_indeterminate();
        let (timeout_rejections, semantic_receipts, recovery_required, physical_recovery_bindings) =
            cancellation.into_parts();
        if requires_indeterminate || !completed.is_empty() {
            return self.indeterminate(
                frame,
                retention,
                attempt,
                UiIndeterminatePresentationEvidence::new(affected, completed)
                    .with_semantic_receipts(semantic_receipts)
                    .with_recovery_required(recovery_required)
                    .with_physical_recovery_bindings(physical_recovery_bindings),
            );
        }
        self.active.borrow_mut().remove(&attempt);
        let mut rejections = rejected;
        rejections.extend(timeout_rejections);
        rejected_outcome(attempt, frame, retention, rejections)
    }

    fn poll_pending_completion(
        &mut self,
        state: UiMountedPresentationInFlightState,
        host: UiHostEffectPort<'_>,
    ) -> UiMountedPresentationOutcome {
        let UiMountedPresentationInFlightState {
            frame,
            retention,
            attempt,
            deadline,
            pending,
            rejected,
            completed,
            superseded_costs,
            semantic_requests,
            superseded,
            reconstructed_bindings,
            candidates,
        } = state;
        let mut progress = super::UiMountedPresentationProgress {
            pending: Vec::new(),
            rejected,
            completed,
            superseded_costs,
            semantic_requests,
            superseded,
        };
        let mut remaining = Vec::new();
        let mut pending_iter = pending.into_iter();
        while let Some(pending_surface) = pending_iter.next() {
            let observation = {
                let mut context = PendingCompletionContext::new(
                    &frame,
                    &mut progress,
                    &mut self.text,
                    self.presentation_async.as_mut(),
                );
                observe_pending_surface(host, pending_surface, &mut context)
            };
            if let Some(observation) = observation {
                remaining.extend(pending_iter);
                progress.pending.extend(remaining);
                let evidence = super::surface_uncertainty::terminalize(
                    &mut progress,
                    host,
                    self.presentation_async.as_mut(),
                    observation,
                );
                return self.indeterminate(frame, retention, attempt, evidence);
            }
        }
        progress.pending.extend(remaining);
        self.finish_or_wait(super::UiMountedPresentationSettlement {
            frame,
            retention,
            attempt,
            deadline,
            pending: progress.pending,
            rejected: progress.rejected,
            completed: progress.completed,
            superseded_costs: progress.superseded_costs,
            semantic_requests: progress.semantic_requests,
            superseded: progress.superseded,
            reconstructed_bindings,
            candidates,
            host,
        })
    }

    pub fn cancel(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
        host: UiHostEffectPort<'_>,
    ) -> Result<UiMountedPresentationOutcome, UiMountedPresentationCompletionDenial> {
        let Some(state) = self.in_flight.remove(&in_flight.attempt()) else {
            return Err(UiMountedPresentationCompletionDenial::UnknownAttempt);
        };
        Ok(self.stop_state(
            state,
            host,
            worth_ui_host_contract::UiHostSurfaceStopReason::Cancelled,
        ))
    }

    pub(crate) fn supersede(
        &mut self,
        in_flight: UiMountedPresentationInFlight,
        host: UiHostEffectPort<'_>,
    ) -> Result<UiMountedPresentationOutcome, UiMountedPresentationCompletionDenial> {
        let Some(state) = self.in_flight.remove(&in_flight.attempt()) else {
            return Err(UiMountedPresentationCompletionDenial::UnknownAttempt);
        };
        Ok(self.stop_state(
            state,
            host,
            worth_ui_host_contract::UiHostSurfaceStopReason::Superseded,
        ))
    }

    pub(crate) fn shutdown(
        &mut self,
        host: UiHostEffectPort<'_>,
    ) -> (
        super::super::UiMountedPresentationShutdownReport,
        Vec<UiMountedPresentationOutcome>,
        Option<crate::native_platform::text_presentation::UiPresentationAsyncTerminalCleanup>,
    ) {
        self.shutting_down = true;
        let attempts = self.in_flight.keys().copied().collect::<Vec<_>>();
        let mut records = Vec::with_capacity(attempts.len());
        let mut outcomes = Vec::with_capacity(attempts.len());
        for attempt in attempts {
            let state = self
                .in_flight
                .remove(&attempt)
                .expect("shutdown attempt was retained by the coordinator");
            let outcome = self.stop_state(
                state,
                host,
                worth_ui_host_contract::UiHostSurfaceStopReason::Cancelled,
            );
            let (disposition, affected) = match &outcome {
                UiMountedPresentationOutcome::RejectedBeforeEffects(_) => (
                    super::super::UiMountedPresentationShutdownDisposition::CancelledBeforeEffects,
                    Vec::new(),
                ),
                UiMountedPresentationOutcome::PresentationIndeterminate(frame) => (
                    super::super::UiMountedPresentationShutdownDisposition::PresentationIndeterminate,
                    frame.report().affected_bindings().to_vec(),
                ),
                UiMountedPresentationOutcome::Presented(_)
                | UiMountedPresentationOutcome::Superseded(_)
                | UiMountedPresentationOutcome::InFlight(_) => {
                    unreachable!("shutdown cancellation always reaches a terminal state")
                }
            };
            records.push(super::super::UiMountedPresentationShutdownAttempt::new(
                attempt,
                disposition,
                affected,
            ));
            outcomes.push(outcome);
        }
        let (
            closed_query_resources,
            query_close_complete,
            query_transitions,
            query_transition_trace_complete,
            cleanup,
        ) = match self.presentation_async.take() {
            Some(runtime) => match runtime.into_terminal_close() {
                Ok(receipt) => {
                    self.unresolved_semantic_receipts.clear();
                    self.unresolved_semantic_recoveries.clear();
                    (
                        receipt.closed_query_resources(),
                        true,
                        receipt.transitions().to_vec().into_boxed_slice(),
                        receipt.transition_trace_complete(),
                        None,
                    )
                }
                Err(cleanup) => (
                    0,
                    false,
                    Vec::<worth_ui_query_binding::WorthUiPresentationTransitionObservation>::new()
                        .into_boxed_slice(),
                    false,
                    Some(cleanup),
                ),
            },
            None => (
                0,
                true,
                Vec::<worth_ui_query_binding::WorthUiPresentationTransitionObservation>::new()
                    .into_boxed_slice(),
                true,
                None,
            ),
        };
        let (text_presentation_work, text_presentation_work_trace_complete) =
            self.text.take_work_observations();
        (
            super::super::UiMountedPresentationShutdownReport::new(
                records,
                super::super::UiMountedPresentationQueryShutdown {
                    closed_resources: closed_query_resources,
                    complete: query_close_complete,
                    transitions: query_transitions,
                    transition_trace_complete: query_transition_trace_complete,
                },
                super::super::UiMountedPresentationTextShutdown {
                    work: text_presentation_work,
                    trace_complete: text_presentation_work_trace_complete,
                },
            ),
            outcomes,
            cleanup,
        )
    }

    fn stop_state(
        &mut self,
        state: UiMountedPresentationInFlightState,
        host: UiHostEffectPort<'_>,
        reason: worth_ui_host_contract::UiHostSurfaceStopReason,
    ) -> UiMountedPresentationOutcome {
        let UiMountedPresentationInFlightState {
            frame,
            retention,
            attempt,
            pending,
            rejected,
            completed,
            superseded_costs: _,
            ..
        } = state;
        let affected = aggregate_affected(&completed, &pending, &rejected);
        let stopped = super::cancellation::stop_all(pending, host, reason);
        let cancellation = super::cancellation_settlement::settle(
            stopped,
            self.presentation_async.as_mut(),
            worth_ui_host_contract::UiHostSurfacePresentationDenial::CancelledBeforeEffects,
        );
        let requires_indeterminate = cancellation.requires_indeterminate();
        let (
            cancellation_rejections,
            semantic_receipts,
            recovery_required,
            physical_recovery_bindings,
        ) = cancellation.into_parts();
        if requires_indeterminate || !completed.is_empty() {
            return self.indeterminate(
                frame,
                retention,
                attempt,
                UiIndeterminatePresentationEvidence::new(affected, completed)
                    .with_semantic_receipts(semantic_receipts)
                    .with_recovery_required(recovery_required)
                    .with_physical_recovery_bindings(physical_recovery_bindings),
            );
        }
        self.active.borrow_mut().remove(&attempt);
        let mut rejections = rejected;
        rejections.extend(cancellation_rejections);
        rejected_outcome(attempt, frame, retention, rejections)
    }
}

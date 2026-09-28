mod settlement;

use super::{
    UiActiveIntentExecutionReservation, UiIntentExecutionSlotPhase, UiIntentExecutionState,
    UiPreparedFrameworkIntentAttempt, UiRunningFrameworkIntentAttempt,
};
use crate::runtime::intent_execution::provider::{
    UiManagedIntentExecution, UiManagedIntentExecutionPoll, UiManagedIntentExecutionStart,
};
use crate::runtime::intent_execution::{
    UiIntentExecutionAdvanceMetrics, UiIntentExecutionAdvanceOutcome,
    UiIntentExecutionAdvanceReport, UiIntentExecutionAdvanceStop,
    UiIntentExecutionCancellationContext, UiIntentExecutionCancellationReason,
    UiIntentExecutionPollContext, UiIntentExecutionTransition, UiIntentExecutionTransitionPosture,
    UiIntentProviderStop,
};
use settlement::UiIntentSettlementContext;

struct UiIntentExecutionAdvancePass {
    metrics: UiIntentExecutionAdvanceMetrics,
    transitions: Vec<UiIntentExecutionTransition>,
}

struct UiRunningIntentProgression {
    reservation: UiActiveIntentExecutionReservation,
    attempt: crate::runtime::intent_execution::UiIntentExecutionAttemptIdentity,
    idempotency: crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity,
    deadline: crate::runtime::intent_execution::UiIntentExecutionDeadline,
    cancellation: Option<UiIntentExecutionCancellationReason>,
}

impl UiIntentExecutionState {
    pub(crate) fn advance(&mut self, tick: u64) -> UiIntentExecutionAdvanceOutcome {
        if let Err((previous, observed)) = self.admit_monotonic_tick(tick) {
            return UiIntentExecutionAdvanceOutcome::Stopped(
                UiIntentExecutionAdvanceStop::MonotonicTimeRegressed { previous, observed },
            );
        }
        let mut pass = UiIntentExecutionAdvancePass {
            metrics: UiIntentExecutionAdvanceMetrics::default(),
            transitions: Vec::new(),
        };
        for index in 0..self.slots.len() {
            let Some(phase) = self.slots[index].phase.take() else {
                continue;
            };
            match phase {
                UiIntentExecutionSlotPhase::AttemptPrepared(prepared) => {
                    pass.metrics.active_slots_visited += 1;
                    self.advance_prepared(index, tick, prepared, &mut pass);
                }
                UiIntentExecutionSlotPhase::Running(running) => {
                    pass.metrics.active_slots_visited += 1;
                    self.advance_running(index, tick, running, &mut pass);
                }
                retained @ (UiIntentExecutionSlotPhase::Admitted(_)
                | UiIntentExecutionSlotPhase::Recovery(_)
                | UiIntentExecutionSlotPhase::ConsequencePending(_)
                | UiIntentExecutionSlotPhase::ConsequenceReady(_)
                | UiIntentExecutionSlotPhase::ConsequenceHandoff(_)) => {
                    self.slots[index].phase = Some(retained)
                }
            }
        }
        UiIntentExecutionAdvanceOutcome::Advanced(UiIntentExecutionAdvanceReport::new(
            pass.transitions,
            pass.metrics,
        ))
    }

    fn advance_prepared(
        &mut self,
        index: usize,
        tick: u64,
        prepared: UiPreparedFrameworkIntentAttempt,
        pass: &mut UiIntentExecutionAdvancePass,
    ) {
        if tick > prepared.deadline.tick() {
            let posture_basis = prepared.reservation.posture_basis();
            pass.metrics.settlements += 1;
            pass.transitions.push(
                UiIntentExecutionTransition::new(
                    prepared.attempt,
                    prepared.idempotency,
                    UiIntentExecutionTransitionPosture::TimedOutBeforeEffect {
                        detail: UiIntentProviderStop::stable(
                            "worth_ui.execution.deadline_elapsed_before_start",
                        ),
                    },
                    None,
                )
                .with_posture_basis(posture_basis),
            );
            self.release_reservation(prepared.reservation);
            return;
        }
        let context =
            crate::runtime::intent_execution::provider::UiManagedIntentExecutionStartContext::new(
                prepared.attempt,
                prepared.idempotency,
                prepared.deadline,
            );
        pass.metrics.provider_calls += 1;
        match prepared.execution.start(context) {
            UiManagedIntentExecutionStart::Running(execution) => {
                pass.transitions.push(UiIntentExecutionTransition::new(
                    prepared.attempt,
                    prepared.idempotency,
                    UiIntentExecutionTransitionPosture::Started,
                    None,
                ));
                self.slots[index].phase = Some(UiIntentExecutionSlotPhase::Running(
                    UiRunningFrameworkIntentAttempt {
                        reservation: prepared.reservation,
                        execution,
                        attempt: prepared.attempt,
                        idempotency: prepared.idempotency,
                        deadline: prepared.deadline,
                        cancellation: None,
                    },
                ));
            }
            UiManagedIntentExecutionStart::Settled(settlement) => self.settle(
                UiIntentSettlementContext::new(
                    index,
                    prepared.reservation,
                    prepared.attempt,
                    prepared.idempotency,
                ),
                settlement,
                pass,
            ),
        }
    }

    fn advance_running(
        &mut self,
        index: usize,
        tick: u64,
        running: UiRunningFrameworkIntentAttempt,
        pass: &mut UiIntentExecutionAdvancePass,
    ) {
        let UiRunningFrameworkIntentAttempt {
            reservation,
            execution,
            attempt,
            idempotency,
            deadline,
            mut cancellation,
        } = running;
        if cancellation.is_none() && tick > deadline.tick() {
            cancellation = Some(UiIntentExecutionCancellationReason::DeadlineExpired);
        }
        let poll = if let Some(reason) = cancellation {
            pass.metrics.cancellation_calls += 1;
            execution.cancel(UiIntentExecutionCancellationContext::new(tick, reason))
        } else {
            pass.metrics.provider_polls += 1;
            execution.poll(UiIntentExecutionPollContext::at_tick(tick))
        };
        self.accept_poll(
            index,
            UiRunningIntentProgression {
                reservation,
                attempt,
                idempotency,
                deadline,
                cancellation,
            },
            poll,
            pass,
        );
    }

    fn accept_poll(
        &mut self,
        index: usize,
        running: UiRunningIntentProgression,
        poll: UiManagedIntentExecutionPoll,
        pass: &mut UiIntentExecutionAdvancePass,
    ) {
        let attempt = running.attempt;
        let idempotency = running.idempotency;
        let posture = match poll {
            UiManagedIntentExecutionPoll::PendingBeforeEffect(execution) => {
                self.retain_running(index, running, execution);
                UiIntentExecutionTransitionPosture::PendingBeforeEffect
            }
            UiManagedIntentExecutionPoll::PendingEffectMayHaveBegun(execution) => {
                self.retain_running(index, running, execution);
                UiIntentExecutionTransitionPosture::PendingEffectMayHaveBegun
            }
            UiManagedIntentExecutionPoll::Settled(settlement) => {
                return self.settle(
                    UiIntentSettlementContext::new(
                        index,
                        running.reservation,
                        running.attempt,
                        running.idempotency,
                    ),
                    settlement,
                    pass,
                );
            }
        };
        pass.transitions.push(UiIntentExecutionTransition::new(
            attempt,
            idempotency,
            posture,
            None,
        ));
    }

    fn retain_running(
        &mut self,
        index: usize,
        running: UiRunningIntentProgression,
        execution: Box<dyn UiManagedIntentExecution>,
    ) {
        self.slots[index].phase = Some(UiIntentExecutionSlotPhase::Running(
            UiRunningFrameworkIntentAttempt {
                reservation: running.reservation,
                execution,
                attempt: running.attempt,
                idempotency: running.idempotency,
                deadline: running.deadline,
                cancellation: running.cancellation,
            },
        ));
    }
}

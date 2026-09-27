use super::super::{
    UiActiveIntentExecutionReservation, UiIntentExecutionSlotPhase, UiIntentExecutionState,
    UiRecoveringFrameworkIntentAttempt, UiSettledFrameworkIntentAttempt,
};
use super::UiIntentExecutionAdvancePass;
use crate::runtime::intent_execution::provider::{
    UiManagedIntentOutcomeMaterial, UiManagedIntentPartialEffect, UiManagedIntentRecovery,
    UiManagedIntentSettlement,
};
use crate::runtime::intent_execution::{
    UiIntentExecutionAttemptIdentity, UiIntentExecutionIdempotencyIdentity,
    UiIntentExecutionTransition, UiIntentExecutionTransitionPosture,
};

pub(super) struct UiIntentSettlementContext {
    index: usize,
    reservation: UiActiveIntentExecutionReservation,
    attempt: UiIntentExecutionAttemptIdentity,
    idempotency: UiIntentExecutionIdempotencyIdentity,
}

struct UiIntentSettlementDisposition {
    phase: Option<UiIntentExecutionSlotPhase>,
    transition: UiIntentExecutionTransition,
    release: Option<UiActiveIntentExecutionReservation>,
}

impl UiIntentSettlementContext {
    pub(super) const fn new(
        index: usize,
        reservation: UiActiveIntentExecutionReservation,
        attempt: UiIntentExecutionAttemptIdentity,
        idempotency: UiIntentExecutionIdempotencyIdentity,
    ) -> Self {
        Self {
            index,
            reservation,
            attempt,
            idempotency,
        }
    }
}

impl UiIntentExecutionState {
    pub(super) fn settle(
        &mut self,
        context: UiIntentSettlementContext,
        settlement: UiManagedIntentSettlement,
        pass: &mut UiIntentExecutionAdvancePass,
    ) {
        let index = context.index;
        pass.metrics.settlements += 1;
        let disposition = match settlement {
            UiManagedIntentSettlement::Completed(outcome) => completed(context, outcome),
            UiManagedIntentSettlement::Partial { effect, recovery } => {
                recoverable_partial(context, effect, recovery)
            }
            UiManagedIntentSettlement::Indeterminate { detail, recovery } => {
                recoverable_indeterminate(context, detail, recovery)
            }
            terminal @ (UiManagedIntentSettlement::RejectedBeforeEffect(_)
            | UiManagedIntentSettlement::FailedBeforeEffect(_)
            | UiManagedIntentSettlement::CancelledBeforeEffect(_)
            | UiManagedIntentSettlement::TimedOutBeforeEffect(_)) => {
                before_effect(context, terminal)
            }
        };
        if let Some(reservation) = disposition.release {
            self.release_reservation(reservation);
        }
        self.slots[index].phase = disposition.phase;
        pass.transitions.push(disposition.transition);
    }
}

fn completed(
    context: UiIntentSettlementContext,
    outcome: Box<dyn UiManagedIntentOutcomeMaterial>,
) -> UiIntentSettlementDisposition {
    let outcome_schema = outcome.schema();
    let basis = context.reservation.consequence_basis();
    let (consequence, consequence_lease) =
        crate::runtime::intent_execution::UiIntentConsequenceHandle::new(
            context.attempt,
            context.idempotency,
        );
    UiIntentSettlementDisposition {
        phase: Some(UiIntentExecutionSlotPhase::ConsequencePending(
            UiSettledFrameworkIntentAttempt {
                attempt: context.attempt,
                idempotency: context.idempotency,
                outcome,
                consequence_lease,
                basis,
            },
        )),
        transition: UiIntentExecutionTransition::completed(
            context.attempt,
            context.idempotency,
            outcome_schema,
            consequence,
        ),
        release: Some(context.reservation),
    }
}

fn recoverable_partial(
    context: UiIntentSettlementContext,
    effect: UiManagedIntentPartialEffect,
    recovery: Box<dyn UiManagedIntentRecovery>,
) -> UiIntentSettlementDisposition {
    let posture = UiIntentExecutionTransitionPosture::Partial {
        outcome: effect.outcome_schema(),
        detail: effect.detail(),
    };
    recovering(context, recovery, Some(effect), posture)
}

fn recoverable_indeterminate(
    context: UiIntentSettlementContext,
    detail: Option<crate::runtime::intent_execution::UiIntentProviderStop>,
    recovery: Box<dyn UiManagedIntentRecovery>,
) -> UiIntentSettlementDisposition {
    recovering(
        context,
        recovery,
        None,
        UiIntentExecutionTransitionPosture::Indeterminate { detail },
    )
}

fn recovering(
    context: UiIntentSettlementContext,
    recovery: Box<dyn UiManagedIntentRecovery>,
    partial: Option<UiManagedIntentPartialEffect>,
    posture: UiIntentExecutionTransitionPosture,
) -> UiIntentSettlementDisposition {
    let (handle, lease) = crate::runtime::intent_execution::UiIntentRecoveryHandle::new(
        context.attempt,
        context.idempotency,
    );
    UiIntentSettlementDisposition {
        phase: Some(UiIntentExecutionSlotPhase::Recovery(
            UiRecoveringFrameworkIntentAttempt {
                reservation: context.reservation,
                recovery,
                attempt: context.attempt,
                idempotency: context.idempotency,
                lease,
                partial,
            },
        )),
        transition: UiIntentExecutionTransition::new(
            context.attempt,
            context.idempotency,
            posture,
            Some(handle),
        ),
        release: None,
    }
}

fn before_effect(
    context: UiIntentSettlementContext,
    settlement: UiManagedIntentSettlement,
) -> UiIntentSettlementDisposition {
    let posture_basis = context.reservation.posture_basis();
    let posture = match settlement {
        UiManagedIntentSettlement::RejectedBeforeEffect(detail) => {
            UiIntentExecutionTransitionPosture::RejectedBeforeEffect { detail }
        }
        UiManagedIntentSettlement::FailedBeforeEffect(detail) => {
            UiIntentExecutionTransitionPosture::FailedBeforeEffect { detail }
        }
        UiManagedIntentSettlement::CancelledBeforeEffect(detail) => {
            UiIntentExecutionTransitionPosture::CancelledBeforeEffect { detail }
        }
        UiManagedIntentSettlement::TimedOutBeforeEffect(detail) => {
            UiIntentExecutionTransitionPosture::TimedOutBeforeEffect { detail }
        }
        UiManagedIntentSettlement::Completed(_)
        | UiManagedIntentSettlement::Partial { .. }
        | UiManagedIntentSettlement::Indeterminate { .. } => {
            unreachable!("completed and recovery settlements are handled before pre-effect")
        }
    };
    UiIntentSettlementDisposition {
        phase: None,
        transition: UiIntentExecutionTransition::new(
            context.attempt,
            context.idempotency,
            posture,
            None,
        )
        .with_posture_basis(posture_basis),
        release: Some(context.reservation),
    }
}

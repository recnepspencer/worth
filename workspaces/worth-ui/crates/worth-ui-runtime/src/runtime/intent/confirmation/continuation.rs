use super::matching::matching_slots;
use super::validation::{
    validate_challenge, UiIntentConfirmationReadContext, UiIntentConfirmationSourceBasis,
};

use super::super::UiIntentAttemptLineage;
use super::state::{
    UiIntentConfirmationSlotState, UiIntentConfirmationState, UiIntentConfirmationTerminal,
    UiIntentConfirmationTerminalKind,
};
use super::{
    UiIntentConfirmationCancellationReason, UiIntentConfirmationChallenge,
    UiIntentConfirmationLookupCost, UiIntentConfirmationStop, UiIntentConfirmationStopReason,
};

#[must_use]
pub enum UiIntentConfirmationContinuation {
    AdmissionReady(UiConfirmedIntentCandidate),
    Stopped(UiIntentConfirmationStop),
}

#[must_use]
pub struct UiConfirmedIntentCandidate {
    candidate: super::super::payload::UiPreparedIntentPayload,
    confirmation_decision: super::super::operability::UiIntentOperabilityDecision,
    lineage: UiIntentAttemptLineage,
}

pub(crate) fn continue_confirmation(
    state: &mut UiIntentConfirmationState,
    route: super::super::UiResolvedConfirmationIntentRoute,
    context: UiIntentConfirmationReadContext<'_>,
) -> UiIntentConfirmationContinuation {
    let (_, route_definition, route_declaration, source, _route_resolution, _) = route.into_parts();
    let declaration = route_declaration.identity().as_str();
    let (pending, terminal, inspected) = matching_slots(state, declaration, route_definition);
    if pending.len() > 1 {
        settle_ambiguous(state, &pending);
        state.record_stopped();
        return stopped(
            UiIntentConfirmationStopReason::AmbiguousPendingChallenges {
                declaration: declaration.into(),
                observed: pending.len(),
            },
            inspected,
        );
    }
    let Some(slot) = pending.first().copied() else {
        state.record_stopped();
        return terminal_stop(state, declaration, terminal, inspected);
    };
    let challenge = take_pending(state, slot);
    let stop = validate_challenge(
        &challenge,
        route_definition,
        &route_declaration,
        UiIntentConfirmationSourceBasis {
            generation: source.generation(),
            target: source.target(),
            time_basis: source.time_basis(),
        },
        &context,
    );
    if let Some(reason) = stop {
        let terminal_kind = if matches!(reason, UiIntentConfirmationStopReason::Expired { .. }) {
            state.record_expired();
            UiIntentConfirmationTerminalKind::Expired
        } else {
            UiIntentConfirmationTerminalKind::Stopped
        };
        set_terminal(state, slot, &challenge, terminal_kind);
        state.record_stopped();
        return stopped(reason, inspected);
    }
    let UiIntentConfirmationChallenge {
        candidate,
        decision,
        lineage,
        slot_identity,
        ..
    } = challenge;
    let terminal = UiIntentConfirmationTerminal {
        declaration: candidate.declaration_identity().into(),
        definition: candidate.definition_id(),
        lineage,
        slot_identity,
        kind: UiIntentConfirmationTerminalKind::Continued,
    };
    state.slots[slot].state = UiIntentConfirmationSlotState::Terminal(terminal);
    state.record_continued();
    UiIntentConfirmationContinuation::AdmissionReady(UiConfirmedIntentCandidate {
        candidate,
        confirmation_decision: decision,
        lineage,
    })
}

fn settle_ambiguous(state: &mut UiIntentConfirmationState, slots: &[usize]) {
    for slot in slots {
        let challenge = take_pending(state, *slot);
        set_terminal(
            state,
            *slot,
            &challenge,
            UiIntentConfirmationTerminalKind::Cancelled(
                UiIntentConfirmationCancellationReason::AmbiguousContinuation,
            ),
        );
        drop(challenge);
    }
    state.record_cancelled(slots.len());
}

fn take_pending(
    state: &mut UiIntentConfirmationState,
    slot: usize,
) -> UiIntentConfirmationChallenge {
    let previous = core::mem::replace(
        &mut state.slots[slot].state,
        UiIntentConfirmationSlotState::Vacant,
    );
    let UiIntentConfirmationSlotState::Pending(challenge) = previous else {
        unreachable!("selected confirmation slot is pending")
    };
    challenge
}

fn set_terminal(
    state: &mut UiIntentConfirmationState,
    slot: usize,
    challenge: &UiIntentConfirmationChallenge,
    kind: UiIntentConfirmationTerminalKind,
) {
    state.slots[slot].state = UiIntentConfirmationSlotState::Terminal(
        UiIntentConfirmationTerminal::from_challenge(challenge, kind),
    );
}

fn terminal_stop(
    state: &mut UiIntentConfirmationState,
    declaration: &str,
    terminal: Vec<usize>,
    inspected: usize,
) -> UiIntentConfirmationContinuation {
    if terminal.len() != 1 {
        return stopped(
            if terminal.is_empty() {
                UiIntentConfirmationStopReason::NoPendingChallenge {
                    declaration: declaration.into(),
                }
            } else {
                UiIntentConfirmationStopReason::AmbiguousPendingChallenges {
                    declaration: declaration.into(),
                    observed: terminal.len(),
                }
            },
            inspected,
        );
    }
    let slot = terminal[0];
    let previous = core::mem::replace(
        &mut state.slots[slot].state,
        UiIntentConfirmationSlotState::Vacant,
    );
    let UiIntentConfirmationSlotState::Terminal(marker) = previous else {
        unreachable!("selected confirmation terminal is present")
    };
    state.record_replay();
    let reason = marker.stop_reason();
    let _terminal_identity = (marker.lineage, marker.slot_identity);
    stopped(reason, inspected)
}

fn stopped(
    reason: UiIntentConfirmationStopReason,
    slots_inspected: usize,
) -> UiIntentConfirmationContinuation {
    UiIntentConfirmationContinuation::Stopped(UiIntentConfirmationStop::new(
        reason,
        UiIntentConfirmationLookupCost::new(slots_inspected),
    ))
}

impl UiConfirmedIntentCandidate {
    pub const fn definition_id(&self) -> crate::capability::UiIntentId {
        self.candidate.definition_id()
    }

    pub fn declaration_identity(&self) -> &str {
        self.candidate.declaration_identity()
    }

    pub const fn lineage(&self) -> UiIntentAttemptLineage {
        self.lineage
    }

    pub const fn confirmation_decision(
        &self,
    ) -> &super::super::operability::UiIntentOperabilityDecision {
        &self.confirmation_decision
    }

    pub fn retained_payload_count(&self) -> usize {
        self.candidate.retained_payload_count()
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::super::payload::UiPreparedIntentPayload,
        super::super::operability::UiIntentOperabilityDecision,
        UiIntentAttemptLineage,
    ) {
        (self.candidate, self.confirmation_decision, self.lineage)
    }
}

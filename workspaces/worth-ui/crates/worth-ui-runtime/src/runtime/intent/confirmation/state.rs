use super::challenge::{
    prepare_challenge, UiIntentConfirmationChallenge, UiIntentConfirmationIssueOutcome,
    UiIntentConfirmationSlotIdentity, UI_PENDING_INTENT_CONFIRMATION_LIMIT,
};
use super::{
    UiIntentConfirmationCancellationReason, UiIntentConfirmationLookupCost,
    UiIntentConfirmationStop, UiIntentConfirmationStopReason,
};

pub(crate) struct UiIntentConfirmationState {
    pub(super) slots: Box<[UiIntentConfirmationSlot]>,
    pub(super) counters: UiIntentConfirmationCounters,
}

pub(super) struct UiIntentConfirmationSlot {
    generation: u64,
    pub(super) state: UiIntentConfirmationSlotState,
}

pub(super) enum UiIntentConfirmationSlotState {
    Vacant,
    Pending(UiIntentConfirmationChallenge),
    Terminal(UiIntentConfirmationTerminal),
}

pub(super) struct UiIntentConfirmationTerminal {
    pub(super) declaration: Box<str>,
    pub(super) definition: crate::capability::UiIntentId,
    pub(super) lineage: super::super::UiIntentAttemptLineage,
    pub(super) slot_identity: UiIntentConfirmationSlotIdentity,
    pub(super) kind: UiIntentConfirmationTerminalKind,
}

#[derive(Clone, Copy)]
pub(super) enum UiIntentConfirmationTerminalKind {
    Continued,
    Cancelled(UiIntentConfirmationCancellationReason),
    Expired,
    Stopped,
}

#[derive(Default)]
pub(super) struct UiIntentConfirmationCounters {
    pub(super) issued: u64,
    pub(super) continued: u64,
    pub(super) stopped: u64,
    pub(super) cancelled: u64,
    pub(super) expired: u64,
    pub(super) replays: u64,
}

impl UiIntentConfirmationState {
    pub(crate) fn new() -> Self {
        Self {
            slots: (0..UI_PENDING_INTENT_CONFIRMATION_LIMIT)
                .map(|_| UiIntentConfirmationSlot {
                    generation: 0,
                    state: UiIntentConfirmationSlotState::Vacant,
                })
                .collect(),
            counters: Default::default(),
        }
    }

    pub(crate) fn issue(
        &mut self,
        candidate: super::super::operability::UiInoperableIntentCandidate,
        lineage: Option<super::super::UiIntentAttemptLineage>,
    ) -> UiIntentConfirmationIssueOutcome {
        let prepared = match prepare_challenge(candidate) {
            Ok(prepared) => prepared,
            Err(reason) => {
                self.record_stopped();
                return stopped(reason, 0);
            }
        };
        let (slot, inspected) = match self.issue_slot() {
            Some(slot) => slot,
            None => {
                self.record_stopped();
                return stopped(
                    UiIntentConfirmationStopReason::ChallengeCapacityExceeded {
                        maximum: UI_PENDING_INTENT_CONFIRMATION_LIMIT,
                    },
                    UI_PENDING_INTENT_CONFIRMATION_LIMIT,
                );
            }
        };
        let Some(lineage) = lineage else {
            self.record_stopped();
            return stopped(
                UiIntentConfirmationStopReason::ChallengeIdentityExhausted,
                inspected,
            );
        };
        let Some(generation) = self.slots[slot].generation.checked_add(1) else {
            self.record_stopped();
            return stopped(
                UiIntentConfirmationStopReason::ChallengeIdentityExhausted,
                inspected,
            );
        };
        let slot_identity = UiIntentConfirmationSlotIdentity::new(slot as u8, generation);
        let (challenge, pending) = prepared.seal(lineage, slot_identity);
        self.slots[slot].generation = generation;
        self.slots[slot].state = UiIntentConfirmationSlotState::Pending(challenge);
        self.counters.issued = next(self.counters.issued);
        UiIntentConfirmationIssueOutcome::Pending(pending)
    }

    fn issue_slot(&self) -> Option<(usize, usize)> {
        let mut inspected = 0;
        for (index, slot) in self.slots.iter().enumerate() {
            inspected += 1;
            if !matches!(slot.state, UiIntentConfirmationSlotState::Pending(_)) {
                return Some((index, inspected));
            }
        }
        None
    }

    pub(super) fn pending_count(&self) -> usize {
        self.slots
            .iter()
            .filter(|slot| matches!(slot.state, UiIntentConfirmationSlotState::Pending(_)))
            .count()
    }

    pub(super) fn record_stopped(&mut self) {
        self.counters.stopped = next(self.counters.stopped);
    }

    pub(super) fn record_continued(&mut self) {
        self.counters.continued = next(self.counters.continued);
    }

    pub(super) fn record_expired(&mut self) {
        self.counters.expired = next(self.counters.expired);
    }

    pub(super) fn record_replay(&mut self) {
        self.counters.replays = next(self.counters.replays);
    }

    pub(super) fn record_cancelled(&mut self, count: usize) {
        self.counters.cancelled = self
            .counters
            .cancelled
            .checked_add(count as u64)
            .expect("bounded confirmation cancellation accounting exhausted");
    }
}

impl UiIntentConfirmationTerminal {
    pub(super) fn stop_reason(&self) -> UiIntentConfirmationStopReason {
        match self.kind {
            UiIntentConfirmationTerminalKind::Continued => {
                UiIntentConfirmationStopReason::AlreadyContinued
            }
            UiIntentConfirmationTerminalKind::Cancelled(reason) => {
                UiIntentConfirmationStopReason::LifecycleCancelled(reason)
            }
            UiIntentConfirmationTerminalKind::Expired => {
                UiIntentConfirmationStopReason::AlreadyStopped
            }
            UiIntentConfirmationTerminalKind::Stopped => {
                UiIntentConfirmationStopReason::AlreadyStopped
            }
        }
    }

    pub(super) fn from_challenge(
        challenge: &UiIntentConfirmationChallenge,
        kind: UiIntentConfirmationTerminalKind,
    ) -> Self {
        Self {
            declaration: challenge.candidate.declaration_identity().into(),
            definition: challenge.candidate.definition_id(),
            lineage: challenge.lineage,
            slot_identity: challenge.slot_identity,
            kind,
        }
    }
}

fn stopped(
    reason: UiIntentConfirmationStopReason,
    slots_inspected: usize,
) -> UiIntentConfirmationIssueOutcome {
    UiIntentConfirmationIssueOutcome::Stopped(UiIntentConfirmationStop::new(
        reason,
        UiIntentConfirmationLookupCost::new(slots_inspected),
    ))
}

fn next(value: u64) -> u64 {
    value
        .checked_add(1)
        .expect("bounded confirmation lifecycle accounting exhausted")
}

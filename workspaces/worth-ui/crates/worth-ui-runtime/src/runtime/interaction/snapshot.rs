#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiInteractionLifecycleCounters {
    button_reports: u64,
    gestures_started: u64,
    gestures_completed: u64,
    pointer_stop_outcomes: u64,
    active_gestures_settled: u64,
    recipients_bound: u64,
    draft_sessions_started: u64,
    draft_sessions_settled: u64,
    draft_mutations: u64,
    local_input_stop_outcomes: u64,
    semantic_interactions: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiInteractionStateSnapshot {
    active_gestures: usize,
    pointer_presence_records: usize,
    active_recipients: usize,
    active_draft_sessions: usize,
    retained_draft_utf8_bytes: usize,
    counters: UiInteractionLifecycleCounters,
}

impl UiInteractionStateSnapshot {
    pub(super) fn from_parts(
        pointer: super::gesture::UiPointerGestureStateSnapshot,
        pointer_presence_records: usize,
        draft: super::draft::UiDraftStateSnapshot,
        semantic_interactions: u64,
    ) -> Self {
        Self {
            active_gestures: pointer.active_gestures,
            pointer_presence_records,
            active_recipients: draft.active_recipients,
            active_draft_sessions: draft.active_sessions,
            retained_draft_utf8_bytes: draft.retained_utf8_bytes,
            counters: UiInteractionLifecycleCounters {
                button_reports: pointer.counters.button_reports,
                gestures_started: pointer.counters.gestures_started,
                gestures_completed: pointer.counters.gestures_completed,
                pointer_stop_outcomes: pointer.counters.stop_outcomes,
                active_gestures_settled: pointer.counters.active_gestures_settled,
                recipients_bound: draft.counters.recipients_bound,
                draft_sessions_started: draft.counters.sessions_started,
                draft_sessions_settled: draft.counters.sessions_settled,
                draft_mutations: draft.counters.mutations,
                local_input_stop_outcomes: draft.counters.stop_outcomes,
                semantic_interactions,
            },
        }
    }

    pub const fn active_gestures(self) -> usize {
        self.active_gestures
    }

    pub const fn active_recipients(self) -> usize {
        self.active_recipients
    }

    pub const fn pointer_presence_records(self) -> usize {
        self.pointer_presence_records
    }

    pub const fn active_draft_sessions(self) -> usize {
        self.active_draft_sessions
    }

    pub const fn retained_draft_utf8_bytes(self) -> usize {
        self.retained_draft_utf8_bytes
    }

    pub const fn counters(self) -> UiInteractionLifecycleCounters {
        self.counters
    }
}

impl UiInteractionLifecycleCounters {
    pub const fn button_reports(self) -> u64 {
        self.button_reports
    }

    pub const fn gestures_started(self) -> u64 {
        self.gestures_started
    }

    pub const fn gestures_completed(self) -> u64 {
        self.gestures_completed
    }

    pub const fn stop_outcomes(self) -> u64 {
        self.pointer_stop_outcomes
    }

    pub const fn active_gestures_settled(self) -> u64 {
        self.active_gestures_settled
    }

    pub const fn recipients_bound(self) -> u64 {
        self.recipients_bound
    }

    pub const fn draft_sessions_started(self) -> u64 {
        self.draft_sessions_started
    }

    pub const fn draft_sessions_settled(self) -> u64 {
        self.draft_sessions_settled
    }

    pub const fn draft_mutations(self) -> u64 {
        self.draft_mutations
    }

    pub const fn local_input_stop_outcomes(self) -> u64 {
        self.local_input_stop_outcomes
    }

    pub const fn semantic_interactions(self) -> u64 {
        self.semantic_interactions
    }
}

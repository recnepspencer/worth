use std::sync::{Arc, Mutex};

use super::WorthQueryJournalReplayDenialKind;

#[derive(Clone, Debug, Default)]
pub(in crate::runtime) struct WorthQueryJournalReplayCounters {
    state: Arc<Mutex<WorthQueryJournalReplayCounterState>>,
}

impl WorthQueryJournalReplayCounters {
    pub fn record_admitted_replay(
        &self,
        scanned_entry_count: usize,
        resolved_entry_count: usize,
        journal_gap_count: usize,
        outcome_digest: &str,
    ) {
        let mut state = self.state.lock().expect("journal replay counters lock");
        state.replay_admission_count += 1;
        state.replay_scanned_entry_count += scanned_entry_count;
        state.replay_resolved_entry_count += resolved_entry_count;
        state.replay_gap_count += journal_gap_count;
        state.last_replay_outcome_digest = Some(outcome_digest.to_string());
    }

    pub fn record_denial(&self, kind: WorthQueryJournalReplayDenialKind) {
        let mut state = self.state.lock().expect("journal replay counters lock");
        *state.denial_count_mut(kind) += 1;
    }

    pub fn snapshot(&self, retained_entry_count: usize) -> WorthQueryJournalReplayCounterSnapshot {
        let state = self.state.lock().expect("journal replay counters lock");
        WorthQueryJournalReplayCounterSnapshot {
            retained_entry_count,
            replay_admission_count: state.replay_admission_count,
            invalid_segment_bounds_denial_count: state.invalid_segment_bounds_denial_count,
            unknown_segment_identity_denial_count: state.unknown_segment_identity_denial_count,
            stale_basis_replay_denial_count: state.stale_basis_replay_denial_count,
            cross_scheme_replay_denial_count: state.cross_scheme_replay_denial_count,
            journal_gap_denial_count: state.journal_gap_denial_count,
            identity_derivation_denial_count: state.identity_derivation_denial_count,
            replay_scanned_entry_count: state.replay_scanned_entry_count,
            replay_resolved_entry_count: state.replay_resolved_entry_count,
            replay_gap_count: state.replay_gap_count,
            replay_residue_count: state.replay_residue_count,
            last_replay_outcome_digest: state.last_replay_outcome_digest.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct WorthQueryJournalReplayCounterState {
    replay_admission_count: usize,
    invalid_segment_bounds_denial_count: usize,
    unknown_segment_identity_denial_count: usize,
    stale_basis_replay_denial_count: usize,
    cross_scheme_replay_denial_count: usize,
    journal_gap_denial_count: usize,
    identity_derivation_denial_count: usize,
    replay_scanned_entry_count: usize,
    replay_resolved_entry_count: usize,
    replay_gap_count: usize,
    replay_residue_count: usize,
    last_replay_outcome_digest: Option<String>,
}

impl WorthQueryJournalReplayCounterState {
    fn denial_count_mut(&mut self, kind: WorthQueryJournalReplayDenialKind) -> &mut usize {
        match kind {
            WorthQueryJournalReplayDenialKind::InvalidSegmentBounds => {
                &mut self.invalid_segment_bounds_denial_count
            }
            WorthQueryJournalReplayDenialKind::UnknownSegmentIdentity => {
                &mut self.unknown_segment_identity_denial_count
            }
            WorthQueryJournalReplayDenialKind::StaleBasisReplay => {
                &mut self.stale_basis_replay_denial_count
            }
            WorthQueryJournalReplayDenialKind::CrossSchemeReplay => {
                &mut self.cross_scheme_replay_denial_count
            }
            WorthQueryJournalReplayDenialKind::JournalGap => &mut self.journal_gap_denial_count,
            WorthQueryJournalReplayDenialKind::EvidenceIdentityDerivationDenied(_) => {
                &mut self.identity_derivation_denial_count
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalReplayCounterSnapshot {
    retained_entry_count: usize,
    replay_admission_count: usize,
    invalid_segment_bounds_denial_count: usize,
    unknown_segment_identity_denial_count: usize,
    stale_basis_replay_denial_count: usize,
    cross_scheme_replay_denial_count: usize,
    journal_gap_denial_count: usize,
    identity_derivation_denial_count: usize,
    replay_scanned_entry_count: usize,
    replay_resolved_entry_count: usize,
    replay_gap_count: usize,
    replay_residue_count: usize,
    last_replay_outcome_digest: Option<String>,
}

impl WorthQueryJournalReplayCounterSnapshot {
    pub fn retained_entry_count(&self) -> usize {
        self.retained_entry_count
    }

    pub fn replay_admission_count(&self) -> usize {
        self.replay_admission_count
    }

    pub fn denial_count(&self, kind: WorthQueryJournalReplayDenialKind) -> usize {
        match kind {
            WorthQueryJournalReplayDenialKind::InvalidSegmentBounds => {
                self.invalid_segment_bounds_denial_count
            }
            WorthQueryJournalReplayDenialKind::UnknownSegmentIdentity => {
                self.unknown_segment_identity_denial_count
            }
            WorthQueryJournalReplayDenialKind::StaleBasisReplay => {
                self.stale_basis_replay_denial_count
            }
            WorthQueryJournalReplayDenialKind::CrossSchemeReplay => {
                self.cross_scheme_replay_denial_count
            }
            WorthQueryJournalReplayDenialKind::JournalGap => self.journal_gap_denial_count,
            WorthQueryJournalReplayDenialKind::EvidenceIdentityDerivationDenied(_) => {
                self.identity_derivation_denial_count
            }
        }
    }

    pub fn replay_scanned_entry_count(&self) -> usize {
        self.replay_scanned_entry_count
    }

    pub fn replay_resolved_entry_count(&self) -> usize {
        self.replay_resolved_entry_count
    }

    pub fn replay_gap_count(&self) -> usize {
        self.replay_gap_count
    }

    pub fn replay_residue_count(&self) -> usize {
        self.replay_residue_count
    }

    pub fn last_replay_outcome_digest(&self) -> Option<&str> {
        self.last_replay_outcome_digest.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalReplayDiagnostics {
    counter_snapshot: WorthQueryJournalReplayCounterSnapshot,
}

impl WorthQueryJournalReplayDiagnostics {
    pub(in crate::runtime) fn new(
        counter_snapshot: WorthQueryJournalReplayCounterSnapshot,
    ) -> Self {
        Self { counter_snapshot }
    }

    pub fn counter_snapshot(&self) -> &WorthQueryJournalReplayCounterSnapshot {
        &self.counter_snapshot
    }
}

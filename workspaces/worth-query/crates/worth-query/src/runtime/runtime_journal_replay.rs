use super::*;

impl WorthQueryRuntime {
    pub fn replay_journal_segment(
        &self,
        request: WorthQueryJournalReplayRequest,
    ) -> Result<WorthQueryJournalReplayOutcome, WorthQueryRuntimeError> {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Replay)?;
        self.journal_replay
            .replay(
                request,
                &self.current_snapshot_identity()?,
                journal_replay::published_artifact_replay_digest(
                    &self.published_artifact_diagnostics(),
                ),
            )
            .map_err(WorthQueryRuntimeError::JournalReplayDenied)
    }

    pub fn journal_replay_diagnostics(&self) -> WorthQueryJournalReplayDiagnostics {
        self.journal_replay.diagnostics()
    }

    pub fn retain_journal_replay_positions_for_certification(
        &mut self,
        retained_positions: &std::collections::BTreeSet<u64>,
    ) {
        self.journal_replay
            .retain_replay_positions_for_certification(retained_positions);
    }
}

impl WorthQueryWorkspace {
    pub fn replay_journal_segment(
        &self,
        request: WorthQueryJournalReplayRequest,
    ) -> Result<WorthQueryJournalReplayOutcome, WorthQueryRuntimeError> {
        self.runtime.replay_journal_segment(request)
    }

    pub fn journal_replay_diagnostics(&self) -> WorthQueryJournalReplayDiagnostics {
        self.runtime.journal_replay_diagnostics()
    }

    pub fn retain_journal_replay_positions_for_certification(
        &mut self,
        retained_positions: &std::collections::BTreeSet<u64>,
    ) {
        self.runtime
            .retain_journal_replay_positions_for_certification(retained_positions);
    }
}

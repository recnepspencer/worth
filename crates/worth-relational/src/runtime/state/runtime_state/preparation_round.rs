//! Provisional telemetry and diagnostics for a candidate that may be abandoned.
use super::RelationalPreparationRuntime;

impl RelationalPreparationRuntime {
    pub(crate) fn preparation_round(&self) -> Self {
        let mut round = self.clone();
        round.services.instrumentation = self.services.instrumentation.preparation_round();
        round.diagnostics = Default::default();
        round
    }

    /// Settle only a miss-free round. Identity and request-work owners were
    /// shared throughout; this never rewinds either issuance or actual work.
    pub(crate) fn settle_preparation_round(&self, round: &Self) {
        self.performance_access()
            .accumulate_round_counts(round.performance_access().complexity_counters_snapshot());
        for artifact in round.diagnostics.snapshot() {
            self.diagnostics.push(artifact);
        }
    }
}

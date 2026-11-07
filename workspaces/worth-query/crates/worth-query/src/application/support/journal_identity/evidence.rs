use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalIdentityInventoryEvidence {
    forbidden_pattern_count: usize,
    missing_required_pattern_count: usize,
    missing_operation_count: usize,
    inventory_digest: String,
}
impl WorthQueryJournalIdentityInventoryEvidence {
    pub fn new(
        forbidden_pattern_count: usize,
        missing_required_pattern_count: usize,
        missing_operation_count: usize,
        inventory_digest: impl Into<String>,
    ) -> Self {
        Self {
            forbidden_pattern_count,
            missing_required_pattern_count,
            missing_operation_count,
            inventory_digest: inventory_digest.into(),
        }
    }

    pub fn total_failure_count(&self) -> usize {
        self.forbidden_pattern_count
            + self.missing_required_pattern_count
            + self.missing_operation_count
    }

    pub fn inventory_digest(&self) -> &str {
        &self.inventory_digest
    }

    #[cfg(test)]
    pub fn with_forbidden_failure_for_sabotage(&self) -> Self {
        Self {
            forbidden_pattern_count: self.forbidden_pattern_count + 1,
            ..self.clone()
        }
    }
}

use crate::runtime::{
    WorthQueryJournalPositionSchedule, WorthQueryJournalReplayCounterSnapshot,
    WorthQueryJournalReplayOutcome, WorthQueryWriteReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalIdentityScheduleEvidence {
    monotonic_position_count: usize,
    stable_replay_count: usize,
    collision_free_count: usize,
    expected_position_count: usize,
    schedule_digest: String,
}
impl WorthQueryJournalIdentityScheduleEvidence {
    pub fn derive(
        canonical: &WorthQueryJournalPositionSchedule,
        replay: &WorthQueryJournalPositionSchedule,
    ) -> Self {
        Self::new(
            canonical.monotonic_position_count(),
            canonical.stable_replay_count(replay),
            canonical.collision_free_count(),
            canonical.expected_position_count(),
            canonical.schedule_digest().as_str(),
        )
    }

    fn new(
        monotonic_position_count: usize,
        stable_replay_count: usize,
        collision_free_count: usize,
        expected_position_count: usize,
        schedule_digest: impl Into<String>,
    ) -> Self {
        Self {
            monotonic_position_count,
            stable_replay_count,
            collision_free_count,
            expected_position_count,
            schedule_digest: schedule_digest.into(),
        }
    }

    pub fn certified(&self) -> bool {
        self.expected_position_count > 0
            && self.monotonic_position_count == self.expected_position_count
            && self.stable_replay_count == self.expected_position_count
            && self.collision_free_count == self.expected_position_count
            && !self.schedule_digest.is_empty()
    }

    pub fn schedule_digest(&self) -> &str {
        &self.schedule_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalReplaySurfaceEvidence {
    journal_segment_identity_digest: String,
    replayed_receipt_count: usize,
    expected_receipt_count: usize,
    expected_journal_position_count: usize,
    resolved_journal_position_count: usize,
    journal_gap_count: usize,
    scanned_entry_count: usize,
    committed_truth_digest: String,
    replay_truth_digest: String,
    published_artifact_digest: String,
    replay_outcome_digest: String,
    counter_snapshot: WorthQueryJournalReplayCounterSnapshot,
}
impl WorthQueryJournalReplaySurfaceEvidence {
    pub fn derive_from_committed_receipts(
        outcome: &WorthQueryJournalReplayOutcome,
        expected_receipt_count: usize,
        committed_receipts: &[WorthQueryWriteReceipt],
        counter_snapshot: WorthQueryJournalReplayCounterSnapshot,
    ) -> Result<Self, IdentityDerivationDenial> {
        let committed_truth_identity =
            crate::runtime::journal_replay_truth_reconstruction_identity(
                &committed_receipts
                    .iter()
                    .map(|receipt| receipt.committed_truth_identity().clone())
                    .collect::<Vec<_>>(),
                expected_receipt_count,
                0,
            )?;
        Ok(Self {
            journal_segment_identity_digest: outcome
                .segment_identity()
                .identity_digest()
                .to_string(),
            replayed_receipt_count: outcome.write_receipts().len(),
            expected_receipt_count,
            expected_journal_position_count: outcome.expected_journal_position_count(),
            resolved_journal_position_count: outcome.resolved_journal_position_count(),
            journal_gap_count: outcome.journal_gap_count(),
            scanned_entry_count: outcome.scanned_entry_count(),
            committed_truth_digest: committed_truth_identity.as_str().to_string(),
            replay_truth_digest: outcome.truth_reconstruction_identity().as_str().to_string(),
            published_artifact_digest: outcome.published_artifact_digest().as_str().to_string(),
            replay_outcome_digest: outcome.outcome_identity().as_str().to_string(),
            counter_snapshot,
        })
    }

    pub fn certified(&self) -> bool {
        self.expected_receipt_count > 0
            && self.replayed_receipt_count == self.expected_receipt_count
            && self.expected_journal_position_count == self.expected_receipt_count
            && self.resolved_journal_position_count == self.expected_receipt_count
            && self.journal_gap_count == 0
            && self.committed_truth_digest == self.replay_truth_digest
            && self.scanned_entry_count >= self.resolved_journal_position_count
            && !self.committed_truth_digest.is_empty()
            && !self.replay_truth_digest.is_empty()
            && !self.published_artifact_digest.is_empty()
            && !self.replay_outcome_digest.is_empty()
            && self.counter_snapshot.replay_residue_count() == 0
            && self.counter_snapshot.replay_admission_count() > 0
            && self.counter_snapshot.last_replay_outcome_digest()
                == Some(self.replay_outcome_digest.as_str())
    }

    pub fn replay_outcome_digest(&self) -> &str {
        &self.replay_outcome_digest
    }

    pub fn journal_segment_identity_digest(&self) -> &str {
        &self.journal_segment_identity_digest
    }

    pub fn replay_truth_digest(&self) -> &str {
        &self.replay_truth_digest
    }

    pub fn published_artifact_digest(&self) -> &str {
        &self.published_artifact_digest
    }

    pub fn counter_snapshot(&self) -> &WorthQueryJournalReplayCounterSnapshot {
        &self.counter_snapshot
    }

    #[cfg(test)]
    pub fn with_gap_for_sabotage(&self) -> Self {
        Self {
            journal_gap_count: self.journal_gap_count + 1,
            ..self.clone()
        }
    }

    #[cfg(test)]
    pub fn with_truth_mismatch_for_sabotage(&self) -> Self {
        Self {
            replay_truth_digest: format!("{}:sabotaged", self.replay_truth_digest),
            ..self.clone()
        }
    }
}

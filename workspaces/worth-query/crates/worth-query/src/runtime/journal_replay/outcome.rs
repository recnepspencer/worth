use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryJournalPositionSchedule, WorthQueryJournalSegmentIdentity, WorthQueryWriteReceipt,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryJournalReplayOutcome {
    segment_identity: WorthQueryJournalSegmentIdentity,
    write_receipts: Vec<WorthQueryWriteReceipt>,
    position_schedule: WorthQueryJournalPositionSchedule,
    expected_journal_position_count: usize,
    resolved_journal_position_count: usize,
    journal_gap_count: usize,
    scanned_entry_count: usize,
    truth_reconstruction_identity: WorthQueryEvidenceIdentity,
    published_artifact_digest: WorthQueryEvidenceIdentity,
    outcome_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryJournalReplayOutcome {
    pub(in crate::runtime) fn new(
        segment_identity: WorthQueryJournalSegmentIdentity,
        write_receipts: Vec<WorthQueryWriteReceipt>,
        expected_journal_position_count: usize,
        journal_gap_count: usize,
        scanned_entry_count: usize,
        truth_reconstruction_identity: WorthQueryEvidenceIdentity,
        published_artifact_digest: WorthQueryEvidenceIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let position_schedule = WorthQueryJournalPositionSchedule::derive(
            write_receipts
                .iter()
                .map(|receipt| receipt.journal_position().clone()),
        )?;
        let resolved_journal_position_count = write_receipts.len();
        let receipt_identities = write_receipts
            .iter()
            .map(|receipt| receipt.commit_evidence_identity().clone())
            .collect::<Vec<_>>();
        let outcome_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::JournalReplayOutcome)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("journal_segment_identity"),
                    segment_identity.identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("journal_position_schedule"),
                    position_schedule.schedule_digest(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("published_artifact_digest"),
                    &published_artifact_digest,
                )
                .field_evidence_identity_sequence(
                    WorthQueryEvidenceTag::new("replayed_write_commit_identity"),
                    receipt_identities.iter(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("truth_reconstruction_identity"),
                    &truth_reconstruction_identity,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("expected_journal_position_count"),
                    expected_journal_position_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("resolved_journal_position_count"),
                    resolved_journal_position_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("journal_gap_count"),
                    journal_gap_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("scanned_entry_count"),
                    scanned_entry_count,
                )
                .seal()?;
        Ok(Self {
            segment_identity,
            write_receipts,
            position_schedule,
            expected_journal_position_count,
            resolved_journal_position_count,
            journal_gap_count,
            scanned_entry_count,
            truth_reconstruction_identity,
            published_artifact_digest,
            outcome_identity,
        })
    }

    pub fn segment_identity(&self) -> &WorthQueryJournalSegmentIdentity {
        &self.segment_identity
    }

    pub fn write_receipts(&self) -> &[WorthQueryWriteReceipt] {
        &self.write_receipts
    }

    pub fn position_schedule(&self) -> &WorthQueryJournalPositionSchedule {
        &self.position_schedule
    }

    pub fn journal_gap_count(&self) -> usize {
        self.journal_gap_count
    }

    pub fn expected_journal_position_count(&self) -> usize {
        self.expected_journal_position_count
    }

    pub fn resolved_journal_position_count(&self) -> usize {
        self.resolved_journal_position_count
    }

    pub fn scanned_entry_count(&self) -> usize {
        self.scanned_entry_count
    }

    pub fn truth_reconstruction_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.truth_reconstruction_identity
    }

    pub fn published_artifact_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.published_artifact_digest
    }

    pub fn outcome_digest(&self) -> &str {
        self.outcome_identity.as_str()
    }

    pub fn outcome_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.outcome_identity
    }
}

pub(crate) fn journal_replay_truth_reconstruction_identity(
    committed_truth_identities: &[WorthQueryEvidenceIdentity],
    expected_journal_position_count: usize,
    journal_gap_count: usize,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::JournalReplayOutcome)
            .field_evidence_identity_sequence(
                WorthQueryEvidenceTag::new("committed_truth_identity"),
                committed_truth_identities.iter(),
            )
            .field_usize(
                WorthQueryEvidenceTag::new("expected_journal_position_count"),
                expected_journal_position_count,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("journal_gap_count"),
                journal_gap_count,
            )
            .seal()?,
    )
}

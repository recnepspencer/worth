use super::{
    BlobPublicationClassification, BlobPublicationClassificationDenial,
    BlobPublicationClassificationDenialKind, BlobPublicationCrashEdge, BlobPublicationCrashOutcome,
    BlobPublicationDurableWal, BlobPublicationEvidence, BlobPublicationObservationSet,
    BlobPublicationReplayCounterSnapshot,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationCrashBoundaryReport {
    outcome: BlobPublicationCrashOutcome,
    classification_digest: String,
    counters: BlobPublicationReplayCounterSnapshot,
    replayable: bool,
    replayable_durable_wal: Option<BlobPublicationDurableWal>,
    ambiguous: bool,
    observed_crash_edges: u64,
}

impl BlobPublicationCrashBoundaryReport {
    pub fn admit_observations(
        observations: BlobPublicationObservationSet,
    ) -> Result<Self, BlobPublicationClassificationDenial> {
        Self::admit_classification(&BlobPublicationClassification::classify_observations(
            observations,
        ))
    }

    pub fn admit_evidence(
        evidence: BlobPublicationEvidence,
    ) -> Result<Self, BlobPublicationClassificationDenial> {
        Self::admit_classification(&BlobPublicationClassification::classify(evidence))
    }

    pub fn admit_crash_edge(
        crash_edge: BlobPublicationCrashEdge,
    ) -> Result<Self, BlobPublicationClassificationDenial> {
        Self::admit_evidence(BlobPublicationEvidence::from_persisted_crash_edge(
            crash_edge,
        ))
    }

    pub fn admit_classification(
        classification: &BlobPublicationClassification,
    ) -> Result<Self, BlobPublicationClassificationDenial> {
        match classification.outcome() {
            BlobPublicationCrashOutcome::RejectedNonAuthoritativePromotion => Err(
                BlobPublicationClassificationDenial::new(
                    BlobPublicationClassificationDenialKind::
                        BackendResidueCannotStandInForCrashBoundaryAuthority,
                ),
            ),
            BlobPublicationCrashOutcome::CheckpointCutoverAmbiguous
            | BlobPublicationCrashOutcome::Ambiguous => Err(
                BlobPublicationClassificationDenial::new(
                    BlobPublicationClassificationDenialKind::
                        AmbiguousResidueCannotStandInForCrashBoundaryAuthority,
                ),
            ),
            _ => Ok(Self::from_classification(classification)),
        }
    }

    fn from_classification(classification: &BlobPublicationClassification) -> Self {
        Self {
            outcome: classification.outcome(),
            classification_digest: classification.classification_digest().to_owned(),
            counters: classification.counters(),
            replayable: classification
                .recovered_or_rejected()
                .is_replayable_without_promoting_acknowledgment(),
            replayable_durable_wal: classification
                .recovered_or_rejected()
                .replayable_durable_wal()
                .cloned(),
            ambiguous: matches!(
                classification.outcome(),
                BlobPublicationCrashOutcome::CheckpointCutoverAmbiguous
                    | BlobPublicationCrashOutcome::Ambiguous
            ),
            observed_crash_edges: classification.counters().observed_crash_edges() as u64,
        }
    }

    pub const fn outcome(&self) -> BlobPublicationCrashOutcome {
        self.outcome
    }

    pub fn classification_digest(&self) -> &str {
        &self.classification_digest
    }

    pub const fn counters(&self) -> BlobPublicationReplayCounterSnapshot {
        self.counters
    }

    pub const fn replayable(&self) -> bool {
        self.replayable
    }

    pub fn replayable_durable_wal(&self) -> Option<&BlobPublicationDurableWal> {
        self.replayable_durable_wal.as_ref()
    }

    pub const fn ambiguous(&self) -> bool {
        self.ambiguous
    }

    pub const fn observed_crash_edges(&self) -> u64 {
        self.observed_crash_edges
    }
}

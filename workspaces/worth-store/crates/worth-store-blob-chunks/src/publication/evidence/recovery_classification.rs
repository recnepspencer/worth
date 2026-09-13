use super::{
    BlobPublicationAmbiguityReport, BlobPublicationCrashEdge, BlobPublicationCrashOutcome,
    BlobPublicationEvidence, BlobPublicationEvidenceKind, BlobPublicationNonAuthoritativeDenial,
    BlobPublicationNonAuthoritativeSource, BlobPublicationObservationAdmission,
    BlobPublicationObservationSet, BlobPublicationRecoveredOrRejected,
    BlobPublicationReplayCounterSnapshot, BlobPublicationTornPublicationDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationClassification {
    outcome: BlobPublicationCrashOutcome,
    recovered_or_rejected: BlobPublicationRecoveredOrRejected,
    counters: BlobPublicationReplayCounterSnapshot,
    classification_digest: String,
    before_wal_append_operation_digest: Option<String>,
}

impl BlobPublicationClassification {
    pub fn classify_observations(observations: BlobPublicationObservationSet) -> Self {
        Self::classify(
            BlobPublicationObservationAdmission::admit_observations(observations).into_evidence(),
        )
    }

    pub fn classify(evidence: BlobPublicationEvidence) -> Self {
        match evidence.kind() {
            BlobPublicationEvidenceKind::PersistedCrashEdge(edge) => {
                classify_persisted_crash_edge(edge, evidence.persisted_digest())
            }
            BlobPublicationEvidenceKind::BackendResidueOnly { .. } => classify_non_authoritative(
                BlobPublicationNonAuthoritativeSource::BackendResidue,
                evidence.persisted_digest(),
                BlobPublicationReplayCounterSnapshot::default().with_rejected_residue_promotion(),
            ),
            BlobPublicationEvidenceKind::LiveAcknowledgmentMemoryOnly => {
                classify_non_authoritative(
                    BlobPublicationNonAuthoritativeSource::LiveAcknowledgmentMemory,
                    evidence.persisted_digest(),
                    BlobPublicationReplayCounterSnapshot::default()
                        .with_rejected_live_ack_promotion(),
                )
            }
            BlobPublicationEvidenceKind::LogOnly => classify_non_authoritative(
                BlobPublicationNonAuthoritativeSource::LogOnly,
                evidence.persisted_digest(),
                BlobPublicationReplayCounterSnapshot::default().with_rejected_log_only_promotion(),
            ),
            BlobPublicationEvidenceKind::TornPublication(denial) => {
                classify_torn_publication(denial.clone(), evidence.persisted_digest())
            }
            BlobPublicationEvidenceKind::InsufficientPersistedEvidence { ambiguity_digest } => {
                classify_ambiguity(ambiguity_digest)
            }
        }
    }

    pub const fn outcome(&self) -> BlobPublicationCrashOutcome {
        self.outcome
    }

    pub const fn recovered_or_rejected(&self) -> &BlobPublicationRecoveredOrRejected {
        &self.recovered_or_rejected
    }

    pub const fn counters(&self) -> BlobPublicationReplayCounterSnapshot {
        self.counters
    }

    pub fn classification_digest(&self) -> &str {
        &self.classification_digest
    }

    pub fn before_wal_append_operation_digest(&self) -> Option<&str> {
        self.before_wal_append_operation_digest.as_deref()
    }

    pub fn recover_or_reject_without_live_ack_memory(self) -> BlobPublicationRecoveredOrRejected {
        self.recovered_or_rejected
    }
}

fn classify_persisted_crash_edge(
    edge: &BlobPublicationCrashEdge,
    digest: &str,
) -> BlobPublicationClassification {
    let counters = BlobPublicationReplayCounterSnapshot::default().with_observed_crash_edge();
    match edge {
        BlobPublicationCrashEdge::BeforeWalAppend { operation_digest } => {
            classification_with_before(
                BlobPublicationCrashOutcome::NoWalAppendObserved,
                BlobPublicationRecoveredOrRejected::NoRecoveredWork { counters },
                counters,
                digest,
                operation_digest.clone(),
            )
        }
        BlobPublicationCrashEdge::AfterWalAppendBeforeDurability { .. } => classification(
            BlobPublicationCrashOutcome::WalAppendedButNotDurable,
            BlobPublicationRecoveredOrRejected::NoRecoveredWork { counters },
            counters,
            digest,
        ),
        BlobPublicationCrashEdge::AfterDurabilityBeforeAck { durable_wal } => {
            let counters = counters.with_replayable_durable_wal();
            classification(
                BlobPublicationCrashOutcome::DurableWalReplayable,
                BlobPublicationRecoveredOrRejected::ReplayableDurableWal {
                    durable_wal: durable_wal.clone(),
                    counters,
                },
                counters,
                digest,
            )
        }
        BlobPublicationCrashEdge::DuringCheckpointCutover { checkpoint_digest } => {
            let counters = counters.with_ambiguous_outcome();
            classification(
                BlobPublicationCrashOutcome::CheckpointCutoverAmbiguous,
                BlobPublicationRecoveredOrRejected::Ambiguous {
                    report: BlobPublicationAmbiguityReport::insufficient_persisted_evidence(
                        checkpoint_digest.clone(),
                    ),
                    counters,
                },
                counters,
                digest,
            )
        }
    }
}

fn classify_non_authoritative(
    source: BlobPublicationNonAuthoritativeSource,
    digest: &str,
    counters: BlobPublicationReplayCounterSnapshot,
) -> BlobPublicationClassification {
    classification(
        BlobPublicationCrashOutcome::RejectedNonAuthoritativePromotion,
        BlobPublicationRecoveredOrRejected::RejectedNonAuthoritativePromotion {
            denial: BlobPublicationNonAuthoritativeDenial::new(source, digest),
            counters,
        },
        counters,
        digest,
    )
}

fn classify_torn_publication(
    denial: BlobPublicationTornPublicationDenial,
    digest: &str,
) -> BlobPublicationClassification {
    let counters = BlobPublicationReplayCounterSnapshot::default().with_torn_publication_denial();
    classification(
        BlobPublicationCrashOutcome::TornPublicationRejected,
        BlobPublicationRecoveredOrRejected::RejectedTornPublication { denial, counters },
        counters,
        digest,
    )
}

fn classify_ambiguity(ambiguity_digest: &str) -> BlobPublicationClassification {
    let counters = BlobPublicationReplayCounterSnapshot::default().with_ambiguous_outcome();
    classification(
        BlobPublicationCrashOutcome::Ambiguous,
        BlobPublicationRecoveredOrRejected::Ambiguous {
            report: BlobPublicationAmbiguityReport::insufficient_persisted_evidence(
                ambiguity_digest,
            ),
            counters,
        },
        counters,
        ambiguity_digest,
    )
}

fn classification(
    outcome: BlobPublicationCrashOutcome,
    recovered_or_rejected: BlobPublicationRecoveredOrRejected,
    counters: BlobPublicationReplayCounterSnapshot,
    digest: &str,
) -> BlobPublicationClassification {
    BlobPublicationClassification {
        outcome,
        recovered_or_rejected,
        counters,
        classification_digest: format!("{outcome:?}:{digest}"),
        before_wal_append_operation_digest: None,
    }
}

fn classification_with_before(
    outcome: BlobPublicationCrashOutcome,
    recovered_or_rejected: BlobPublicationRecoveredOrRejected,
    counters: BlobPublicationReplayCounterSnapshot,
    digest: &str,
    operation_digest: String,
) -> BlobPublicationClassification {
    BlobPublicationClassification {
        outcome,
        recovered_or_rejected,
        counters,
        classification_digest: format!("{outcome:?}:{digest}"),
        before_wal_append_operation_digest: Some(operation_digest),
    }
}

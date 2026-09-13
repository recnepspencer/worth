use super::{
    BlobPublicationDurableWal, BlobPublicationNonAuthoritativeDenial,
    BlobPublicationReplayCounterSnapshot, BlobPublicationTornPublicationDenial,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobPublicationCrashOutcome {
    NoWalAppendObserved,
    WalAppendedButNotDurable,
    DurableWalReplayable,
    CheckpointCutoverAmbiguous,
    RejectedNonAuthoritativePromotion,
    TornPublicationRejected,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationAmbiguityReport {
    ambiguity_digest: String,
    counters: BlobPublicationReplayCounterSnapshot,
}

impl BlobPublicationAmbiguityReport {
    pub fn insufficient_persisted_evidence(ambiguity_digest: impl Into<String>) -> Self {
        Self {
            ambiguity_digest: ambiguity_digest.into(),
            counters: BlobPublicationReplayCounterSnapshot::default().with_ambiguous_outcome(),
        }
    }

    pub fn ambiguity_digest(&self) -> &str {
        &self.ambiguity_digest
    }

    pub const fn counters(&self) -> BlobPublicationReplayCounterSnapshot {
        self.counters
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobPublicationRecoveredOrRejected {
    NoRecoveredWork {
        counters: BlobPublicationReplayCounterSnapshot,
    },
    ReplayableDurableWal {
        durable_wal: BlobPublicationDurableWal,
        counters: BlobPublicationReplayCounterSnapshot,
    },
    RejectedTornPublication {
        denial: BlobPublicationTornPublicationDenial,
        counters: BlobPublicationReplayCounterSnapshot,
    },
    RejectedNonAuthoritativePromotion {
        denial: BlobPublicationNonAuthoritativeDenial,
        counters: BlobPublicationReplayCounterSnapshot,
    },
    Ambiguous {
        report: BlobPublicationAmbiguityReport,
        counters: BlobPublicationReplayCounterSnapshot,
    },
}

impl BlobPublicationRecoveredOrRejected {
    pub const fn is_replayable_without_promoting_acknowledgment(&self) -> bool {
        matches!(self, Self::ReplayableDurableWal { .. })
    }

    pub const fn replayable_durable_wal(&self) -> Option<&BlobPublicationDurableWal> {
        match self {
            Self::ReplayableDurableWal { durable_wal, .. } => Some(durable_wal),
            _ => None,
        }
    }

    pub const fn counters(&self) -> BlobPublicationReplayCounterSnapshot {
        match self {
            Self::NoRecoveredWork { counters }
            | Self::ReplayableDurableWal { counters, .. }
            | Self::RejectedTornPublication { counters, .. }
            | Self::RejectedNonAuthoritativePromotion { counters, .. }
            | Self::Ambiguous { counters, .. } => *counters,
        }
    }
}

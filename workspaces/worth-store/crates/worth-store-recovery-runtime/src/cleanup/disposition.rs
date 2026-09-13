use worth_store::physical_runtime::recovery_wal::{WalLsnRange, WalSegmentArtifactIdentity};
use worth_store_physical_format::{PhysicalCheckpointIdentity, RecordArtifactFile};
use worth_store_recovery_physics::PhysicalRecoveryResidueKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecoveryCleanupTarget {
    Record(RecordArtifactFile),
    Checkpoint(PhysicalCheckpointIdentity),
    Wal(WalSegmentArtifactIdentity),
    Residue {
        name: Box<str>,
        kind: PhysicalRecoveryResidueKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCleanupDispositionKind {
    Current,
    Retained,
    Eligible,
    Deferred(RecoveryCleanupDeferralReason),
    QuarantinedOrUnsupported,
    SafelyRemoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryCleanupDeferralReason {
    CandidateLimit,
    ByteLimit,
    UnresolvedOperationFate,
    FreshnessUnavailable,
    PublishedGenerationChanged,
    EligibilityChanged,
    Cancelled,
    CancellationBindingMismatch,
    DeniedBeforeEffect,
    IndeterminateEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryCleanupDisposition {
    target: RecoveryCleanupTarget,
    kind: RecoveryCleanupDispositionKind,
    wal_range: Option<WalLsnRange>,
    byte_count: u64,
    wal_digest: Option<[u8; 32]>,
}

impl RecoveryCleanupDisposition {
    pub(crate) const fn new(
        target: RecoveryCleanupTarget,
        kind: RecoveryCleanupDispositionKind,
        wal_range: Option<WalLsnRange>,
        byte_count: u64,
        wal_digest: Option<[u8; 32]>,
    ) -> Self {
        Self {
            target,
            kind,
            wal_range,
            byte_count,
            wal_digest,
        }
    }

    pub fn target(&self) -> &RecoveryCleanupTarget {
        &self.target
    }

    pub const fn kind(&self) -> RecoveryCleanupDispositionKind {
        self.kind
    }

    pub const fn wal_range(&self) -> Option<WalLsnRange> {
        self.wal_range
    }

    pub const fn byte_count(&self) -> u64 {
        self.byte_count
    }

    pub const fn wal_digest(&self) -> Option<[u8; 32]> {
        self.wal_digest
    }

    pub(crate) fn transition_eligible(&mut self, kind: RecoveryCleanupDispositionKind) -> bool {
        if self.kind != RecoveryCleanupDispositionKind::Eligible {
            return false;
        }
        self.kind = kind;
        true
    }
}

use worth_store_physical_backend::{
    BackendDurabilityProfile, BackendDurabilityProfileId, WalAppendReceipt,
    WalDurabilityBarrierSet, WalFrameDigest,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange, WalSegmentGeneration, WalSegmentId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationDurableWal {
    profile_id: BackendDurabilityProfileId,
    segment_id: WalSegmentId,
    generation: WalSegmentGeneration,
    lsn_range: WalLsnRange,
    frame_digest: WalFrameDigest,
    expected_bytes: u64,
    required_barriers: WalDurabilityBarrierSet,
    completed_barriers: WalDurabilityBarrierSet,
}

impl BlobPublicationDurableWal {
    pub fn from_append_receipt<P: BackendDurabilityProfile>(receipt: WalAppendReceipt<P>) -> Self {
        Self {
            profile_id: receipt.profile_id(),
            segment_id: receipt.segment_id(),
            generation: receipt.generation(),
            lsn_range: receipt.lsn_range(),
            frame_digest: receipt.frame_digest().clone(),
            expected_bytes: receipt.expected_bytes(),
            required_barriers: receipt.required_barriers(),
            completed_barriers: receipt.completed_barriers(),
        }
    }

    pub const fn profile_id(&self) -> BackendDurabilityProfileId {
        self.profile_id
    }

    pub const fn segment_id(&self) -> WalSegmentId {
        self.segment_id
    }

    pub const fn generation(&self) -> WalSegmentGeneration {
        self.generation
    }

    pub const fn lsn_range(&self) -> WalLsnRange {
        self.lsn_range
    }

    pub fn frame_digest(&self) -> &WalFrameDigest {
        &self.frame_digest
    }

    pub const fn expected_bytes(&self) -> u64 {
        self.expected_bytes
    }

    pub const fn required_barriers(&self) -> WalDurabilityBarrierSet {
        self.required_barriers
    }

    pub const fn completed_barriers(&self) -> WalDurabilityBarrierSet {
        self.completed_barriers
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobPublicationCrashEdge {
    BeforeWalAppend {
        operation_digest: String,
    },
    AfterWalAppendBeforeDurability {
        wal_range: WalLsnRange,
        operation_digest: String,
    },
    AfterDurabilityBeforeAck {
        durable_wal: BlobPublicationDurableWal,
    },
    DuringCheckpointCutover {
        checkpoint_digest: String,
    },
}

impl BlobPublicationCrashEdge {
    pub fn before_wal_append(operation_digest: impl Into<String>) -> Self {
        Self::BeforeWalAppend {
            operation_digest: operation_digest.into(),
        }
    }

    pub fn after_wal_append_before_durability(
        wal_range: WalLsnRange,
        operation_digest: impl Into<String>,
    ) -> Self {
        Self::AfterWalAppendBeforeDurability {
            wal_range,
            operation_digest: operation_digest.into(),
        }
    }

    pub fn after_durability_before_ack<P: BackendDurabilityProfile>(
        receipt: WalAppendReceipt<P>,
    ) -> Self {
        Self::AfterDurabilityBeforeAck {
            durable_wal: BlobPublicationDurableWal::from_append_receipt(receipt),
        }
    }

    pub fn during_checkpoint_cutover(checkpoint_digest: impl Into<String>) -> Self {
        Self::DuringCheckpointCutover {
            checkpoint_digest: checkpoint_digest.into(),
        }
    }

    pub const fn first_lsn(&self) -> Option<LogSequenceNumber> {
        match self {
            Self::BeforeWalAppend { .. } | Self::DuringCheckpointCutover { .. } => None,
            Self::AfterWalAppendBeforeDurability { wal_range, .. } => Some(wal_range.start()),
            Self::AfterDurabilityBeforeAck { durable_wal } => Some(durable_wal.lsn_range().start()),
        }
    }
}

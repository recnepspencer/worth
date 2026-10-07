use std::sync::Arc;

use worth_store_physical_format::{
    CheckpointStreamFooter, PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};

use super::{
    PhysicalCheckpointCaptureBasis, PhysicalCheckpointCaptureFailureKind,
    PhysicalCheckpointIdempotencyKey, PhysicalCheckpointProgress,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedPhysicalCheckpoint {
    basis: PhysicalCheckpointCaptureBasis,
    footer: CheckpointStreamFooter,
    encoded_bytes: u64,
    dirty_records: u64,
    retained_wal_tail: Arc<super::ContiguousRetainedWalTail>,
    binding_compaction: crate::physical_runtime::PhysicalMutationBindingCompaction,
    wal_reclamation: crate::physical_runtime::PhysicalWalReclamationObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalCheckpointProvenNoEffectCause {
    CancelledBeforeCandidate,
    CancelledAndCandidateRemoved,
    FailedAndCandidateRemoved(PhysicalCheckpointCaptureFailureKind),
    DeniedBeforeCandidate(PhysicalCheckpointCaptureFailureKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenNoEffectPhysicalCheckpoint {
    identity: PhysicalCheckpointIdentity,
    idempotency: PhysicalCheckpointIdempotencyKey,
    cause: PhysicalCheckpointProvenNoEffectCause,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndeterminatePhysicalCheckpoint {
    identity: PhysicalCheckpointIdentity,
    idempotency: PhysicalCheckpointIdempotencyKey,
    failure: PhysicalCheckpointCaptureFailureKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalCheckpointOutcome {
    Completed(CompletedPhysicalCheckpoint),
    ProvenNoEffect(ProvenNoEffectPhysicalCheckpoint),
    Indeterminate(IndeterminatePhysicalCheckpoint),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalCheckpointPoll {
    Pending(PhysicalCheckpointProgress),
    Terminal(PhysicalCheckpointOutcome),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalCheckpointCancellationOutcome {
    Accepted {
        identity: PhysicalCheckpointIdentity,
    },
    PublicationAlreadyEffectful {
        identity: PhysicalCheckpointIdentity,
    },
    AlreadyTerminal(PhysicalCheckpointOutcome),
    RuntimeClosing {
        identity: PhysicalCheckpointIdentity,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalCheckpointDisposal {
    ObservationAbandoned {
        identity: PhysicalCheckpointIdentity,
    },
    Terminal(PhysicalCheckpointOutcome),
}

impl CompletedPhysicalCheckpoint {
    pub(super) const fn new(
        basis: PhysicalCheckpointCaptureBasis,
        footer: CheckpointStreamFooter,
        encoded_bytes: u64,
        dirty_records: u64,
        retained_wal_tail: Arc<super::ContiguousRetainedWalTail>,
        binding_compaction: crate::physical_runtime::PhysicalMutationBindingCompaction,
        wal_reclamation: crate::physical_runtime::PhysicalWalReclamationObservation,
    ) -> Self {
        Self {
            basis,
            footer,
            encoded_bytes,
            dirty_records,
            retained_wal_tail,
            binding_compaction,
            wal_reclamation,
        }
    }

    pub const fn basis(&self) -> PhysicalCheckpointCaptureBasis {
        self.basis
    }

    pub const fn footer(&self) -> CheckpointStreamFooter {
        self.footer
    }

    pub const fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    pub const fn dirty_records(&self) -> u64 {
        self.dirty_records
    }

    pub fn retained_wal_tail(&self) -> &super::ContiguousRetainedWalTail {
        &self.retained_wal_tail
    }

    pub const fn binding_compaction(
        &self,
    ) -> &crate::physical_runtime::PhysicalMutationBindingCompaction {
        &self.binding_compaction
    }

    pub const fn wal_reclamation(
        &self,
    ) -> crate::physical_runtime::PhysicalWalReclamationObservation {
        self.wal_reclamation
    }
}

impl ProvenNoEffectPhysicalCheckpoint {
    pub(super) const fn new(
        identity: PhysicalCheckpointIdentity,
        idempotency: PhysicalCheckpointIdempotencyKey,
        cause: PhysicalCheckpointProvenNoEffectCause,
    ) -> Self {
        Self {
            identity,
            idempotency,
            cause,
        }
    }

    pub const fn identity(&self) -> PhysicalCheckpointIdentity {
        self.identity
    }

    pub const fn idempotency_key(&self) -> PhysicalCheckpointIdempotencyKey {
        self.idempotency
    }

    pub fn cause(&self) -> PhysicalCheckpointProvenNoEffectCause {
        self.cause.clone()
    }
}

impl IndeterminatePhysicalCheckpoint {
    pub(super) const fn new(
        identity: PhysicalCheckpointIdentity,
        idempotency: PhysicalCheckpointIdempotencyKey,
        failure: PhysicalCheckpointCaptureFailureKind,
    ) -> Self {
        Self {
            identity,
            idempotency,
            failure,
        }
    }

    pub const fn identity(&self) -> PhysicalCheckpointIdentity {
        self.identity
    }

    pub const fn idempotency_key(&self) -> PhysicalCheckpointIdempotencyKey {
        self.idempotency
    }

    pub fn failure(&self) -> PhysicalCheckpointCaptureFailureKind {
        self.failure.clone()
    }
}

impl PhysicalCheckpointOutcome {
    pub const fn identity(&self) -> PhysicalCheckpointIdentity {
        match self {
            Self::Completed(completed) => completed.basis().identity(),
            Self::ProvenNoEffect(no_effect) => no_effect.identity(),
            Self::Indeterminate(indeterminate) => indeterminate.identity(),
        }
    }

    pub const fn source(&self) -> Option<PhysicalCheckpointSource> {
        match self {
            Self::Completed(completed) => Some(completed.basis().source()),
            Self::ProvenNoEffect(_) | Self::Indeterminate(_) => None,
        }
    }
}

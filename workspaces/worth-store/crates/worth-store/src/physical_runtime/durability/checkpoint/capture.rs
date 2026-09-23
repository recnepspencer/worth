use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

use worth_store_buffer_pool::PhysicalDirtyGenerationCaptureSession;
use worth_store_physical_format::{
    CheckpointRootBasis, PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};

use super::{
    PhysicalCheckpointCaptureFailure, PhysicalCheckpointCaptureFailureKind,
    PhysicalCheckpointWorkPort,
};
use crate::physical_runtime::record_serving::{RecordFramePorts, RecordPublicationDirector};

mod execution;
mod publication_cutover;
mod streaming;
pub(super) use execution::PhysicalCheckpointExecutionResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalCheckpointCaptureBasis {
    source: PhysicalCheckpointSource,
    policy: crate::physical_runtime::PhysicalDurabilityPolicyIdentity,
}

pub(in crate::physical_runtime) struct PhysicalCheckpointCaptureFoundation {
    pub(in crate::physical_runtime) publication: Arc<RecordPublicationDirector>,
    pub(in crate::physical_runtime) wal: crate::physical_runtime::durability::PhysicalWalAppendPort,
    pub(in crate::physical_runtime) binding_compaction:
        crate::physical_runtime::durability::PhysicalMutationBindingCompactionRuntimeAuthority,
    pub(in crate::physical_runtime) frames: RecordFramePorts,
    pub(in crate::physical_runtime) work: PhysicalCheckpointWorkPort,
    pub(in crate::physical_runtime) durability:
        crate::physical_runtime::PhysicalDurabilityObservation,
    pub(in crate::physical_runtime) reclamation:
        crate::physical_runtime::durability::PhysicalWalReclamationOwner,
}

pub(super) struct PhysicalCheckpointCaptureOwner {
    store: worth_store_physical_format::store_namespace::StableStoreIdentity,
    next_sequence: AtomicU64,
    policy: crate::physical_runtime::PhysicalDurabilityPolicyIdentity,
    checkpoint_policy: crate::physical_runtime::PhysicalCheckpointPolicy,
    idempotency_policy: crate::physical_runtime::PhysicalIdempotencyPolicy,
    publication: Weak<RecordPublicationDirector>,
    wal: crate::physical_runtime::durability::PhysicalWalAppendPort,
    binding_compaction:
        crate::physical_runtime::durability::PhysicalMutationBindingCompactionRuntimeAuthority,
    frames: RecordFramePorts,
    work: PhysicalCheckpointWorkPort,
    reclamation: crate::physical_runtime::durability::PhysicalWalReclamationOwner,
}

pub(super) struct AdmittedPhysicalCheckpointCapture {
    basis: PhysicalCheckpointCaptureBasis,
    session: PhysicalDirtyGenerationCaptureSession,
}

impl PhysicalCheckpointCaptureBasis {
    fn new(
        source: PhysicalCheckpointSource,
        policy: crate::physical_runtime::PhysicalDurabilityPolicyIdentity,
    ) -> Self {
        Self { source, policy }
    }

    pub const fn identity(self) -> PhysicalCheckpointIdentity {
        self.source.identity()
    }

    pub const fn source(self) -> PhysicalCheckpointSource {
        self.source
    }

    pub const fn policy_identity(
        self,
    ) -> crate::physical_runtime::PhysicalDurabilityPolicyIdentity {
        self.policy
    }
}

impl PhysicalCheckpointCaptureOwner {
    pub(in crate::physical_runtime) fn new(
        foundation: PhysicalCheckpointCaptureFoundation,
    ) -> Self {
        Self {
            store: foundation.durability.store_identity(),
            next_sequence: AtomicU64::new(1),
            policy: foundation.durability.policy_identity(),
            checkpoint_policy: foundation.durability.checkpoint_policy(),
            idempotency_policy: foundation.durability.idempotency_policy(),
            publication: Arc::downgrade(&foundation.publication),
            wal: foundation.wal,
            binding_compaction: foundation.binding_compaction,
            frames: foundation.frames,
            work: foundation.work,
            reclamation: foundation.reclamation,
        }
    }

    pub(super) fn admit(
        &self,
    ) -> Result<AdmittedPhysicalCheckpointCapture, PhysicalCheckpointCaptureFailure> {
        let publication = self.publication.upgrade().ok_or(
            PhysicalCheckpointCaptureFailure::before_candidate(
                PhysicalCheckpointCaptureFailureKind::RuntimeUnavailable,
            ),
        )?;
        let root = publication.current_root();
        let wal = self.wal.checkpoint_source_range().ok_or(
            PhysicalCheckpointCaptureFailure::before_candidate(
                PhysicalCheckpointCaptureFailureKind::NoDurableWalSource,
            ),
        )?;
        let session = self.frames.begin_checkpoint_capture().map_err(|_denial| {
            PhysicalCheckpointCaptureFailure::before_candidate(
                PhysicalCheckpointCaptureFailureKind::ResidencyUnavailable,
            )
        })?;
        let identity = self.next_identity()?;
        if session.store_identity() != self.store || identity.store_identity() != self.store {
            return Err(PhysicalCheckpointCaptureFailure::before_candidate(
                PhysicalCheckpointCaptureFailureKind::SourceAuthorityMismatch,
            ));
        }
        let mut source = PhysicalCheckpointSource::secured_concurrent(
            identity,
            wal,
            CheckpointRootBasis::new(root.generation(), root.tree_identity()),
            session.frontier().get(),
            self.policy.bytes(),
            self.idempotency_policy.retention().get().get(),
        )
        .expect("an admitted durability policy has a nonzero canonical security binding");
        if root.requires_maintenance_protocol() {
            source = source.with_maintenance_protocol();
        }
        Ok(AdmittedPhysicalCheckpointCapture {
            basis: PhysicalCheckpointCaptureBasis::new(source, self.policy),
            session,
        })
    }

    fn next_identity(
        &self,
    ) -> Result<PhysicalCheckpointIdentity, PhysicalCheckpointCaptureFailure> {
        let sequence = self
            .next_sequence
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(1)
            })
            .map_err(|_| {
                PhysicalCheckpointCaptureFailure::before_candidate(
                    PhysicalCheckpointCaptureFailureKind::SequenceExhausted,
                )
            })?;
        let sequence =
            NonZeroU64::new(sequence).ok_or(PhysicalCheckpointCaptureFailure::before_candidate(
                PhysicalCheckpointCaptureFailureKind::SequenceExhausted,
            ))?;
        Ok(PhysicalCheckpointIdentity::new(self.store, sequence))
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_checkpoint_under_pressure(
        &self,
        foreground_pressure_events: u64,
    ) -> bool {
        let Ok(identity) = self.next_identity() else {
            return false;
        };
        matches!(
            self.work.execute(
                identity,
                crate::physical_runtime::work::PhysicalCheckpointWorkAction::CreateCandidate {
                    byte_count: 64,
                },
                None,
                foreground_pressure_events,
            ),
            Err(super::PhysicalCheckpointActionFailure::BackgroundYielded)
        )
    }

    pub(in crate::physical_runtime) fn release_background_selection(&self) {
        self.work.release_background_selection();
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn fail_next_admission(&self) {
        self.work.fail_next_admission_after_noting_head();
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_reclamation_under_pressure(
        &self,
        foreground_pressure_events: u64,
    ) -> bool {
        let Ok(identity) = self.next_identity() else {
            return false;
        };
        self.reclamation
            .certification_under_pressure(identity, foreground_pressure_events)
    }
}

impl AdmittedPhysicalCheckpointCapture {
    pub(super) const fn basis(&self) -> PhysicalCheckpointCaptureBasis {
        self.basis
    }
}

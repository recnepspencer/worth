use std::sync::Arc;

use super::execution::{
    finish_capture_candidate, remove_created_candidate, remove_durable_candidate,
    synchronize_capture_candidate, PhysicalCheckpointExecutionResult,
};
use super::{PhysicalCheckpointCaptureBasis, PhysicalCheckpointCaptureOwner};
use crate::physical_runtime::durability::checkpoint::publication::{
    CreatedCheckpointCandidate, DurableCheckpointCandidate,
    PhysicalCheckpointNamespaceFinalizationFailure,
};
use crate::physical_runtime::durability::checkpoint::{
    ContiguousRetainedWalTail, PhysicalCheckpointAttempt, PhysicalCheckpointCaptureFailure,
    PhysicalCheckpointCaptureFailureKind, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointProgressPhase, PhysicalCheckpointProvenNoEffectCause,
    RetainedWalTailAdmissionDenial,
};
use crate::physical_runtime::durability::wal::PhysicalWalCheckpointCutover;

struct CheckpointPublicationContext<'attempt> {
    attempt: &'attempt PhysicalCheckpointAttempt,
    basis: PhysicalCheckpointCaptureBasis,
    key: PhysicalCheckpointIdempotencyKey,
}

impl PhysicalCheckpointCaptureOwner {
    pub(super) fn finalize_and_publish_candidate(
        &self,
        candidate: CreatedCheckpointCandidate,
        attempt: &PhysicalCheckpointAttempt,
        basis: PhysicalCheckpointCaptureBasis,
        key: PhysicalCheckpointIdempotencyKey,
    ) -> PhysicalCheckpointExecutionResult {
        let context = CheckpointPublicationContext {
            attempt,
            basis,
            key,
        };
        let publication = match self.publication.upgrade() {
            Some(publication) => publication,
            None => {
                return remove_created_after_tail_denial(
                    candidate,
                    context,
                    PhysicalCheckpointCaptureFailureKind::RuntimeUnavailable,
                )
            }
        };
        let pins = match publication.selected_blob_manifest_pins(
            self.idempotency_policy.live_binding_limit().get().get() as usize,
            self.checkpoint_policy.memory_limit().get().get(),
        ) {
            Ok(pins) => pins,
            Err(_) => {
                return remove_created_after_tail_denial(
                    candidate,
                    context,
                    PhysicalCheckpointCaptureFailureKind::BindingCompactionUnavailable,
                )
            }
        };
        let cutover = match self.wal.checkpoint_cutover() {
            Some(cutover) => cutover,
            None => return remove_created_without_tail(candidate, context),
        };
        let tail = match self.admit_retained_tail(context.basis, &cutover) {
            Ok(tail) => tail,
            Err(kind) => {
                drop(cutover);
                return remove_created_after_tail_denial(candidate, context, kind);
            }
        };
        // The WAL cutover precedes the registry lock. A root change between
        // protected selection and this check invalidates the roster.
        let selected_root = publication.current_root();
        if selected_root.root_cell() != pins.root()
            || selected_root.generation() != context.basis.source().root().generation()
            || selected_root.tree_identity() != context.basis.source().root().tree_identity()
        {
            drop(cutover);
            return remove_created_after_tail_denial(
                candidate,
                context,
                PhysicalCheckpointCaptureFailureKind::BindingCompactionUnavailable,
            );
        }
        let Some(maximum_binding_bytes) = u64::from(
            self.idempotency_policy.live_binding_limit().get().get(),
        )
        .checked_mul(worth_store_physical_format::MAX_CHECKPOINT_BINDING_RECORD_BYTES as u64) else {
            drop(cutover);
            return remove_created_after_tail_denial(
                candidate,
                context,
                PhysicalCheckpointCaptureFailureKind::BindingCompactionUnavailable,
            );
        };
        let binding_cutover = match self.binding_compaction.begin_binding_compaction(
            context.basis.identity(),
            tail.durable_tail_end_lsn_exclusive().get(),
            pins,
            maximum_binding_bytes,
        ) {
            Ok(cutover) => cutover,
            Err(_) => {
                drop(cutover);
                return remove_created_after_tail_denial(
                    candidate,
                    context,
                    PhysicalCheckpointCaptureFailureKind::BindingCompactionUnavailable,
                );
            }
        };
        let captured = match finish_capture_candidate(
            candidate,
            &binding_cutover,
            context.attempt,
            context.key,
        ) {
            Ok(captured) => captured,
            Err(terminal) => return terminal,
        };
        // The candidate file sync is FlushFileBuffers. The WAL runtime mutex
        // and this registry are the foreground append path: a member completion
        // holds the WAL lock and then records its binding. Keeping either
        // across that sync stalls every mutation for the device barrier.
        let pending = binding_cutover.release_registry();
        drop(cutover);
        let durable = match synchronize_capture_candidate(captured, context.attempt, context.key) {
            Ok(durable) => durable,
            Err(terminal) => return terminal,
        };
        let Some(cutover) = self.wal.checkpoint_cutover() else {
            return remove_durable_candidate(
                durable,
                context.attempt,
                context.key,
                PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(
                    PhysicalCheckpointCaptureFailureKind::RetainedWalTailUnavailable,
                ),
            );
        };
        let binding_cutover = match self.binding_compaction.resume_binding_compaction(pending) {
            Ok(cutover) => cutover,
            Err(_) => {
                drop(cutover);
                return remove_durable_candidate(
                    durable,
                    context.attempt,
                    context.key,
                    PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(
                        PhysicalCheckpointCaptureFailureKind::BindingCompactionUnavailable,
                    ),
                );
            }
        };
        if !context.attempt.begin_publication() {
            drop(cutover);
            drop(binding_cutover);
            return remove_durable_candidate(
                durable,
                context.attempt,
                context.key,
                PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved,
            );
        }
        publish_under_cutover(
            durable,
            context,
            cutover,
            binding_cutover,
            tail,
            &publication,
            &self.reclamation,
        )
    }

    fn admit_retained_tail(
        &self,
        basis: PhysicalCheckpointCaptureBasis,
        cutover: &PhysicalWalCheckpointCutover<'_>,
    ) -> Result<Arc<ContiguousRetainedWalTail>, PhysicalCheckpointCaptureFailureKind> {
        ContiguousRetainedWalTail::from_inventory(
            basis.source(),
            &cutover.inventory_snapshot(),
            self.checkpoint_policy.retained_wal_tail_limit(),
        )
        .map(Arc::new)
        .map_err(|denial| match denial {
            RetainedWalTailAdmissionDenial::RetainedByteLimitExceeded => {
                PhysicalCheckpointCaptureFailureKind::RetainedWalTailLimitExceeded
            }
            _ => PhysicalCheckpointCaptureFailureKind::RetainedWalTailUnavailable,
        })
    }
}

fn publish_under_cutover(
    durable: DurableCheckpointCandidate,
    context: CheckpointPublicationContext<'_>,
    cutover: PhysicalWalCheckpointCutover<'_>,
    binding_cutover: crate::physical_runtime::durability::PhysicalMutationBindingCompactionCutover<
        '_,
    >,
    tail: Arc<ContiguousRetainedWalTail>,
    publication_owner: &crate::physical_runtime::record_serving::RecordPublicationDirector,
    reclamation: &crate::physical_runtime::durability::PhysicalWalReclamationOwner,
) -> PhysicalCheckpointExecutionResult {
    let replaced = match durable.publish() {
        Ok(replaced) => replaced,
        Err((durable, failure)) => {
            drop(cutover);
            let capture = PhysicalCheckpointCaptureFailure::from_initial_action(failure);
            if capture.requires_inspection() {
                return PhysicalCheckpointExecutionResult::indeterminate(
                    context.basis.identity(),
                    context.key,
                    capture.kind(),
                );
            }
            return remove_durable_candidate(
                durable,
                context.attempt,
                context.key,
                PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(capture.kind()),
            );
        }
    };
    context
        .attempt
        .enter(PhysicalCheckpointProgressPhase::NamespaceSynchronization);
    let result = match replaced.synchronize_namespace(tail, binding_cutover) {
        Ok(publication) => {
            if publication_owner
                .commit_selected_checkpoint_custody(&publication)
                .is_err()
            {
                drop(cutover);
                return PhysicalCheckpointExecutionResult::indeterminate(
                    context.basis.identity(),
                    context.key,
                    PhysicalCheckpointCaptureFailureKind::CheckpointCustodyUnavailable,
                );
            }
            let checkpoint = publication.basis().identity();
            let plan = cutover.reclamation_plan(&publication);
            drop(cutover);
            let observation = match plan {
                Ok(plan) => reclamation.execute(plan),
                Err(_) => reclamation.eligibility_denied(checkpoint),
            };
            return PhysicalCheckpointExecutionResult::completed(
                publication.with_wal_reclamation(observation),
            );
        }
        Err(PhysicalCheckpointNamespaceFinalizationFailure::Action(_failure)) => {
            PhysicalCheckpointExecutionResult::indeterminate(
                context.basis.identity(),
                context.key,
                PhysicalCheckpointCaptureFailureKind::CandidateContinuationFailed,
            )
        }
        Err(PhysicalCheckpointNamespaceFinalizationFailure::BindingCompaction) => {
            PhysicalCheckpointExecutionResult::indeterminate(
                context.basis.identity(),
                context.key,
                PhysicalCheckpointCaptureFailureKind::BindingCompactionCommitFailed,
            )
        }
    };
    drop(cutover);
    result
}

fn remove_created_without_tail(
    candidate: CreatedCheckpointCandidate,
    context: CheckpointPublicationContext<'_>,
) -> PhysicalCheckpointExecutionResult {
    remove_created_after_tail_denial(
        candidate,
        context,
        PhysicalCheckpointCaptureFailureKind::RetainedWalTailUnavailable,
    )
}

fn remove_created_after_tail_denial(
    candidate: CreatedCheckpointCandidate,
    context: CheckpointPublicationContext<'_>,
    kind: PhysicalCheckpointCaptureFailureKind,
) -> PhysicalCheckpointExecutionResult {
    remove_created_candidate(
        candidate,
        context.attempt,
        context.key,
        PhysicalCheckpointProvenNoEffectCause::FailedAndCandidateRemoved(kind),
    )
}

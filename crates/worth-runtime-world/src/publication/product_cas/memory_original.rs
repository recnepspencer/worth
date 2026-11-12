//! Capture root-free original coordinates before the only product movement.
use super::CompositePublicationReady;
use std::sync::{atomic::AtomicBool, Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::MemoryProductBranchReference;
use worth_relational::facade::branch::{
    RelationalBranchBasisPort, RelationalRecordedComponentRefusal as Refusal,
};

/// Only actual capture (or an explicit retained-Native leg) admits ordinary movement.
#[derive(Debug)]
pub(crate) struct MemoryPublicationReady {
    ready: CompositePublicationReady,
}
impl MemoryPublicationReady {
    pub(crate) fn publish_controlled(
        self,
        cell: &crate::branch::ProductBranchReferenceCell,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
        clock: &crate::lifecycle::RuntimeWorldClock,
        control: &crate::lifecycle::RuntimeWorldOperationControlForTest,
    ) -> crate::publication::RuntimeWorldPublicationOutcome {
        self.ready
            .publish_controlled_after_capture(cell, cancellation, clock, control)
    }
    pub(crate) fn expected_head(&self) -> &crate::branch::ProductBranchObservation {
        self.ready.expected_head()
    }
    pub(crate) fn retain(
        self,
        cause: crate::recovery::ProductUnpublishedCause,
    ) -> crate::publication::RuntimeWorldPublicationOutcome {
        self.ready.retain(cause)
    }
}
impl CompositePublicationReady {
    pub(crate) fn capture_memory_original(
        self,
        execution: &ExecutionRequest<'_, '_>,
        history: &crate::history::CompositeHistoryCatalog,
        port: &RelationalBranchBasisPort,
    ) -> Result<MemoryPublicationReady, (Self, Refusal)> {
        match self.prepare_memory_original(execution, history, port) {
            Ok(()) => Ok(MemoryPublicationReady { ready: self }),
            Err(cause) => Err((self, cause)),
        }
    }

    fn prepare_memory_original(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        history: &crate::history::CompositeHistoryCatalog,
        port: &RelationalBranchBasisPort,
    ) -> Result<(), Refusal> {
        let envelope = self.custody.publication_envelope();
        if envelope.has_memory_original() {
            return Ok(());
        }
        let Some((native, _, _)) = self.original_relational_parts() else {
            return Ok(());
        };
        let expected = self.expected_head();
        // An adoption of an already selected Native original, like a retained
        // leg, cannot create a second World origin for the same Native effect.
        if expected
            .basis()
            .relational_basis()
            .reference()
            .target()
            .as_basis()
            .is_some_and(|prior| prior.selected_commit_id() == native.commit_id().0)
        {
            return Ok(());
        }
        let bytes = std::mem::size_of::<crate::history::MemoryPublicationRecord>()
            + 2 * std::mem::size_of::<usize>();
        let funding = execution
            .reserve_retained_memory(bytes as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let meaning_bytes = std::mem::size_of::<crate::history::MemoryPublicationMeaning>()
            + 2 * expected.branch_identity().name().as_str().len()
            + 2 * std::mem::size_of::<usize>();
        let meaning_funding = execution
            .reserve_retained_memory(meaning_bytes as u64)
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let mut charged = None;
        let scope = execution.run_native_with_checkpoint(|context| {
            charged = Some(if context.has_accounting() {
                context.checkpoint((bytes + meaning_bytes) as u64)
            } else {
                context.poll()
            });
        });
        match (scope, charged) {
            (Ok(_), Some(Ok(()))) => {}
            (_, Some(Err(cause))) => return Err(Refusal::Request(cause.into())),
            (Err(cause), _) => return Err(Refusal::Request(cause.into())),
            (Ok(_), None) => unreachable!("original capture charged before copying"),
        }
        let head = |generation, commit| {
            MemoryProductBranchReference::from_observed(
                expected.owner_identity().get(),
                expected.branch_identity().name().as_str().to_owned(),
                expected.lifecycle_incarnation().ordinal(),
                generation,
                commit,
            )
            .expect("an actual World branch has a valid name")
        };
        let meaning = Arc::new(crate::history::MemoryPublicationMeaning {
            before: head(
                expected.reference_generation().get(),
                expected.selected_commit().ordinal(),
            ),
            after: head(
                expected
                    .reference_generation()
                    .advance()
                    .expect("generation admitted before effects")
                    .get(),
                self.commit.identity().ordinal(),
            ),
            attempt:
                worth_foundational::facade::RecordedCompositePublicationAttemptIdentity::Memory {
                    owner: self.attempt_identity.owner_identity().get(),
                    ordinal: self.attempt_identity.ordinal(),
                },
            _funding: meaning_funding,
        });
        let record = Arc::new(crate::history::MemoryPublicationRecord {
            meaning,
            receiver: crate::history::MemoryPublicationReceiver::Live,
            before_component: port
                .observe_memory_component(execution, expected.basis().relational_basis())?,
            after_component: port
                .observe_memory_component(execution, self.commit.basis().relational_basis())?,
            committed: AtomicBool::new(false),
            _funding: funding,
        });
        envelope.install_memory_original(history.register_memory_original(execution, record)?);
        Ok(())
    }
}

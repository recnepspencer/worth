//! Seal root-free original coordinates before the only product movement.
//! Their capture was admitted before the attempt's first owner effect.
use super::CompositePublicationReady;
use worth_execution::ExecutionRequest;
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
        envelope.seal_memory_original(
            execution,
            history,
            port,
            expected,
            self.commit.basis().relational_basis(),
        )
    }
}

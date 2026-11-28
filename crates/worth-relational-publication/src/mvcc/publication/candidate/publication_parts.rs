//! Transfer original candidate custody into the one publication receiver.
use super::*;

impl PreparedRelationalCommitCandidate {
    pub(in crate::internal_port::mvcc::publication) fn into_publication_parts(
        mut self,
    ) -> PreparedRelationalPublicationParts {
        self.publication_binding
            .release_candidate_registration(self.candidate_id);
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .expect(/* Proof: only this consuming receiver, Drop of an unconsumed candidate and the abandoned-registration reaper empty the payload. This candidate is alive and unconsumed, and it holds the payload's only strong reference, so the reaper cannot see it as abandoned. */ "a live candidate retains its payload until consumed");
        self.consumed = true;
        let (mut movement, completion) = payload.execution.split(payload.published_snapshot_slot);
        movement.unique_fields = payload.unique_fields;
        PreparedRelationalPublicationParts {
            companion_read: payload.companion_read,
            managed_source_reads: payload.managed_source_reads,
            current_read: payload.current_read,
            runtime_instance_id: self.runtime_instance_id,
            candidate_id: self.candidate_id,
            publication_binding: self.publication_binding.clone(),
            expected: payload.expected_basis,
            expected_root: payload.expected_root,
            publication_cell: payload.publication_cell,
            movement,
            completion,
            candidate_retention: payload.retention,
            control: payload.control,
        }
    }
}

#[doc(hidden)]
pub trait PreparedRelationalCommitCandidateOwnerPort: Sized {
    fn into_publication_parts(self) -> PreparedRelationalPublicationParts;
}

impl PreparedRelationalCommitCandidateOwnerPort for PreparedRelationalCommitCandidate {
    fn into_publication_parts(self) -> PreparedRelationalPublicationParts {
        self.into_publication_parts()
    }
}

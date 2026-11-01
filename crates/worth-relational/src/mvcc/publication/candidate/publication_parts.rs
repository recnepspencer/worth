//! Transfer original candidate custody into the one publication receiver.
use super::*;

impl PreparedRelationalCommitCandidate {
    pub(in crate::mvcc::publication) fn into_publication_parts(
        mut self,
    ) -> Result<PreparedRelationalPublicationParts, super::super::RelationalPublicationDeferred>
    {
        let payload = self
            .publication_binding
            .take_candidate(self.candidate_id)
            .ok_or(
                super::super::RelationalPublicationDeferred::CandidateLifetimeExpired {
                    maximum_lifetime_millis: self.maximum_lifetime_millis,
                },
            )?;
        self.consumed = true;
        let (movement, completion) = payload.execution.split(payload.published_snapshot_slot);
        Ok(PreparedRelationalPublicationParts {
            companion_read: payload.companion_read,
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
            expires_at: self.expires_at,
            maximum_lifetime_millis: self.maximum_lifetime_millis,
        })
    }
}

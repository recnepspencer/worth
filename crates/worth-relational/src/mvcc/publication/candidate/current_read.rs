//! Bind a prepared publication to the original owner's live readonly source.
use super::PreparedRelationalCommitCandidate;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::mvcc::publication::RelationalPublicationReadDependencyDenial as Denial;

impl PreparedRelationalCommitCandidate {
    /// Require this exact owner-admitted source to remain current until the
    /// publication cutover. The source is a condition, never write authority.
    /// This does not change the candidate's branch, payload or expected head.
    /// The initial profile requires a selected immutable source root. An
    /// uncommitted Empty reference has no cell-owned schema/visibility root
    /// and refuses here; a committed root may contain zero business records.
    pub fn require_current_read(
        &mut self,
        source: &AdmittedRelationalBranchBasis,
    ) -> Result<(), Denial> {
        if source.identity().runtime_instance_id() != self.runtime_instance_id {
            return Err(Denial::ForeignOwner);
        }
        if matches!(
            source.reference().target(),
            worth_foundational::FoundationalBranchTarget::Empty
        ) {
            return Err(Denial::EmptySourceUnsupported);
        }
        let mut payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let payload = payload.as_mut().ok_or(Denial::CandidateUnavailable)?;
        if payload.current_read.is_some() {
            return Err(Denial::AlreadyRequired);
        }
        if source.identity() == payload.publication_cell.identity()
            && source.descriptor() != &payload.expected_basis
        {
            return Err(Denial::ConflictingSource);
        }
        // The actual admitted lease retains its original cell and full axes;
        // neither a descriptor nor an identifier can stand in for this input.
        payload.current_read = Some(source.clone());
        Ok(())
    }
}

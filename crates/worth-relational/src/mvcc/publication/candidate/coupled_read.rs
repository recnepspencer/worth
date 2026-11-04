//! The complete prepared image keeps its genuine proposal and registration phase.
use super::PreparedRelationalCommitCandidate;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    mvcc::RelationalMutationProposalIdentity,
    runtime::{
        RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalPreparationRuntime,
    },
    transactions::data::TransactionCommitError,
};

pub(super) struct CoupledCandidateReadContext {
    runtime: RelationalPreparationRuntime,
    before: AdmittedRelationalBranchBasis,
    proposal: RelationalMutationProposalIdentity,
}
impl PreparedRelationalCommitCandidate {
    pub(crate) fn retain_coupled_read(
        &mut self,
        runtime: &RelationalPreparationRuntime,
        before: AdmittedRelationalBranchBasis,
        proposal: RelationalMutationProposalIdentity,
    ) {
        let mut payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let payload = payload.as_mut().expect("newly registered candidate");
        assert!(std::sync::Arc::ptr_eq(
            &before.inner.root,
            &payload.expected_root
        ));
        assert_eq!(proposal.runtime_instance_id(), self.runtime_instance_id);
        assert!(payload
            .coupled_read
            .replace(CoupledCandidateReadContext {
                runtime: runtime.clone(),
                before,
                proposal,
            })
            .is_none());
    }
    /// Reads this complete unpublished image, including Native consequences.
    /// No snapshot or Current observation can be obtained from this pin.
    pub fn pin_coupled_candidate(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>>,
        TransactionCommitError,
    > {
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let payload = payload
            .as_ref()
            .ok_or_else(|| crate::mvcc::origin_denial("prepared candidate was consumed"))?;
        let Some(read) = &payload.coupled_read else {
            return Ok(None);
        };
        RelationalPinnedRead::capture_prepared(
            execution,
            &read.runtime,
            &read.before,
            &read.proposal,
            self.candidate_id,
            std::sync::Arc::clone(payload.execution.prepared_root()),
            payload.execution.changed_records(),
        )
        .map(Some)
    }
}

//! The complete prepared image keeps its genuine proposal and registration phase.
use super::PreparedRelationalCommitCandidate;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::visibility::materialization::read_records::reader::pinned_read::RelationalAppliedCandidateReadIdentity;
use crate::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedRead;

pub(super) struct CoupledCandidateReadContext {
    dictionary: RuntimeSymbolTable,
    instrumentation: RuntimeInstrumentation,
    before: AdmittedRelationalBranchBasis,
    proposal: RelationalMutationProposalIdentity,
}
impl PreparedRelationalCommitCandidate {
    pub(crate) fn retain_coupled_read(
        &mut self,
        dictionary: RuntimeSymbolTable,
        instrumentation: RuntimeInstrumentation,
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
                dictionary,
                instrumentation,
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
        let payload = payload.as_ref().ok_or_else(|| {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "prepared candidate was consumed",
            )
        })?;
        let Some(read) = &payload.coupled_read else {
            return Ok(None);
        };
        RelationalPinnedRead::capture_prepared(
            execution,
            &read.dictionary,
            &read.instrumentation,
            self.runtime_instance_id,
            &read.before,
            &read.proposal,
            self.candidate_id,
            std::sync::Arc::clone(payload.execution.prepared_root()),
            payload.execution.changed_records(),
        )
        .map(Some)
    }
}

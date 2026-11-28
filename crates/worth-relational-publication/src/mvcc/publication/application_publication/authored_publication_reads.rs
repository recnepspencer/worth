//! Exact installed ancestry and projection reads used by authored publication custody.
use super::*;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub struct AuthoredPublicationReads<'a> {
    pub(crate) history: RetirementHistoryInputs<'a>,
}

impl<'a> AuthoredPublicationReads<'a> {
    pub const fn new(history: RetirementHistoryInputs<'a>) -> Self {
        Self { history }
    }

    pub(crate) const fn runtime_instance_id(&self) -> u64 {
        self.history.projection.runtime_identity
    }

    pub(crate) fn readmit_canonical_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        origin: &Arc<CanonicalRequiredOutputOrigin>,
    ) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
        crate::publication::ownership::native_transition_succession::readmit_canonical_publication(
            self.history,
            execution,
            installed,
            basis,
            origin,
        )
    }
}

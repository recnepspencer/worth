//! Exact installed ancestry and projection reads used by authored publication custody.
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::canonical_commit_envelope::required_output_origin::CanonicalRequiredOutputOrigin;
use crate::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(crate) struct AuthoredPublicationReads<'a> {
    pub(super) history: RetirementHistoryInputs<'a>,
}

impl<'a> AuthoredPublicationReads<'a> {
    pub(crate) const fn new(history: RetirementHistoryInputs<'a>) -> Self {
        Self { history }
    }

    pub(super) const fn runtime_instance_id(&self) -> u64 {
        self.history.projection.runtime_identity
    }

    pub(super) fn readmit_canonical_publication(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        origin: &Arc<CanonicalRequiredOutputOrigin>,
    ) -> Result<RelationalRequiredOutputPublication, TransactionCommitError> {
        super::native_transition_succession::readmit_canonical_publication(
            self.history,
            execution,
            installed,
            basis,
            origin,
        )
    }
}

use crate::diagnostics::data::DiagnosticCode;
use crate::errors::data::{ErrorContext, ErrorOperation, RelationalSubsystem, SuggestedFix};
use crate::history::data::BranchId;
use crate::identity::data::VersionId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectedBranchRootDenialReason {
    ReferenceMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitPreparationReason {
    SelectedBranchRoot(SelectedBranchRootDenialReason),
    ProposalIdentityOrdinalExhausted,
    SymbolIdentityExhausted,
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitPreparationError {
    branch_id: BranchId,
    reason: CommitPreparationReason,
    expected_commit_id: Option<u64>,
    expected_version_id: VersionId,
    context: ErrorContext,
}

impl CommitPreparationError {
    pub(crate) fn selected_branch_root_reference_mismatch(
        branch_id: BranchId,
        expected_commit_id: Option<u64>,
        expected_version_id: VersionId,
    ) -> Self {
        Self::selected_branch_root(
            branch_id,
            SelectedBranchRootDenialReason::ReferenceMismatch,
            expected_commit_id,
            expected_version_id,
            SuggestedFix::VerifyBranchInputs,
        )
    }

    fn selected_branch_root(
        branch_id: BranchId,
        reason: SelectedBranchRootDenialReason,
        expected_commit_id: Option<u64>,
        expected_version_id: VersionId,
        suggested_fix: SuggestedFix,
    ) -> Self {
        Self {
            branch_id,
            reason: CommitPreparationReason::SelectedBranchRoot(reason),
            expected_commit_id,
            expected_version_id,
            context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Validate)
                .with_version(expected_version_id)
                .with_fix(suggested_fix),
        }
    }

    pub(crate) fn proposal_identity_exhausted(
        branch_id: BranchId,
        expected_version_id: VersionId,
    ) -> Self {
        Self {
            branch_id,
            reason: CommitPreparationReason::ProposalIdentityOrdinalExhausted,
            expected_commit_id: None,
            expected_version_id,
            context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Validate)
                .with_version(expected_version_id)
                .with_fix(SuggestedFix::InspectDiagnostics),
        }
    }

    pub(crate) fn storage_unavailable(
        branch_id: BranchId,
        expected_commit_id: Option<u64>,
        expected_version_id: VersionId,
        failure: crate::storage::data::RelationalStorageUnavailable,
    ) -> Self {
        Self {
            branch_id,
            expected_commit_id,
            expected_version_id,
            reason: CommitPreparationReason::StorageUnavailable(failure),
            context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Validate)
                .with_version(expected_version_id)
                .with_fix(SuggestedFix::RepairDurableStore),
        }
    }

    pub(crate) fn symbol_identity_exhausted(
        branch_id: BranchId,
        expected_version_id: VersionId,
    ) -> Self {
        Self {
            branch_id,
            reason: CommitPreparationReason::SymbolIdentityExhausted,
            expected_commit_id: None,
            expected_version_id,
            context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Validate)
                .with_version(expected_version_id)
                .with_fix(SuggestedFix::InspectDiagnostics),
        }
    }

    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub const fn reason(&self) -> CommitPreparationReason {
        self.reason
    }

    pub const fn expected_commit_id(&self) -> Option<u64> {
        self.expected_commit_id
    }

    pub const fn expected_version_id(&self) -> VersionId {
        self.expected_version_id
    }

    pub const fn code(&self) -> DiagnosticCode {
        DiagnosticCode::PreparationFailure
    }

    pub fn detail(&self) -> String {
        match self.reason {
            CommitPreparationReason::SymbolIdentityExhausted => format!(
                "symbol identity exhausted for branch {} at version {}",
                self.branch_id.0, self.expected_version_id.0
            ),
            CommitPreparationReason::StorageUnavailable(failure) => {
                format!("cold storage unavailable: {failure:?}")
            }
            CommitPreparationReason::ProposalIdentityOrdinalExhausted => format!(
                "proposal identity ordinal exhausted for branch {} at version {}",
                self.branch_id.0, self.expected_version_id.0
            ),
            CommitPreparationReason::SelectedBranchRoot(reason) => {
                let root = match reason {
                    SelectedBranchRootDenialReason::ReferenceMismatch => "does not match",
                };
                format!(
                    "selected branch root {root} binding for branch {} (commit {:?}, version {})",
                    self.branch_id.0, self.expected_commit_id, self.expected_version_id.0
                )
            }
        }
    }

    pub fn context(&self) -> &ErrorContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::{CommitPreparationError, CommitPreparationReason};
    use crate::history::data::BranchId;
    use crate::identity::data::VersionId;

    #[test]
    fn proposal_ordinal_exhaustion_has_a_dedicated_typed_reason() {
        let error = CommitPreparationError::proposal_identity_exhausted(
            BranchId("typed-exhaustion".to_owned()),
            VersionId(9),
        );

        assert_eq!(
            error.reason(),
            CommitPreparationReason::ProposalIdentityOrdinalExhausted
        );
        assert_eq!(error.expected_commit_id(), None);
        assert!(error
            .detail()
            .contains("proposal identity ordinal exhausted"));
    }
}

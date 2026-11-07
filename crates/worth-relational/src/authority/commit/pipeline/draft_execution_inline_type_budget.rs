//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(census: &mut crate::inline_type_sizes::Census) {
    {
        let mut owner = census.owner::<PreparedCommitState>(
            "crates/worth-relational/src/authority/commit/pipeline/draft_execution.rs",
            "PreparedCommitState",
            128 * 1024,
        );
        owner.field::<AdmittedCommitState>("admitted");
        owner.field::<crate::transactions::data::CommitStructuralSummary>(
            "public_structural_summary",
        );
        owner.field::<crate::mvcc::RelationalMutationProposalIdentity>("proposal_identity");
    }
    {
        let mut owner = census.owner::<PreparedCommitExecution>(
            "crates/worth-relational/src/authority/commit/pipeline/draft_execution.rs",
            "PreparedCommitExecution",
            128 * 1024,
        );
        owner.field::<PreparedCommitState>("state");
        owner.field::<AppliedMutationDraft>("mutation");
    }
}

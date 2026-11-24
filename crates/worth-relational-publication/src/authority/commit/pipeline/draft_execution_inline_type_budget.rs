//! Unit-only layout evidence; constructs no runtime, proof or authority.
use super::*;

pub(crate) fn collect(
    census: &mut worth_relational_contracts::internal_port::inline_type_sizes::census::Census,
) {
    {
        let mut owner = census.owner::<PreparedCommitState>(
            "crates/worth-relational/src/authority/commit/pipeline/draft_execution.rs",
            "PreparedCommitState",
            128 * 1024,
        );
        owner.field::<AdmittedCommitState>("admitted");
        owner.field::<worth_relational_contracts::internal_port::transactions::data::outcomes::commit_summaries::CommitStructuralSummary>(
            "public_structural_summary",
        );
        owner.field::<worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>("proposal_identity");
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

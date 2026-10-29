use worth_foundational::{
    foundational_commit_receipt_issuance, BoundaryHandle, FoundationalBranchCloseoutCause,
    FoundationalCommitId, FoundationalCommitReceiptIdentity, FoundationalTransitionIssuanceCause,
};
use worth_proof::AuthorityWitness;

use super::branch::authority_first_candidate;
use super::committed::{accepted_verdict, committed_authority, ordinary_commit_input};

pub fn receipt_authority() -> AuthorityWitness<worth_foundational::FoundationalCommitReceiptIssuance>
{
    foundational_commit_receipt_issuance()
}

pub const fn commit_id(value: u64) -> FoundationalCommitId {
    FoundationalCommitId::new(BoundaryHandle::new(value))
}

pub const fn receipt_identity(value: u64) -> FoundationalCommitReceiptIdentity {
    FoundationalCommitReceiptIdentity::new(BoundaryHandle::new(value))
}

pub fn committed_authority_artifact(
    payload: &'static str,
) -> worth_foundational::FoundationalCommittedAuthorityArtifact<&'static str> {
    accepted_verdict(payload)
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority artifact")
}

pub fn discard_receipt(
    payload: &'static str,
) -> worth_foundational::FoundationalBranchDiscardReceipt {
    authority_first_candidate(payload)
        .discard_with_zero_residue_proof()
        .expect("discard receipt")
}

pub const fn no_op_issuance_cause() -> FoundationalTransitionIssuanceCause {
    FoundationalTransitionIssuanceCause::NoOpAttested
}

pub const fn discard_closeout_cause() -> FoundationalBranchCloseoutCause {
    FoundationalBranchCloseoutCause::ExplicitDiscard
}

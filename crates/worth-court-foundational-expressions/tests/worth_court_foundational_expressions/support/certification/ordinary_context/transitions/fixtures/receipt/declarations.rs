pub(crate) use worth_proof::AuthorityWitness;
pub(crate) use {
    worth_foundational::foundational_commit_receipt_issuance, worth_foundational::BoundaryHandle,
    worth_foundational::FoundationalBranchCloseoutCause, worth_foundational::FoundationalCommitId,
    worth_foundational::FoundationalCommitReceiptIdentity,
    worth_foundational::FoundationalTransitionIssuanceCause,
};

pub(crate) use crate::certification::ordinary_context::transitions::fixtures::branch::authority_first_candidate;
pub(crate) use {
    crate::certification::ordinary_context::transitions::fixtures::committed::accepted_verdict,
    crate::certification::ordinary_context::transitions::fixtures::committed::committed_authority,
    crate::certification::ordinary_context::transitions::fixtures::committed::ordinary_commit_input,
};

pub(crate) fn receipt_authority(
) -> AuthorityWitness<worth_foundational::FoundationalCommitReceiptIssuance> {
    foundational_commit_receipt_issuance()
}

pub(crate) const fn commit_id(value: u64) -> FoundationalCommitId {
    worth_foundational::FoundationalCommitId::new(worth_foundational::BoundaryHandle::new(value))
}

pub(crate) const fn receipt_identity(value: u64) -> FoundationalCommitReceiptIdentity {
    worth_foundational::FoundationalCommitReceiptIdentity::new(
        worth_foundational::BoundaryHandle::new(value),
    )
}

pub(crate) fn committed_authority_artifact(
    payload: &'static str,
) -> worth_foundational::FoundationalCommittedAuthorityArtifact<&'static str> {
    accepted_verdict(payload)
        .commit_with(ordinary_commit_input(), committed_authority())
        .expect("committed authority artifact")
}

pub(crate) fn discard_receipt(
    payload: &'static str,
) -> worth_foundational::FoundationalBranchDiscardReceipt {
    authority_first_candidate(payload)
        .discard_with_zero_residue_proof()
        .expect("discard receipt")
}

pub(crate) const fn no_op_issuance_cause() -> FoundationalTransitionIssuanceCause {
    worth_foundational::FoundationalTransitionIssuanceCause::NoOpAttested
}

pub(crate) const fn discard_closeout_cause() -> FoundationalBranchCloseoutCause {
    worth_foundational::FoundationalBranchCloseoutCause::ExplicitDiscard
}

pub(crate) use worth_relational::facade::history::BranchId;
pub(crate) use worth_relational::facade::mvcc::RelationalOperationControl;
pub(crate) use {
    crate::relational_certification::ordinary_context::owner_service_completion::world::commit_direct,
    crate::relational_certification::ordinary_context::owner_service_completion::world::commit_through_services,
    crate::relational_certification::ordinary_context::owner_service_completion::world::empty_runtime,
    crate::relational_certification::ordinary_context::owner_service_completion::world::fork_direct,
    crate::relational_certification::ordinary_context::owner_service_completion::world::fork_through_services,
};
pub(crate) use {
    worth_relational::facade::branch::RelationalBranchBasisDenial,
    worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    worth_relational::facade::branch::RelationalBranchDeletionOutcome,
    worth_relational::facade::branch::RelationalBranchLifecyclePosture,
    worth_relational::facade::branch::RelationalBranchRetentionTerminalOutcome,
};

pub(crate) fn assert_basis_semantics_equal(
    direct: &RelationalBranchBasisDescriptor,
    ported: &RelationalBranchBasisDescriptor,
) {
    assert_eq!(direct.branch_id(), ported.branch_id());
    assert_eq!(
        direct.reference().generation(),
        ported.reference().generation()
    );
    assert_eq!(direct.truth_version(), ported.truth_version());
    assert_eq!(direct.schema_commitment(), ported.schema_commitment());
    assert_eq!(
        direct.visibility_commitment(),
        ported.visibility_commitment()
    );
    assert_eq!(direct.posture(), ported.posture());
}

//! Late registrations need a genuine generation before ordinary publication.
use super::*;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;

pub(super) fn populate(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    index_ids: &[DerivedIndexId],
) {
    let identity = runtime.branch_identity(branch).unwrap();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let built = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: branch.clone(),
            index_ids: index_ids.to_vec(),
        },
        &basis,
    );
    assert!(built.basis_denial.is_none(), "{built:?}");
    assert!(built.execution_denial.is_none(), "{built:?}");
    assert!(built.failed_indexes.is_empty(), "{built:?}");
    assert_eq!(built.generations.len(), index_ids.len(), "{built:?}");
}

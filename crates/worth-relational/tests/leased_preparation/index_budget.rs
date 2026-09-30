use super::*;
use worth_relational::facade::runtime::RelationalRuntime;

pub(super) fn assert_prep_and_map_share_work(
    runtime: &RelationalRuntime,
    request: &DerivedIndexBuildRequest,
    authority: &ExecutionAuthority,
) {
    let missing_lease = authority
        .request_lease(lease_request_with_work(
            32 * 1024 * 1024,
            CancellationToken::new(),
            1,
        ))
        .expect("one-step missing-index lease is admitted");
    let mut missing_request = request.clone();
    missing_request.index_ids = vec![DerivedIndexId(u64::MAX)];
    let missing = runtime
        .index_authority()
        .build_for_commit_with_lease(missing_request, &missing_lease);
    assert!(missing.execution_denial.is_none());
    assert_eq!(missing.failed_indexes, vec![DerivedIndexId(u64::MAX)]);

    let one_step_lease = authority
        .request_lease(lease_request_with_work(
            32 * 1024 * 1024,
            CancellationToken::new(),
            1,
        ))
        .expect("one-step work lease is admitted");
    let stopped = runtime
        .index_authority()
        .build_for_commit_with_lease(request.clone(), &one_step_lease);
    assert_eq!(
        stopped.execution_denial.map(|denial| denial.kind),
        Some(DerivedIndexExecutionDenialKind::WorkExhausted),
    );
    assert!(stopped.generations.is_empty());
}

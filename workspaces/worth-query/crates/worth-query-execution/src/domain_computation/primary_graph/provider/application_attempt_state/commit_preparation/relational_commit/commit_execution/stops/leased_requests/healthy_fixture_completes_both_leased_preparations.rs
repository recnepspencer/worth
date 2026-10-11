//! healthy fixture completes both leased preparations.
use super::*;

#[test]
fn healthy_fixture_completes_both_leased_preparations() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    in_isolated_process(
        concat!(
            module_path!(),
            "::healthy_fixture_completes_both_leased_preparations"
        ),
        || {
            let runtime = RelationalRuntimeApi::builder()
                .schema_registry(schema())
                .build();
            let created = transaction(&runtime, "source")
                .commit(execution, &runtime)
                .unwrap();
            let index = related_index(&runtime);
            let lease = authority()
                .request_lease(request(32 * 1024 * 1024, 1_000_000))
                .unwrap();
            let outcome = runtime.index_authority().build_for_commit(
                &worth_execution::ExecutionRequest::leased(&lease),
                DerivedIndexBuildRequest {
                    source_commit_id: created.commit.commit_id,
                    branch_id: BranchId("main".to_owned()),
                    index_ids: vec![index.index_id],
                },
            );
            assert!(outcome.execution_denial.is_none());
            assert!(outcome.basis_denial.is_none());
            assert!(outcome.failed_indexes.is_empty());
            assert_eq!(outcome.generations.len(), 1);
            let before = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap()
                .0;
            let _prepared = runtime
                .prepare_branch_transaction(
                    &worth_execution::ExecutionRequest::leased(&lease),
                    transaction(&runtime, "target"),
                )
                .expect("the same fixture admits ordinary leased preparation");
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .0,
                before
            );
        },
    );
}

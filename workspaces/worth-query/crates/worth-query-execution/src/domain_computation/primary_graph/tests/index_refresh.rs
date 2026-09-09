use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_relational::facade::{
    history::{BranchId, RelationalCommitReceipt},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntime,
};

use super::fixture::installed_world;

#[test]
fn mutation_refreshes_every_exact_performed_commit_across_branches() {
    let world = installed_world(&[("index-refresh", WorthQueryPrincipalMappingStatus::Enabled)]);
    let graph = world
        .application
        .runtime
        .primary_graph()
        .expect("fixture publishes its primary graph");
    let handle = graph.integration_handle();
    let index_ids = handle.primary_index_ids.to_vec();

    let (feature_commit, main_commit) = handle
        .execute_mutation_with_index_refresh(|runtime| {
            let (_, source) = runtime
                .observe_fork_source(&BranchId("main".to_owned()))
                .expect("main exposes an exact fork source");
            runtime
                .fork_branch(BranchId("index-refresh-feature".to_owned()), source)
                .expect("feature branch forks from main");
            let feature = commit_empty(runtime, "index-refresh-feature");
            let main = commit_empty(runtime, "main");
            assert!(index_ids.iter().any(|index_id| {
                runtime
                    .index_access()
                    .published_generation_for_commit(*index_id, &feature)
                    .is_none()
            }));
            Ok::<_, &'static str>((feature, main))
        })
        .expect("exact publication delta refresh succeeds")
        .expect("mutation succeeds");

    handle.with_runtime(|runtime| {
        for committed in [&feature_commit, &main_commit] {
            for index_id in &index_ids {
                assert!(
                    runtime
                        .index_access()
                        .published_generation_for_commit(*index_id, committed)
                        .is_some(),
                    "index {index_id:?} must be current for exact commit {:?} on branch {:?}",
                    committed.commit_id,
                    committed.branch_id,
                );
            }
        }
    });
}

fn commit_empty(runtime: &mut RelationalRuntime, branch: &str) -> RelationalCommitReceipt {
    let branch_id = BranchId(branch.to_owned());
    let identity = runtime
        .branch_identity(&branch_id)
        .expect("commit branch identity exists");
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("commit branch basis is current");
    let committed = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("transaction binds to exact branch")
        .commit(runtime)
        .expect("empty fixture transaction commits");
    let receipt = committed.commit.clone();
    super::fixture::release_test_commit_snapshot(runtime, &committed);
    receipt
}

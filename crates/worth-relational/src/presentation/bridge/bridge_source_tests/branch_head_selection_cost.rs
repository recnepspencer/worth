use std::sync::{Arc, Mutex};

use worth_runtime_bridge::facade::{
    CommittedPatchSource, RelationalCommittedPatchRequest, TruthBranchHeadSource,
    TruthBranchIdentity, TruthCommitIdentity,
};

use crate::presentation::bridge::relational_test_support::{
    create_entity, create_entity_outcome, release_test_commit_snapshot,
};

use super::super::RuntimeBridgeRelationalSource;
use super::support::{linear_ancestry_work, runtime_with_test_schema};

#[test]
fn exact_selection_is_constant_and_historical_selection_uses_exact_ancestry() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let ancestor = create_entity_outcome(&runtime.lock().unwrap(), "deep-head-ancestor");
    let ancestor_commit_id = ancestor.commit.commit_id;
    release_test_commit_snapshot(&runtime.lock().unwrap(), &ancestor);
    for ordinal in 1..=255 {
        create_entity(
            &runtime.lock().unwrap(),
            &format!("deep-head-history-{ordinal}"),
        );
    }
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, basis) = source.observe_branch_basis(&identity).unwrap();
    let head_commit = basis
        .observation()
        .commit_id()
        .expect("deep branch head commit");
    let ancestry_work = linear_ancestry_work(&runtime.lock().unwrap(), head_commit);
    let _head_lease = source.bind_branch_head_basis_for_bridge(&basis).unwrap();
    let branch = TruthBranchIdentity::from_relational_branch_id("main");

    source
        .load_branch_head_patch(&branch)
        .expect("exact branch head publication");
    assert_eq!(source.selection_work_totals(), (1, 0));

    source
        .load_committed_patch(RelationalCommittedPatchRequest::on_branch(
            TruthCommitIdentity::from_relational_commit_id(ancestor_commit_id.0),
            branch.clone(),
        ))
        .expect("visible historical ancestor");
    assert_eq!(source.selection_work_totals(), (2, ancestry_work));

    let future = create_entity_outcome(&runtime.lock().unwrap(), "future-after-bound-head");
    let future_commit_id = future.commit.commit_id;
    release_test_commit_snapshot(&runtime.lock().unwrap(), &future);
    let denial = source
        .load_committed_patch(RelationalCommittedPatchRequest::on_branch(
            TruthCommitIdentity::from_relational_commit_id(future_commit_id.0),
            branch,
        ))
        .expect_err("a commit beyond the bound head is unreachable");
    assert!(denial.to_string().contains("cannot see requested commit"));
    assert!(head_commit < future_commit_id);
    assert_eq!(source.selection_work_totals(), (3, 2 * ancestry_work));
}

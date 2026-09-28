use std::sync::{Arc, Mutex};

use worth_runtime_bridge::facade::{
    BridgeTruthViewEvaluationRequest, CommittedPatchSource, RelationalCommittedPatchRequest,
    TruthBranchIdentity, TruthCommitIdentity,
};

use crate::facade::history::BranchId;
use crate::presentation::bridge::relational_test_support::{create_entity_outcome, create_entity_outcome_on_branch, fork_branch};

use super::super::RuntimeBridgeRelationalSource;
use super::support::{runtime_bridge_for_envelope, runtime_with_test_schema};

#[test]
fn advanced_fork_selects_inherited_ancestor_but_not_post_fork_source_sibling() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let inherited = create_entity_outcome(&runtime.lock().unwrap(), "fork-ancestor");
    let feature = BranchId("feature".to_owned());
    fork_branch(&runtime.lock().unwrap(), feature.clone(), &BranchId("main".to_owned()));
    let source_sibling = create_entity_outcome(&runtime.lock().unwrap(), "source-sibling");
    let feature_head =
        create_entity_outcome_on_branch(&runtime.lock().unwrap(), "feature-head", feature.clone());
    assert!(source_sibling.commit.commit_id < feature_head.commit.commit_id);

    let feature_identity = runtime
        .lock()
        .unwrap()
        .branch_identity(&feature)
        .expect("feature branch identity");
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let (_, basis) = source.observe_branch_basis(&feature_identity).unwrap();
    let _head_lease = source.bind_branch_head_basis_for_bridge(&basis).unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let snapshot = lease.snapshot_identity().clone();
    let branch = TruthBranchIdentity::from_relational_branch_id("feature");
    let ancestor_commit =
        TruthCommitIdentity::from_relational_commit_id(inherited.commit.commit_id.0);

    let inherited_selection = source
        .select_commit_at_snapshot(inherited.commit.commit_id, &snapshot)
        .unwrap();
    assert_eq!(inherited_selection.work().selections(), 1);
    assert!(inherited_selection.work().ancestry_visits() > 0);
    let inherited_envelope = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            ancestor_commit.clone(),
            snapshot.clone(),
        ))
        .expect("an advanced fork must retain its source-branch ancestor");
    assert_eq!(
        inherited_envelope.branch_identity().relational_branch_id(),
        Some("feature")
    );
    let source_basis = inherited_envelope
        .producer_metadata()
        .authoritative_source()
        .expect("selected fork provenance")
        .source_basis();
    assert!(source_basis.contains("selected-branch=feature"));
    assert!(source_basis.contains("authoring-branch=main"));
    runtime_bridge_for_envelope(source.clone(), &inherited_envelope)
        .evaluate(BridgeTruthViewEvaluationRequest::for_historical_commit(
            branch,
            ancestor_commit,
        ))
        .expect("historical evaluation must retain the advanced fork's ancestor");

    let sibling_selection = source
        .select_commit_at_snapshot(source_sibling.commit.commit_id, &snapshot)
        .unwrap();
    assert_eq!(sibling_selection.work().selections(), 1);
    assert!(sibling_selection.work().ancestry_visits() > 0);
    assert!(sibling_selection.into_result().is_err());
    let denial = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            TruthCommitIdentity::from_relational_commit_id(source_sibling.commit.commit_id.0),
            snapshot,
        ))
        .expect_err("a post-fork source sibling is not beneath the feature head");
    assert!(denial.to_string().contains("cannot see requested commit"));
}

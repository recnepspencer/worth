use std::sync::{Arc, Mutex};

use crate::facade::{
    BridgeTruthViewEvaluationRequest, CommittedPatchSource, RelationalCommittedPatchRequest,
    TruthBranchIdentity, TruthCommitIdentity,
};

use crate::relational_source::relational_test_support::{
    create_entity_outcome, create_entity_outcome_on_branch, fork_branch,
};
use worth_relational::facade::history::BranchId;

use super::super::RuntimeBridgeRelationalSource;
use super::support::{linear_ancestry_work, runtime_bridge_for_envelope, runtime_with_test_schema};

#[test]
fn advanced_fork_selects_inherited_ancestor_but_not_post_fork_source_sibling() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let inherited = create_entity_outcome(&runtime.lock().unwrap(), "fork-ancestor");
    let feature = BranchId("feature".to_owned());
    fork_branch(
        &runtime.lock().unwrap(),
        feature.clone(),
        &BranchId("main".to_owned()),
    );
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

    let ancestry_work =
        linear_ancestry_work(&runtime.lock().unwrap(), feature_head.commit.commit_id);

    let inherited_envelope = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            ancestor_commit.clone(),
            snapshot.clone(),
        ))
        .expect("an advanced fork must retain its source-branch ancestor");
    assert_eq!(source.selection_work_totals(), (1, ancestry_work));
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

    let after_evaluation = source.selection_work_totals();
    assert_eq!(after_evaluation, (2, 2 * ancestry_work));
    let denial = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            TruthCommitIdentity::from_relational_commit_id(source_sibling.commit.commit_id.0),
            snapshot,
        ))
        .expect_err("a post-fork source sibling is not beneath the feature head");
    assert!(denial.to_string().contains("cannot see requested commit"));
    // The sibling costs one more selection and exactly the same full walk as
    // the inherited ancestor.
    assert_eq!(
        source.selection_work_totals(),
        (after_evaluation.0 + 1, after_evaluation.1 + ancestry_work)
    );
}

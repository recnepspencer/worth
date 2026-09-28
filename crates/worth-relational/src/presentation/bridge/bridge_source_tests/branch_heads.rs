use std::sync::{Arc, Barrier, Mutex};

use worth_runtime_bridge::facade::{
    BridgeTruthViewEvaluationRequest, CommittedPatchSource, RelationalCommittedPatchRequest,
    TruthBranchHeadSource, TruthBranchIdentity, TruthCommitIdentity,
};

use crate::facade::history::BranchId;
use crate::presentation::bridge::relational_test_support::{
    create_entity_outcome, create_entity_outcome_on_branch, fork_branch,
};

use super::super::RuntimeBridgeRelationalSource;
use super::support::{runtime_bridge_for_envelope, runtime_with_test_schema};

#[test]
fn branch_head_loading_requires_an_explicit_owner_admitted_binding() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let branch = TruthBranchIdentity::from_relational_branch_id("main");
    let before = runtime.lock().unwrap().branch_basis_cost_counters();

    let error = source
        .load_branch_head_patch(&branch)
        .expect_err("a branch label alone cannot select an owner head");

    assert!(error
        .to_string()
        .contains("no explicitly admitted head basis"));
    assert_eq!(runtime.lock().unwrap().branch_basis_cost_counters(), before);
}

#[test]
fn sibling_heads_sharing_a_commit_keep_exact_branch_snapshot_bindings() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let (storm_identity, maintenance_identity) = {
        let runtime = runtime.lock().unwrap();
        create_entity_outcome(&runtime, "shared-sibling-root");
        fork_branch(
            &runtime,
            BranchId("storm".to_owned()),
            &BranchId("main".to_owned()),
        );
        fork_branch(
            &runtime,
            BranchId("maintenance".to_owned()),
            &BranchId("main".to_owned()),
        );
        (
            runtime
                .branch_identity(&BranchId("storm".to_owned()))
                .unwrap(),
            runtime
                .branch_identity(&BranchId("maintenance".to_owned()))
                .unwrap(),
        )
    };
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let (_, storm_basis) = source.observe_branch_basis(&storm_identity).unwrap();
    let (_, maintenance_basis) = source.observe_branch_basis(&maintenance_identity).unwrap();
    assert_eq!(
        storm_basis.observation().commit_id(),
        maintenance_basis.observation().commit_id(),
    );
    let before_bind = runtime.lock().unwrap().branch_basis_cost_counters();
    let storm_lease = source
        .bind_branch_head_basis_for_bridge(&storm_basis)
        .unwrap();
    let maintenance_lease = source
        .bind_branch_head_basis_for_bridge(&maintenance_basis)
        .unwrap();
    let storm_snapshot = storm_lease.snapshot_identity().clone();
    let maintenance_snapshot = maintenance_lease.snapshot_identity().clone();
    assert_ne!(storm_snapshot, maintenance_snapshot);

    let storm_branch = TruthBranchIdentity::from_relational_branch_id("storm");
    let maintenance_branch = TruthBranchIdentity::from_relational_branch_id("maintenance");
    let storm_envelope = source.load_branch_head_patch(&storm_branch).unwrap();
    let maintenance_envelope = source.load_branch_head_patch(&maintenance_branch).unwrap();
    assert_eq!(
        storm_envelope.branch_identity().relational_branch_id(),
        Some("storm")
    );
    assert_eq!(
        maintenance_envelope
            .branch_identity()
            .relational_branch_id(),
        Some("maintenance")
    );
    assert_eq!(storm_envelope.snapshot_identity(), &storm_snapshot);
    assert_eq!(
        maintenance_envelope.snapshot_identity(),
        &maintenance_snapshot
    );
    runtime_bridge_for_envelope(source.clone(), &storm_envelope)
        .evaluate(BridgeTruthViewEvaluationRequest::for_branch_head(
            storm_branch.clone(),
        ))
        .expect("an inherited fork head must plan through the real branch-head selector");
    let commit_id = storm_basis.observation().commit_id().unwrap();
    let collision = source
        .load_committed_patch(RelationalCommittedPatchRequest::new(
            TruthCommitIdentity::from_relational_commit_id(commit_id.0),
        ))
        .expect_err("a commit shared by two observations is ambiguous");
    assert!(collision
        .to_string()
        .contains("admitted head of multiple branches"));
    let commit_identity = TruthCommitIdentity::from_relational_commit_id(commit_id.0);
    let storm_selected = source
        .load_committed_patch(RelationalCommittedPatchRequest::on_branch(
            commit_identity.clone(),
            storm_branch.clone(),
        ))
        .expect("the selected storm branch disambiguates the shared commit");
    let maintenance_selected = source
        .load_committed_patch(RelationalCommittedPatchRequest::on_branch(
            commit_identity.clone(),
            maintenance_branch.clone(),
        ))
        .expect("the selected maintenance branch disambiguates the shared commit");
    assert_eq!(storm_selected.snapshot_identity(), &storm_snapshot);
    assert_eq!(
        maintenance_selected.snapshot_identity(),
        &maintenance_snapshot
    );
    let storm_source_basis = storm_selected
        .producer_metadata()
        .authoritative_source()
        .expect("storm source provenance")
        .source_basis();
    assert!(storm_source_basis.contains("selected-branch=storm"));
    assert!(storm_source_basis.contains("authoring-branch=main"));
    let maintenance_source_basis = maintenance_selected
        .producer_metadata()
        .authoritative_source()
        .expect("maintenance source provenance")
        .source_basis();
    assert!(maintenance_source_basis.contains("selected-branch=maintenance"));
    assert!(maintenance_source_basis.contains("authoring-branch=main"));
    runtime_bridge_for_envelope(source.clone(), &storm_selected)
        .evaluate(BridgeTruthViewEvaluationRequest::for_historical_commit(
            storm_branch.clone(),
            commit_identity.clone(),
        ))
        .expect("historical selector retains the storm branch axis");
    runtime_bridge_for_envelope(source.clone(), &maintenance_selected)
        .evaluate(BridgeTruthViewEvaluationRequest::for_historical_commit(
            maintenance_branch.clone(),
            commit_identity,
        ))
        .expect("historical selector retains the maintenance branch axis");

    let storm_receipt = storm_lease.release();
    assert!(storm_receipt.unbound());
    assert_eq!(storm_receipt.branch_identity(), &storm_branch);
    assert!(source.load_branch_head_patch(&storm_branch).is_err());
    assert!(source.load_branch_head_patch(&maintenance_branch).is_ok());
    let sole_remaining = source
        .load_committed_patch(RelationalCommittedPatchRequest::new(
            TruthCommitIdentity::from_relational_commit_id(commit_id.0),
        ))
        .unwrap();
    assert_eq!(sole_remaining.snapshot_identity(), &maintenance_snapshot);
    let after_explicit = runtime.lock().unwrap().branch_basis_cost_counters();
    assert_eq!(
        after_explicit.external_retention_acquires,
        before_bind.external_retention_acquires + 2
    );
    assert_eq!(
        after_explicit.external_retention_releases,
        before_bind.external_retention_releases + 1
    );
    assert_eq!(
        after_explicit.external_retention_drop_releases,
        before_bind.external_retention_drop_releases
    );

    drop(maintenance_lease);
    assert!(source.load_branch_head_patch(&maintenance_branch).is_err());
    let after_drop = runtime.lock().unwrap().branch_basis_cost_counters();
    assert_eq!(
        after_drop.external_retention_releases,
        before_bind.external_retention_releases + 2
    );
    assert_eq!(
        after_drop.external_retention_drop_releases,
        before_bind.external_retention_drop_releases + 1
    );
}

#[test]
fn explicit_snapshot_request_rejects_an_earlier_sibling_commit() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let ancestor = create_entity_outcome(&runtime.lock().unwrap(), "shared-ancestor");
    let feature = BranchId("feature".to_owned());
    fork_branch(
        &runtime.lock().unwrap(),
        feature.clone(),
        &BranchId("main".to_owned()),
    );
    let sibling =
        create_entity_outcome_on_branch(&runtime.lock().unwrap(), "feature-only", feature);
    let main_head = create_entity_outcome(&runtime.lock().unwrap(), "main-head");
    assert!(sibling.commit.commit_id < main_head.commit.commit_id);

    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let main_identity = runtime.lock().unwrap().main_branch_identity();
    let (_, basis) = source.observe_branch_basis(&main_identity).unwrap();
    let lease = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let snapshot = lease.snapshot_identity().clone();

    source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            TruthCommitIdentity::from_relational_commit_id(ancestor.commit.commit_id.0),
            snapshot.clone(),
        ))
        .expect("the common ancestor is visible beneath the selected main root");
    let denial = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            TruthCommitIdentity::from_relational_commit_id(sibling.commit.commit_id.0),
            snapshot,
        ))
        .expect_err("an earlier sibling commit is not visible beneath the selected main root");
    assert!(denial.to_string().contains("cannot see requested commit"));
}

#[test]
fn explicit_snapshot_request_admits_visible_ancestors_and_rejects_future_commits() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let first = create_entity_outcome(&runtime.lock().unwrap(), "first-exact-basis");
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, first_basis) = source.observe_branch_basis(&identity).unwrap();
    let first_lease = source.retain_branch_basis_for_bridge(&first_basis).unwrap();
    let first_snapshot = first_lease.snapshot_identity().clone();

    let second = create_entity_outcome(&runtime.lock().unwrap(), "second-exact-basis");
    let (_, second_basis) = source.observe_branch_basis(&identity).unwrap();
    let second_lease = source
        .retain_branch_basis_for_bridge(&second_basis)
        .unwrap();
    let second_snapshot = second_lease.snapshot_identity().clone();
    let first_commit = TruthCommitIdentity::from_relational_commit_id(first.commit.commit_id.0);
    let second_commit = TruthCommitIdentity::from_relational_commit_id(second.commit.commit_id.0);

    let denial = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            second_commit.clone(),
            first_snapshot.clone(),
        ))
        .expect_err("an earlier observation cannot authorize a future commit envelope");
    assert!(denial.to_string().contains("cannot see requested commit"));

    let unavailable = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            TruthCommitIdentity::from_relational_commit_id(u64::MAX),
            first_snapshot.clone(),
        ))
        .expect_err("a selected observation cannot authorize an unavailable commit");
    assert!(unavailable
        .to_string()
        .contains("unavailable requested commit"));

    let visible_ancestor = source
        .load_committed_patch(RelationalCommittedPatchRequest::at_snapshot(
            first_commit.clone(),
            second_snapshot.clone(),
        ))
        .expect("a retained observation authorizes commits visible beneath its exact root");
    assert_eq!(visible_ancestor.commit_identity(), &first_commit);
    assert_eq!(visible_ancestor.snapshot_identity(), &second_snapshot);
    assert_ne!(visible_ancestor.commit_identity(), &second_commit);
}

#[test]
fn same_branch_rebinding_survives_delayed_old_release() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    create_entity_outcome(&runtime.lock().unwrap(), "first-head");
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, old_basis) = source.observe_branch_basis(&identity).unwrap();
    let old_lease = source
        .bind_branch_head_basis_for_bridge(&old_basis)
        .unwrap();
    let old_snapshot = old_lease.snapshot_identity().clone();

    create_entity_outcome(&runtime.lock().unwrap(), "second-head");
    let (_, new_basis) = source.observe_branch_basis(&identity).unwrap();
    let new_lease = source
        .bind_branch_head_basis_for_bridge(&new_basis)
        .unwrap();
    let new_snapshot = new_lease.snapshot_identity().clone();
    assert_ne!(old_snapshot, new_snapshot);

    let old_release = old_lease.release();
    assert!(!old_release.unbound());
    let current = source
        .load_branch_head_patch(&TruthBranchIdentity::from_relational_branch_id("main"))
        .unwrap();
    assert_eq!(current.snapshot_identity(), &new_snapshot);

    let final_release = new_lease.release();
    assert!(final_release.unbound());
    assert!(source
        .load_branch_head_patch(&TruthBranchIdentity::from_relational_branch_id("main"))
        .is_err());
}

#[test]
fn sole_branch_head_selects_commit_among_equivalent_retained_observations() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    let committed = create_entity_outcome(&runtime.lock().unwrap(), "retained-head");
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, basis) = source.observe_branch_basis(&identity).unwrap();
    let retained = source.retain_branch_basis_for_bridge(&basis).unwrap();
    let head = source.bind_branch_head_basis_for_bridge(&basis).unwrap();

    let envelope = source
        .load_committed_patch(RelationalCommittedPatchRequest::new(
            TruthCommitIdentity::from_relational_commit_id(committed.commit.commit_id.0),
        ))
        .expect("the sole explicit head disambiguates equivalent observations");

    assert_eq!(envelope.snapshot_identity(), head.snapshot_identity());
    assert_ne!(envelope.snapshot_identity(), retained.snapshot_identity());
}

#[test]
fn barrier_ordered_rebind_cannot_be_removed_by_the_old_lease() {
    let runtime = Arc::new(Mutex::new(runtime_with_test_schema()));
    create_entity_outcome(&runtime.lock().unwrap(), "race-first-head");
    let source =
        RuntimeBridgeRelationalSource::for_shared_graph_role(Arc::clone(&runtime), "model")
            .unwrap();
    let identity = runtime.lock().unwrap().main_branch_identity();
    let (_, old_basis) = source.observe_branch_basis(&identity).unwrap();
    let old_lease = source
        .bind_branch_head_basis_for_bridge(&old_basis)
        .unwrap();
    create_entity_outcome(&runtime.lock().unwrap(), "race-second-head");
    let (_, new_basis) = source.observe_branch_basis(&identity).unwrap();

    let start = Arc::new(Barrier::new(2));
    let rebound = Arc::new(Barrier::new(2));
    let bind_source = source.clone();
    let bind_start = Arc::clone(&start);
    let bind_rebound = Arc::clone(&rebound);
    let binder = std::thread::spawn(move || {
        bind_start.wait();
        let lease = bind_source
            .bind_branch_head_basis_for_bridge(&new_basis)
            .unwrap();
        bind_rebound.wait();
        lease
    });
    let release_start = Arc::clone(&start);
    let release_rebound = Arc::clone(&rebound);
    let releaser = std::thread::spawn(move || {
        release_start.wait();
        release_rebound.wait();
        old_lease.release()
    });

    let new_lease = binder.join().unwrap();
    let old_release = releaser.join().unwrap();
    assert!(!old_release.unbound());
    let current = source
        .load_branch_head_patch(&TruthBranchIdentity::from_relational_branch_id("main"))
        .unwrap();
    assert_eq!(current.snapshot_identity(), new_lease.snapshot_identity());
    assert!(new_lease.release().unbound());
}

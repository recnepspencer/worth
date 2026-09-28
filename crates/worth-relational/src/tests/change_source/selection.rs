use crate::facade::change_source::{RelationalCommitSelectionDenial, RelationalCommitSelectionWork};
use crate::facade::history::BranchId;
use crate::tests::support::{
    create_entity_outcome, create_entity_outcome_on_branch, runtime_with_test_schema,
};

use super::observe;

#[test]
fn reachable_selection_reports_exactly_the_ancestry_work_it_did() {
    let runtime = runtime_with_test_schema();
    let ancestor = create_entity_outcome(&runtime, "ancestor").commit.commit_id;
    let head = create_entity_outcome(&runtime, "head").commit.commit_id;
    let observation = observe(&runtime, "main").observation();
    let expected_visits = runtime
        .history()
        .classify_commit_in_ancestry(&runtime.history().inspect_commit_ancestry(head), ancestor)
        .traversal_work();

    runtime.performance_access().reset_counters();
    let selection = runtime.select_reachable_commit(&observation, ancestor);
    let work = selection.work();
    let selected = selection.into_outcome().expect("ancestor is reachable");

    assert_eq!(selected.commit_id(), ancestor);
    assert_eq!(selected.runtime_instance_id(), runtime.runtime_instance_id());
    assert_eq!(work.selections(), 1);
    assert_eq!(work.ancestry_visits(), expected_visits);
    assert!(work.ancestry_visits() > 0);
    let counters = runtime.performance_access().counters();
    assert_eq!(counters.observation_commit_selections, 1);
    assert_eq!(
        counters.observation_commit_ancestry_visits,
        expected_visits
    );
}

#[test]
fn exact_selection_costs_one_selection_and_no_ancestry() {
    let runtime = runtime_with_test_schema();
    let ancestor = create_entity_outcome(&runtime, "ancestor").commit.commit_id;
    let head = create_entity_outcome(&runtime, "head").commit.commit_id;
    let observation = observe(&runtime, "main").observation();

    let exact = runtime.select_exact_commit(&observation, head);
    assert_eq!(exact.work().selections(), 1);
    assert_eq!(exact.work().ancestry_visits(), 0);
    assert_eq!(exact.into_outcome().unwrap().commit_id(), head);

    let not_head = runtime.select_exact_commit(&observation, ancestor);
    assert_eq!(not_head.work().ancestry_visits(), 0);
    assert_eq!(
        not_head.into_outcome().unwrap_err(),
        RelationalCommitSelectionDenial::NotSelectedCommit {
            selected: head,
            requested: ancestor,
        }
    );
}

#[test]
fn fork_sees_inherited_ancestor_but_not_post_fork_sibling_at_equal_cost_class() {
    let runtime = runtime_with_test_schema();
    let inherited = create_entity_outcome(&runtime, "inherited").commit.commit_id;
    let feature = BranchId("feature".to_owned());
    runtime
        .history_authority()
        .fork_branch_from(feature.clone(), &BranchId("main".to_owned()))
        .unwrap();
    let sibling = create_entity_outcome(&runtime, "sibling").commit.commit_id;
    let feature_head = create_entity_outcome_on_branch(&runtime, "feature-head", feature)
        .commit
        .commit_id;
    let observation = observe(&runtime, "feature").observation();

    let reachable = runtime.select_reachable_commit(&observation, inherited);
    let unreachable = runtime.select_reachable_commit(&observation, sibling);

    assert!(reachable.work().ancestry_visits() > 0);
    assert!(unreachable.work().ancestry_visits() > 0);
    assert_eq!(reachable.into_outcome().unwrap().commit_id(), inherited);
    assert_eq!(
        unreachable.into_outcome().unwrap_err(),
        RelationalCommitSelectionDenial::Unreachable {
            selected: feature_head,
            requested: sibling,
        }
    );
}

#[test]
fn a_runtime_refuses_an_observation_another_runtime_issued() {
    let owner = runtime_with_test_schema();
    let other = runtime_with_test_schema();
    let commit_id = create_entity_outcome(&owner, "owned").commit.commit_id;
    let observation = observe(&owner, "main").observation();

    let selection = other.select_exact_commit(&observation, commit_id);

    assert_eq!(selection.work(), RelationalCommitSelectionWork::default());
    assert_eq!(
        selection.into_outcome().unwrap_err(),
        RelationalCommitSelectionDenial::ForeignObservation
    );
}

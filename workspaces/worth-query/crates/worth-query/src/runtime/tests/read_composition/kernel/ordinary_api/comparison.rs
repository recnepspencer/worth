use super::super::super::support::*;
use super::fixtures::local_identity_read;
use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField};
use crate::ordinary::comparison::{
    between, current_and_retained, declare, WorthQueryComparisonBasisFamily,
    WorthQueryComparisonChange, WorthQueryComparisonCorrespondencePosture,
    WorthQueryComparisonMaterialization, WorthQueryComparisonNextAction,
    WorthQueryComparisonRowChangeFamily, WorthQueryComparisonStopSource,
};
use crate::runtime::tests::support::{
    insert_command, stateful_bridge_task_runtime, test_session_label, test_string_aspect_value,
};

#[test]
fn current_and_retained_diff_is_unchanged_on_the_same_truth_basis() {
    let declaration = declare(local_identity_read)
        .expect("comparison declaration should build")
        .diff();
    let mut workspace = read_runtime()
        .workspace("ordinary-comparison-diff")
        .expect("workspace should open");
    let context = current_and_retained(&workspace).expect("the fixture requires an open workspace");
    let outcome = declaration.using(context).run(&mut workspace);
    let completion = outcome.completed().expect("comparison should complete");

    assert_eq!(completion.change(), WorthQueryComparisonChange::Unchanged);
    assert_eq!(completion.left().rows(), completion.right().rows());
    assert_eq!(
        completion.basis_pair().family(),
        WorthQueryComparisonBasisFamily::CurrentToHistorical
    );
    let WorthQueryComparisonMaterialization::RetainedHistorical(materialization) =
        completion.basis_pair().right().materialization()
    else {
        panic!("right basis should preserve retained historical materialization");
    };
    assert_eq!(
        materialization.resolved_path_class().as_str(),
        "resolved_retained_snapshot_path"
    );
    assert_eq!(
        completion
            .journey_counters()
            .historical_materialization_attempt_count(),
        1
    );
}

#[test]
fn branch_pair_diff_executes_both_real_bases_and_reports_added_rows() {
    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .diff();
    let mut left = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-left-branch")
        .expect("left workspace should open");
    let mut right = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-right-branch")
        .expect("right workspace should open");
    write_task(&mut left, "shared");
    write_task(&mut right, "shared");
    write_task(&mut right, "right-only");

    let context = between(
        &left,
        test_session_label("comparison-left-branch"),
        &right,
        test_session_label("comparison-right-branch"),
    )
    .expect("branch bases should admit");
    let outcome = declaration.using(context).run((&mut left, &mut right));
    let completion = outcome
        .completed()
        .expect("branch comparison should complete");

    assert_eq!(completion.change(), WorthQueryComparisonChange::Changed);
    assert_eq!(
        completion.basis_pair().family(),
        WorthQueryComparisonBasisFamily::BranchToBranch
    );
    assert_ne!(
        completion.basis_pair().left().branch_admission_identity(),
        completion.basis_pair().right().branch_admission_identity()
    );
    assert_eq!(
        completion.left().receipt().snapshot_identity(),
        completion.basis_pair().left().snapshot()
    );
    assert_eq!(
        completion.right().receipt().snapshot_identity(),
        completion.basis_pair().right().snapshot()
    );
    assert_eq!(completion.row_changes().len(), 1);
    assert!(completion.row_changes().iter().any(|change| {
        change.family() == WorthQueryComparisonRowChangeFamily::Added
            && change.left().is_none()
            && change.right().is_some()
    }));
    assert_eq!(
        completion.journey_counters().left_execution_attempt_count(),
        1
    );
    assert_eq!(
        completion
            .journey_counters()
            .right_execution_attempt_count(),
        1
    );
    assert_eq!(
        completion.journey_counters().left_row_scan_count(),
        completion.left().rows().len()
    );
    assert_eq!(
        completion.journey_counters().right_row_scan_count(),
        completion.right().rows().len()
    );
    assert_eq!(
        completion.journey_counters().emitted_row_change_count(),
        completion.row_changes().len()
    );
}

#[test]
fn structural_ambiguity_remains_advisory_and_exposes_no_partial_completion() {
    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .correspondence(4);
    let mut left = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-ambiguous-left")
        .expect("left workspace should open");
    let mut right = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-ambiguous-right")
        .expect("right workspace should open");
    write_task(&mut left, "subject");
    write_task(&mut right, "candidate-one");
    write_task(&mut right, "candidate-two");
    let context = between(
        &left,
        test_session_label("ambiguous-left-branch"),
        &right,
        test_session_label("ambiguous-right-branch"),
    )
    .expect("branch bases should admit");
    let outcome = declaration.using(context).run((&mut left, &mut right));

    assert!(outcome.completed().is_none());
    assert!(outcome.stop().is_none());
    let correspondence = outcome
        .correspondence()
        .expect("ambiguity must remain correspondence evidence");
    assert_eq!(
        correspondence.posture(),
        WorthQueryComparisonCorrespondencePosture::Advisory
    );
    assert!(!correspondence
        .subject()
        .evidence_identity()
        .as_str()
        .is_empty());
    assert_eq!(
        correspondence.basis_pair().family(),
        WorthQueryComparisonBasisFamily::BranchToBranch
    );
    let ambiguity = correspondence
        .correspondence()
        .outcome()
        .as_advisory_structural_ambiguous()
        .expect("two derived candidates should be ambiguous");
    assert_eq!(ambiguity.candidate_set().len(), 2);
}

#[test]
fn structural_correspondence_denies_multiple_left_subjects_without_partial_evidence() {
    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .correspondence(4);
    let mut workspace = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-multiple-subjects")
        .expect("workspace should open");
    write_task(&mut workspace, "first-subject");
    write_task(&mut workspace, "second-subject");
    let context = current_and_retained(&workspace).expect("the fixture requires an open workspace");
    let outcome = declaration.using(context).run(&mut workspace);

    assert!(outcome.completed().is_none());
    assert!(outcome.correspondence().is_none());
    let stop = outcome.stop().expect("multiple subjects must stop");
    assert_eq!(
        stop.source(),
        WorthQueryComparisonStopSource::CorrespondenceDenied
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryComparisonNextAction::NarrowCandidates
    );
    assert_eq!(stop.journey_counters().left_execution_attempt_count(), 1);
    assert_eq!(stop.journey_counters().right_execution_attempt_count(), 1);
    assert_eq!(
        stop.journey_counters()
            .correspondence_resolution_attempt_count(),
        0
    );
}

#[test]
fn exact_single_row_identity_can_prove_lineage_without_raw_consumer_ids() {
    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .lineage();
    let mut workspace = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-lineage")
        .expect("workspace should open");
    write_task(&mut workspace, "only");
    let context = current_and_retained(&workspace).expect("the fixture requires an open workspace");
    let outcome = declaration.using(context).run(&mut workspace);
    let correspondence = outcome.correspondence().expect("lineage should resolve");

    assert_eq!(
        correspondence.posture(),
        WorthQueryComparisonCorrespondencePosture::AuthoritativeContinuity
    );
    assert!(correspondence
        .correspondence()
        .outcome()
        .as_lineage_continuity()
        .is_some());
}

#[test]
fn stale_structural_pair_denies_before_either_query_executes() {
    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .diff();
    let mut workspace = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-stale")
        .expect("workspace should open");
    let context = current_and_retained(&workspace).expect("the fixture requires an open workspace");
    write_task(&mut workspace, "changed");
    let outcome = declaration.using(context).run(&mut workspace);
    let stop = outcome.stop().expect("stale pair must stop");

    assert_eq!(
        stop.source(),
        WorthQueryComparisonStopSource::StaleBasisPair
    );
    assert_eq!(
        stop.next_action(),
        WorthQueryComparisonNextAction::RefreshBasisPair
    );
    assert_eq!(stop.journey_counters().left_execution_attempt_count(), 0);
    assert_eq!(stop.journey_counters().right_execution_attempt_count(), 0);
}

#[test]
fn stale_right_branch_denies_before_either_query_executes() {
    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .lineage();
    let mut left = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-stale-left")
        .expect("left workspace should open");
    let mut right = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-stale-right")
        .expect("right workspace should open");
    let context = between(
        &left,
        test_session_label("stale-left-branch"),
        &right,
        test_session_label("stale-right-branch"),
    )
    .expect("branch bases should admit");
    write_task(&mut right, "advanced-after-binding");

    let outcome = declaration.using(context).run((&mut left, &mut right));
    let stop = outcome.stop().expect("stale branch pair must stop");

    assert_eq!(
        stop.source(),
        WorthQueryComparisonStopSource::StaleBasisPair
    );
    assert_eq!(stop.journey_counters().left_execution_attempt_count(), 0);
    assert_eq!(stop.journey_counters().right_execution_attempt_count(), 0);
}

#[test]
fn branch_context_rejects_single_workspace_execution_before_query_work() {
    let declaration = declare(task_collection_read)
        .expect("comparison declaration should build")
        .diff();
    let mut left = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-resource-left")
        .expect("left workspace should open");
    let right = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-resource-right")
        .expect("right workspace should open");
    let context = between(
        &left,
        test_session_label("resource-left-branch"),
        &right,
        test_session_label("resource-right-branch"),
    )
    .expect("branch bases should admit");

    let outcome = declaration.using(context).run(&mut left);
    let stop = outcome.stop().expect("wrong execution family must stop");

    assert_eq!(
        stop.source(),
        WorthQueryComparisonStopSource::InvalidBasisPair
    );
    assert_eq!(stop.journey_counters().left_execution_attempt_count(), 0);
    assert_eq!(stop.journey_counters().right_execution_attempt_count(), 0);
}

#[test]
fn two_resources_claiming_one_workspace_authority_are_rejected() {
    let left = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-competing-authority")
        .expect("left workspace should open");
    let right = stateful_bridge_task_runtime()
        .workspace("ordinary-comparison-competing-authority")
        .expect("right workspace should open");
    let stop = between(
        &left,
        test_session_label("competing-left-branch"),
        &right,
        test_session_label("competing-right-branch"),
    )
    .expect_err("competing authority must stop");

    assert_eq!(
        stop.source(),
        WorthQueryComparisonStopSource::InvalidBasisPair
    );
    assert_eq!(stop.journey_counters().left_execution_attempt_count(), 0);
    assert_eq!(stop.journey_counters().right_execution_attempt_count(), 0);
}

fn task_collection_read<Output>(
    read: crate::runtime::WorthQueryReadBuilder<Output>,
) -> Result<Output, crate::runtime::WorthQueryReadDenial> {
    read.local_collection(
        "Task",
        manager_schema(),
        |query| {
            query.project(
                AspectFieldSelector::new("identity", "id").expect("identity selector should build"),
            )
        },
        |shape| {
            shape.field(
                AuthoredResultShapeField::new("identity", "id", "id")
                    .expect("identity result field should build"),
            )
        },
    )
}

fn write_task(workspace: &mut crate::runtime::WorthQueryWorkspace, label: &str) {
    workspace
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value(label)),
                ("title.value", test_string_aspect_value(label)),
            ],
        ))
        .expect("task write should advance the snapshot");
}

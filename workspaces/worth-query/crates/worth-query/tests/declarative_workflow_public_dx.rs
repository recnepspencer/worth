use crate::support;

use support::public_bridge_runtime::public_relational_merge_runtime;
use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn mutation_journey_uses_only_mutation_capability_vocabulary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query::facade::mutation::{
        authoritative, declare, WorthQueryAspectTouch, WorthQueryAuthoredAspectValue,
        WorthQueryAuthorityLane,
    };

    let declaration = declare(|mutation| {
        mutation
            .set_aspect(
                WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")?,
                WorthQueryAuthoredAspectValue::string("mutation-dx"),
            )
            .build_insert("Task")
    })
    .expect("mutation should declare");
    let mut workspace = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace("public-mutation-dx")
        .expect("workspace should open");
    let context = authoritative(request_execution, &workspace).expect("authority should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace);
    let completion = outcome.completed().expect("mutation should complete");
    assert_eq!(
        completion.aftermath().authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        completion
            .counters()
            .lower_runtime_execution_completed_count(),
        1
    );
}

#[test]
fn preview_journey_has_distinct_read_only_and_promotion_eligible_states() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query::facade::preview::{
        declare, declare_mutation, promotion, read_only, WorthQueryAspectTouch,
        WorthQueryAuthoredAspectValue, WorthQueryPreviewCloseoutKind, WorthQuerySessionLabel,
    };

    let mut workspace = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace("public-preview-dx")
        .expect("workspace should open");
    let read_label = WorthQuerySessionLabel::scoped_strs("public-preview", ["read-only"])
        .expect("label should build");
    let read_context =
        read_only(request_execution, &workspace, read_label.clone()).expect("context should admit");
    let read_outcome = declare(read_label)
        .using(read_context)
        .open_and_close(request_execution, &mut workspace);
    assert_eq!(
        read_outcome
            .read_only_completion()
            .expect("read-only preview should complete")
            .aftermath()
            .closeout_kind(),
        WorthQueryPreviewCloseoutKind::Discarded
    );

    let promote_label = WorthQuerySessionLabel::scoped_strs("public-preview", ["promote"])
        .expect("label should build");
    let mutation = declare_mutation(|mutation| {
        mutation
            .set_aspect(
                WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")?,
                WorthQueryAuthoredAspectValue::string("preview-dx"),
            )
            .build_insert("Task")
    })
    .expect("mutation should declare");
    let promote_context = promotion(request_execution, &workspace, promote_label.clone())
        .expect("context should admit");
    let promote_outcome = declare(promote_label)
        .with_mutation(mutation)
        .using(promote_context)
        .open_and_close(request_execution, request_execution, &mut workspace);
    let completion = promote_outcome
        .promotion_completion()
        .expect("promotion should complete");
    assert_eq!(
        completion.aftermath().closeout_kind(),
        WorthQueryPreviewCloseoutKind::Promoted
    );
    assert!(!completion
        .promotion_eligibility()
        .identity_for_reporting()
        .is_empty());
}

#[test]
fn workflow_journey_uses_only_workflow_capability_vocabulary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query::facade::workflow::{
        declare, declare_mutation, preview, WorthQueryAspectTouch, WorthQueryAuthoredAspectValue,
        WorthQueryPreviewCloseoutKind, WorthQuerySessionLabel,
    };

    let mutation = declare_mutation(|mutation| {
        mutation
            .set_aspect(
                WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")?,
                WorthQueryAuthoredAspectValue::string("workflow-dx"),
            )
            .build_insert("Task")
    })
    .expect("mutation should declare");
    let label = WorthQuerySessionLabel::scoped_strs("public-workflow", ["promotion"])
        .expect("label should build");
    let declaration = declare(label.clone(), mutation);
    let mut workspace = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace("public-workflow-dx")
        .expect("workspace should open");
    let context = preview(request_execution, &workspace, label).expect("context should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace);
    assert_eq!(
        outcome
            .completed()
            .expect("workflow should complete")
            .aftermath()
            .closeout_kind(),
        WorthQueryPreviewCloseoutKind::Promoted
    );
}

#[test]
fn writeback_journey_uses_only_workflow_capability_vocabulary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query::facade::workflow::{declare_writeback, projected_state_diff, writeback};

    let declaration = declare_writeback(projected_state_diff()).with_rich_inspection();
    let mut workspace = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime(request_execution)
        .workspace("public-writeback-dx")
        .expect("workspace should open");
    let context =
        writeback(request_execution, &workspace).expect("writeback authority should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace);
    let completion = outcome.completed().expect("writeback should complete");

    assert_eq!(completion.receipt().write_count(), 1);
    assert!(completion.diagnostics().is_some());
    assert!(!completion
        .aftermath()
        .execution_receipt_identity_for_reporting()
        .is_empty());
}

#[test]
fn branch_merge_journey_uses_only_workflow_capability_vocabulary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query::facade::workflow::{branch_merge, declare_branch_merge};

    let declaration = declare_branch_merge("main", "candidate")
        .expect("branch merge should declare")
        .with_rich_inspection();
    let mut workspace = PublicBridgeRuntimeHarness::new()
        .bridge_backed_runtime_with_relational(request_execution, public_relational_merge_runtime())
        .workspace("public-branch-merge-dx")
        .expect("workspace should open");
    let context = branch_merge(request_execution, &workspace, &declaration)
        .expect("merge authority should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace);
    let completion = outcome.completed().expect("branch merge should complete");

    assert_eq!(completion.receipt().write_count(), 1);
    assert!(completion.aftermath().commit_id() > 0);
    assert!(completion.diagnostics().is_some());
}

#[test]
fn domain_journey_lowers_contributed_vocabulary_through_query() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query::facade::domain::{
        preview, WorthQueryAspectTouch, WorthQueryAuthoredAspectValue,
        WorthQueryPreviewCloseoutKind, WorthQuerySessionLabel,
    };

    let label = WorthQuerySessionLabel::scoped_strs("public-domain", ["promotion"])
        .expect("label should build");
    let mut workspace = support::installed_domain::workspace(request_execution, "public-domain-dx");
    let handle = workspace
        .domain(support::installed_domain::PublicInstalledDomain)
        .unwrap();
    let declaration = handle
        .mutation(|mutation| {
            mutation
                .set_aspect(
                    WorthQueryAspectTouch::from_authoring_ingress_text("identity.id")?,
                    WorthQueryAuthoredAspectValue::string("domain-dx"),
                )
                .build_insert("Task")
        })
        .expect("domain mutation should declare")
        .workflow(label.clone());
    let context = preview(request_execution, &workspace, label).expect("context should admit");
    let outcome = declaration
        .using(context)
        .run(request_execution, &mut workspace)
        .unwrap();
    assert_eq!(
        outcome
            .completed()
            .expect("domain workflow should complete")
            .completion()
            .aftermath()
            .closeout_kind(),
        WorthQueryPreviewCloseoutKind::Promoted
    );
}

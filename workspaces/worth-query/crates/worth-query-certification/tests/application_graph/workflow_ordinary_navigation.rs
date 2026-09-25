//! Back's published transition and retained evidence agree with ordinary run.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryApplicationRequestExt, WorthQueryOrdinaryWorkflowRunStop,
};

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::{publish_workflow_on_first_program, BoundedDimensionWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        accept_assessment, advance_instance, assessment_join_terminal_definition,
        propose_authoring_instance, publish_definition, settle_assessment, start_instance,
        WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};

fn run_once(
    application: &BoundedDimensionWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    key: u64,
) -> worth_query_host::facade::application_entry::WorthQueryOrdinaryWorkflowRunProgress {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .run_workflow(application, instance)
        .idempotency_keys(&[key])
        .execute()
}

#[test]
fn ordinary_run_after_back_reuses_advanced_evidence_without_a_new_effect() {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        assessment_join_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        919_200,
    )
    .expect("navigation definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("navigation definition did not publish: {other:?}"),
    };
    let start = |key| match start_instance(&application, published.definition().clone(), key)
        .expect("navigation instance starts")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("navigation instance did not start: {other:?}"),
    };
    let advanced = start(919_201);
    let ordinary = start(919_202);
    for (instance, base) in [
        (advanced.clone(), 919_210_u64),
        (ordinary.clone(), 919_220_u64),
    ] {
        assert!(matches!(
            propose_authoring_instance(&application, instance.clone(), base),
            Ok(WorkflowProposalOutcome::Published(_))
        ));
        let first = settle_assessment(&application, instance.clone(), base + 1);
        assert_eq!(first.required().node_path(), "checks/first");
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &first, base + 2),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
        assert!(matches!(
            advance_instance(&application, instance, base + 3),
            Ok(WorkflowProgressOutcome::AwaitingAssessment(required))
                if required.node_path() == "checks/second"
        ));
    }
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let navigate = |instance: PublishedWorkflowInstanceRef, key: u64| {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_navigate_back(&application, instance)
            .expect("Back request receives fresh admission")
            .execute()
            .expect("Back publishes its transition")
    };
    let advanced_back = navigate(advanced.clone(), 919_230);
    let ordinary_back = navigate(ordinary.clone(), 919_231);
    assert_eq!(advanced_back.node_path(), "checks/second");
    assert_eq!(ordinary_back.node_path(), advanced_back.node_path());
    assert!(!advanced_back.replayed());
    assert!(!ordinary_back.replayed());
    for (instance, key, first) in [
        (advanced.clone(), 919_230, advanced_back.transition()),
        (ordinary.clone(), 919_231, ordinary_back.transition()),
    ] {
        let duplicate = navigate(instance, key);
        assert!(duplicate.replayed());
        assert_eq!(duplicate.transition(), first);
    }
    runtime.release_workflow_instance_progress_for_test();
    let WorkflowProgressOutcome::Completed(advanced_reused) =
        advance_instance(&application, advanced, 919_232)
            .expect("advanced retained-evidence advance prepares")
    else {
        panic!("advanced Back did not reuse first assessment");
    };
    let ordinary_reused = run_once(&application, ordinary, 919_233);
    assert!(matches!(
        ordinary_reused.stop(),
        WorthQueryOrdinaryWorkflowRunStop::CallerKeysExhausted
    ));
    assert_eq!(ordinary_reused.attempted_steps(), 1);
    assert_eq!(ordinary_reused.transitions().len(), 1);
    let ordinary_transition = &ordinary_reused.transitions()[0];
    assert_eq!(ordinary_transition.node_path(), advanced_reused.node_path());
    assert_eq!(advanced_reused.node_path(), "checks/first");
    assert!(advanced_reused.assessment_evidence().is_none());
    assert!(ordinary_transition.assessment_evidence().is_none());
}

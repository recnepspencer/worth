use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
        WorthQueryOrdinaryWorkflowPublicationDenial, WorthQueryOrdinaryWorkflowRunStop,
    },
    declaration::application_program::{
        ApplicationWorkflowDefinitionBuilder, AuthoredWorkflowDefinition,
    },
};

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    workflow::{
        accept_assessment, assessment_join_terminal_definition, propose_instance,
        publish_definition, reviewed_geometry_definition, settle_assessment, start_instance,
        ReviewedGeometryWorkflow, WorkflowAdvanceInput, WorkflowAdvanceIntent,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
        WorkflowInstanceStartInput, WorkflowInstanceStartIntent,
    },
};

fn terminal_draft() -> AuthoredWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "ordinary-terminal",
        super::bounded_dimension_model::workflow::definition_limits(),
    )
    .expect("the ordinary workflow identity is valid");
    let done = builder.terminal("done").expect("the terminal is valid");
    builder.start(&done);
    builder.finish().expect("the draft is complete")
}

#[test]
fn ordinary_run_preserves_a_typed_assessment_wait_and_cancellation() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition("ordinary-wait-terminal"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        918_100,
    )
    .expect("reviewed geometry publication prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected published definition: {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), 918_101)
        .expect("reviewed geometry start prepares")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected started instance: {other:?}"),
    };
    propose_instance(&application, instance.clone(), 918_102)
        .expect("reviewed geometry proposal prepares");
    let runtime = application.runtime();
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            std::time::Instant::now() + std::time::Duration::from_secs(60),
            cancellation.token(),
        );
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let run = || {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .run_workflow(&application, instance.clone())
            .idempotency_keys(&[918_103])
            .execute()
    };
    let waiting = run();
    assert_eq!(waiting.attempted_steps(), 1);
    assert!(waiting.transitions().is_empty());
    match waiting.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(
            WorkflowProgressOutcome::AwaitingAssessment(required),
        ) => {
            assert_eq!(required.node_path(), "checks/structural");
        }
        other => panic!("expected typed assessment wait: {other:?}"),
    }
    let settled = settle_assessment(&application, instance.clone(), 918_104);
    assert!(matches!(
        accept_assessment(&application, instance.clone(), &settled, 918_105),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let resumed = run();
    match resumed.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(
            WorkflowProgressOutcome::AwaitingAssessment(required),
        ) => assert_eq!(required.node_path(), "checks/manufacturability"),
        other => panic!("expected the next typed assessment wait: {other:?}"),
    }
    assert!(resumed.transitions().is_empty());
    cancellation.cancel();
    let interrupted = run();
    assert_eq!(interrupted.attempted_steps(), 0);
    assert!(interrupted.transitions().is_empty());
    assert!(matches!(
        interrupted.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Interrupted(_)
    ));
}

#[test]
fn ordinary_run_pumps_join_and_terminal_with_stable_replay_keys() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        assessment_join_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        918_200,
    )
    .expect("join definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected published join definition: {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), 918_201)
        .expect("join instance starts")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected started join instance: {other:?}"),
    };
    propose_instance(&application, instance.clone(), 918_202).expect("proposal settles");
    for (settle_key, accept_key) in [(918_203, 918_204), (918_205, 918_206)] {
        let settled = settle_assessment(&application, instance.clone(), settle_key);
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &settled, accept_key),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let run = |keys: &[u64]| {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .run_workflow(&application, instance.clone())
            .idempotency_keys(keys)
            .execute()
    };
    let partial = run(&[918_207]);
    assert!(matches!(
        partial.stop(),
        WorthQueryOrdinaryWorkflowRunStop::CallerKeysExhausted
    ));
    assert_eq!(partial.transitions().len(), 1);
    assert!(!partial.transitions()[0].replayed());
    let first = run(&[918_207, 918_208]);
    assert!(matches!(
        first.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert_eq!(first.attempted_steps(), 2);
    assert_eq!(first.transitions().len(), 2);
    assert_eq!(first.transitions()[0].node_path(), "checks/join");
    assert!(!first.transitions()[0].terminal());
    assert_eq!(first.transitions()[1].node_path(), "completed");
    assert!(first.transitions()[1].terminal());
    assert!(first.transitions()[0].replayed());
    assert!(!first.transitions()[1].replayed());
    let replay = run(&[918_207, 918_208]);
    assert!(matches!(
        replay.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert_eq!(replay.attempted_steps(), 2);
    assert!(replay
        .transitions()
        .iter()
        .all(|transition| transition.replayed()));
    assert_eq!(
        replay.transitions()[0].transition(),
        first.transitions()[0].transition()
    );
    assert_eq!(
        replay.transitions()[1].transition(),
        first.transitions()[1].transition()
    );
}

#[test]
fn ordinary_publication_uses_installed_binding_and_real_commit_authority() {
    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let key = 918_001_u64;

    let publish = || {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowDefinitionAuthoringIntent {
                input: WorkflowDefinitionAuthoringInput {
                    identity: PART_IDENTITY.to_owned(),
                    dimension: 8,
                },
            })
            .workflow(&application, terminal_draft())
            .expect("the draft binds to installed support")
            .publish(WorkflowDefinitionExpectedPredecessor::Absent)
            .idempotency(&key)
            .execute()
            .expect("the ordinary publication prepares")
    };
    let first = match publish() {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("the ordinary definition must publish: {other:?}"),
    };
    assert!(!first.replayed());
    let replay = match publish() {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("the exact ordinary retry must publish: {other:?}"),
    };
    assert!(replay.replayed());
    assert_eq!(first.definition(), replay.definition());
    let start_key = 918_002_u64;
    let start = || {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowInstanceStartIntent {
                input: WorkflowInstanceStartInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .start_workflow(&application, first.definition().clone())
            .idempotency(&start_key)
            .execute()
            .expect("the ordinary definition start prepares")
    };
    let started = match start() {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("the ordinary definition must start: {other:?}"),
    };
    assert!(!started.replayed());
    assert_eq!(started.instance().start_node_path(), "done");
    let replayed_start = match start() {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("the exact ordinary start must replay: {other:?}"),
    };
    assert!(replayed_start.replayed());
    assert_eq!(started.instance(), replayed_start.instance());

    let run = |keys: &[u64]| {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .run_workflow(&application, started.instance().clone())
            .idempotency_keys(keys)
            .execute()
    };
    let empty = run(&[]);
    assert!(matches!(
        empty.stop(),
        WorthQueryOrdinaryWorkflowRunStop::CallerKeysExhausted
    ));
    assert_eq!(empty.attempted_steps(), 0);
    let progressed = run(&[918_003]);
    assert!(matches!(
        progressed.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert_eq!(progressed.attempted_steps(), 1);
    assert_eq!(progressed.transitions().len(), 1);
    assert!(progressed.transitions()[0].terminal());
    assert!(!progressed.transitions()[0].replayed());
    let replayed = run(&[918_003]);
    assert!(matches!(
        replayed.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert!(replayed.transitions()[0].replayed());
    assert_eq!(
        progressed.transitions()[0].transition(),
        replayed.transitions()[0].transition()
    );
}

#[test]
fn ordinary_publication_rejects_invalid_draft_before_mutation_preparation() {
    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let draft = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "missing-start",
        super::bounded_dimension_model::workflow::definition_limits(),
    )
    .expect("the identity is valid")
    .finish()
    .expect("the incomplete draft can be represented");
    let result = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&application, draft);
    assert!(matches!(
        result,
        Err(WorthQueryOrdinaryWorkflowPublicationDenial::InvalidDefinition(_))
    ));
}

#[test]
fn ordinary_publication_rejects_a_workflow_runtime_from_another_application() {
    let application = publish_workflow_on_first_program();
    let foreign = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let result = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&foreign, terminal_draft());
    assert!(matches!(
        result,
        Err(WorthQueryOrdinaryWorkflowPublicationDenial::RuntimeMismatch)
    ));
}

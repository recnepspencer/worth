//! Common/advanced parity for typed Phase 3 condition control.

pub use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowCondition, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryApplicationRequestExt, WorthQueryOrdinaryWorkflowRunStop,
};

pub use super::document_retention_model::{
    host::{publish_workflow_on_first_program, DocumentWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_retention,
    retention_entry::{
        DocumentRetentionConditionQueryBinding, DocumentRetentionConditionRead, DOCUMENT_IDENTITY,
    },
    settled_verdict::{settle, RetentionVerdict},
    workflow::{
        advance_instance, bounded_retry_definition, condition_terminal_definition,
        condition_terminal_draft, propose_authoring_instance, publish_definition, run_instance,
        start_instance, WorkflowAdvanceInput, WorkflowAdvanceIntent,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
    },
};

pub fn accept_condition(
    application: &DocumentWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    required: &RequiredWorkflowCondition,
    key: u64,
    expected_positive: bool,
) -> WorkflowProgressOutcome {
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let result = runtime
        .request(&principal, &scope)
        .query(DocumentRetentionConditionRead {
            identity: DOCUMENT_IDENTITY.to_owned(),
        })
        .execute()
        .expect("the installed condition query executes");
    assert_eq!(result.rows(), &[expected_positive]);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                document_identity: DOCUMENT_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(application, instance)
        .expect("typed condition acceptance prepares")
        .condition(required)
        .operand::<DocumentRetentionConditionQueryBinding, _>("retained", result)
        .accept()
        .expect("exact typed condition is accepted")
}

pub fn assert_condition_path_parity(expected_positive: bool) {
    let advanced = publish_workflow_on_first_program();
    let ordinary = publish_workflow_on_first_program();
    if !expected_positive {
        for (application, key) in [(&advanced, 919_020), (&ordinary, 919_021)] {
            assert_eq!(
                settle(set_retention(
                    application.program_runtime(),
                    application.program_runtime().current_world(),
                    0,
                    key,
                )),
                RetentionVerdict::Performed(0)
            );
        }
    }
    let validated = condition_terminal_definition();
    let advanced_definition = match publish_definition(
        &advanced,
        validated,
        WorkflowDefinitionExpectedPredecessor::Absent,
        919_000,
    )
    .expect("advanced condition publication prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("advanced definition did not publish: {other:?}"),
    };
    let runtime = ordinary.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let ordinary_definition = match runtime
        .with_host_advancement(|host_phase| {
            runtime
                .request(&principal, &scope)
                .mutate(WorkflowDefinitionAuthoringIntent {
                    input: WorkflowDefinitionAuthoringInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 8,
                    },
                })
                .workflow(&host_phase, &ordinary, condition_terminal_draft())
                .expect("ordinary condition draft binds")
                .publish(WorkflowDefinitionExpectedPredecessor::Absent)
                .idempotency(&919_010)
        })
        .expect("installed host call opens")
        .execute()
        .expect("ordinary condition publication prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("ordinary definition did not publish: {other:?}"),
    };
    assert_eq!(
        ordinary_definition.definition().content_identity(),
        advanced_definition.definition().content_identity()
    );
    let advanced_instance =
        match start_instance(&advanced, advanced_definition.definition().clone(), 919_001)
            .expect("advanced instance starts")
        {
            WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
            other => panic!("advanced instance did not start: {other:?}"),
        };
    let ordinary_instance =
        match start_instance(&ordinary, ordinary_definition.definition().clone(), 919_011)
            .expect("ordinary instance starts")
        {
            WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
            other => panic!("ordinary instance did not start: {other:?}"),
        };
    for (application, instance, key) in [
        (&advanced, advanced_instance.clone(), 919_002),
        (&ordinary, ordinary_instance.clone(), 919_012),
    ] {
        assert!(matches!(
            propose_authoring_instance(application, instance, key),
            Ok(WorkflowProposalOutcome::Published(_))
        ));
    }
    let advanced_required = match advance_instance(&advanced, advanced_instance.clone(), 919_003)
        .expect("advanced condition wait prepares")
    {
        WorkflowProgressOutcome::AwaitingCondition(required) => required,
        other => panic!("advanced condition wait differs: {other:?}"),
    };
    let ordinary_wait = run_instance(&ordinary, ordinary_instance.clone(), &[919_013]);
    assert_eq!(ordinary_wait.attempted_steps(), 1);
    assert!(ordinary_wait.transitions().is_empty());
    let ordinary_required = match ordinary_wait.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingCondition(
            required,
        )) => required,
        other => panic!("ordinary condition wait differs: {other:?}"),
    };
    assert_eq!(ordinary_required.node_path(), advanced_required.node_path());
    assert_eq!(
        ordinary_required.transition_identity(),
        advanced_required.transition_identity()
    );
    assert_eq!(
        ordinary_required.occurrence(),
        advanced_required.occurrence()
    );
    assert_eq!(ordinary_required.operands(), advanced_required.operands());
    let advanced_accepted = accept_condition(
        &advanced,
        advanced_instance.clone(),
        &advanced_required,
        919_004,
        expected_positive,
    );
    let ordinary_accepted = accept_condition(
        &ordinary,
        ordinary_instance.clone(),
        ordinary_required,
        919_014,
        expected_positive,
    );
    let (
        WorkflowProgressOutcome::Completed(advanced_performed),
        WorkflowProgressOutcome::Completed(ordinary_performed),
    ) = (advanced_accepted, ordinary_accepted)
    else {
        panic!("both condition acceptances must publish");
    };
    assert_eq!(
        advanced_performed.node_path(),
        ordinary_performed.node_path()
    );
    let WorkflowProgressOutcome::Completed(advanced_terminal) =
        advance_instance(&advanced, advanced_instance, 919_005)
            .expect("advanced terminal prepares")
    else {
        panic!("advanced condition did not reach its terminal");
    };
    let ordinary_terminal = run_instance(&ordinary, ordinary_instance, &[919_015]);
    assert!(matches!(
        ordinary_terminal.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert_eq!(ordinary_terminal.transitions().len(), 1);
    assert_eq!(
        ordinary_terminal.transitions()[0].node_path(),
        advanced_terminal.node_path()
    );
    assert_eq!(
        advanced_terminal.node_path(),
        if expected_positive {
            "satisfied"
        } else {
            "unsatisfied"
        }
    );
}

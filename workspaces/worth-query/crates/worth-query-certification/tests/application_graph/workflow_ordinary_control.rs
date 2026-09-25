//! Common/advanced parity for typed Phase 3 condition control.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowCondition, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorthQueryApplicationRequestExt,
    WorthQueryOrdinaryWorkflowRunProgress, WorthQueryOrdinaryWorkflowRunStop,
};

use super::bounded_dimension_model::{
    dimension_entry::{
        PartDimensionConditionQueryBinding, PartDimensionConditionRead, PART_IDENTITY,
    },
    host::{publish_workflow_on_first_program, BoundedDimensionWorkflowRuntime},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        advance_instance, condition_terminal_definition, condition_terminal_draft,
        propose_authoring_instance, publish_definition, start_instance, WorkflowAdvanceInput,
        WorkflowAdvanceIntent, WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
    },
};

fn run(
    application: &BoundedDimensionWorkflowRuntime,
    instance: PublishedWorkflowInstanceRef,
    keys: &[u64],
) -> WorthQueryOrdinaryWorkflowRunProgress {
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
        .idempotency_keys(keys)
        .execute()
}

fn accept_condition(
    application: &BoundedDimensionWorkflowRuntime,
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
        .query(PartDimensionConditionRead {
            identity: PART_IDENTITY.to_owned(),
        })
        .execute()
        .expect("the installed condition query executes");
    assert_eq!(result.rows(), &[expected_positive]);
    runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&key)
        .prepare_workflow_advance(application, instance)
        .expect("typed condition acceptance prepares")
        .accept_condition::<PartDimensionConditionQueryBinding>(required, result)
        .expect("exact typed condition is accepted")
}

fn assert_condition_path_parity(expected_positive: bool) {
    let advanced = publish_workflow_on_first_program();
    let ordinary = publish_workflow_on_first_program();
    if !expected_positive {
        for (application, key) in [(&advanced, 919_020), (&ordinary, 919_021)] {
            assert_eq!(
                settle(set_dimension(
                    application.program_runtime(),
                    application.program_runtime().current_world(),
                    0,
                    key,
                )),
                DimensionVerdict::Performed(0)
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
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&ordinary, condition_terminal_draft())
        .expect("ordinary condition draft binds")
        .publish(WorkflowDefinitionExpectedPredecessor::Absent)
        .idempotency(&919_010)
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
    let ordinary_wait = run(&ordinary, ordinary_instance.clone(), &[919_013]);
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
    assert_eq!(ordinary_required.query(), advanced_required.query());
    assert_eq!(
        ordinary_required.parameter_type(),
        advanced_required.parameter_type()
    );
    assert_eq!(
        ordinary_required.result_type(),
        advanced_required.result_type()
    );
    assert_eq!(ordinary_required.binding(), advanced_required.binding());
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
    let ordinary_terminal = run(&ordinary, ordinary_instance, &[919_015]);
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

#[test]
fn ordinary_and_advanced_satisfied_condition_share_meaning_wait_and_terminal() {
    assert_condition_path_parity(true);
}

#[test]
fn ordinary_and_advanced_unsatisfied_condition_share_meaning_wait_and_terminal() {
    assert_condition_path_parity(false);
}

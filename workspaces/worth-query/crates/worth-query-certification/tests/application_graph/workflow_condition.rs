use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryApplicationRequestExt,
};

use super::bounded_dimension_model::{
    dimension_entry::{
        PartDimensionConditionQueryBinding, PartDimensionConditionRead, PART_IDENTITY,
    },
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        advance_instance, condition_terminal_definition, propose_authoring_instance,
        publish_definition, start_instance, WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};

#[test]
fn false_typed_query_condition_routes_to_unsatisfied_terminal() {
    let application = publish_workflow_on_first_program();
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.program_runtime().current_world(),
            0,
            910,
        )),
        DimensionVerdict::Performed(0)
    );
    let definition = match publish_definition(
        &application,
        condition_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        911,
    )
    .expect("condition definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected published condition definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), 912)
        .expect("condition instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected started condition instance, got {other:?}"),
    };
    assert!(matches!(
        propose_authoring_instance(&application, instance.clone(), 913),
        Ok(WorkflowProposalOutcome::Published(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 914)
        .expect("condition requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingCondition(required) => required,
        other => panic!("expected condition requirement, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let condition_result = runtime
        .request(&principal, &scope)
        .query(PartDimensionConditionRead {
            identity: PART_IDENTITY.to_owned(),
        })
        .execute()
        .expect("the installed condition query must execute");
    assert_eq!(condition_result.rows(), &[false]);
    let outcome = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&915_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("condition acceptance must prepare")
        .accept_condition::<PartDimensionConditionQueryBinding>(&required, condition_result)
        .expect("the false typed condition result must be accepted");
    assert!(matches!(outcome, WorkflowProgressOutcome::Completed(_)));
    match advance_instance(&application, instance, 916)
        .expect("the unsatisfied terminal must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "unsatisfied")
        }
        other => panic!("expected unsatisfied terminal, got {other:?}"),
    }
}

#[test]
fn typed_query_condition_routes_and_duplicate_acceptance_replays() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        condition_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        900,
    )
    .expect("condition definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected published condition definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), 901)
        .expect("condition instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected started condition instance, got {other:?}"),
    };
    match propose_authoring_instance(&application, instance.clone(), 902)
        .expect("condition proposal must prepare")
    {
        WorkflowProposalOutcome::Published(performed) => {
            assert_eq!(performed.node_path(), "proposal")
        }
        other => panic!("expected published proposal, got {other:?}"),
    }
    let required = match advance_instance(&application, instance.clone(), 903)
        .expect("condition requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingCondition(required) => required,
        other => panic!("expected condition requirement, got {other:?}"),
    };

    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let condition_result = runtime
        .request(&principal, &scope)
        .query(PartDimensionConditionRead {
            identity: PART_IDENTITY.to_owned(),
        })
        .execute()
        .expect("the installed condition query must execute");
    assert_eq!(condition_result.rows(), &[true]);
    let outcome = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&904_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("condition acceptance must prepare")
        .accept_condition::<PartDimensionConditionQueryBinding>(&required, condition_result)
        .expect("the exact typed condition result must be accepted");
    match outcome {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "positive-dimension");
            assert!(!performed.replayed());
        }
        other => panic!("expected condition settlement, got {other:?}"),
    }

    let replay_result = runtime
        .request(&principal, &scope)
        .query(PartDimensionConditionRead {
            identity: PART_IDENTITY.to_owned(),
        })
        .execute()
        .expect("the replay condition query must execute");
    let replay = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&904_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("condition replay must prepare")
        .accept_condition::<PartDimensionConditionQueryBinding>(&required, replay_result)
        .expect("the duplicate condition acceptance must resolve");
    match replay {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "positive-dimension");
            assert!(performed.replayed());
        }
        other => panic!("expected replayed condition settlement, got {other:?}"),
    }

    match advance_instance(&application, instance, 905)
        .expect("the satisfied terminal must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "satisfied")
        }
        other => panic!("expected satisfied terminal, got {other:?}"),
    }
}

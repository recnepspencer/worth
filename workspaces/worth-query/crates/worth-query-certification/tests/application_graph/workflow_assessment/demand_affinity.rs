//! A product-supplied assessment requirement is checked by the Query owner.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryWorkflowAssessmentDemandPreparationDenialKind,
};

use super::super::bounded_dimension_model::{
    assessment_output::PartAssessmentDemand,
    operator_identity::{authenticate_operator, request_scope},
    workflow::{WorkflowAdvanceInput, WorkflowAdvanceIntent},
};
use super::*;

#[test]
fn demand_start_refuses_another_instance_requirement_before_source_work() {
    let application = publish_workflow_on_first_program();
    let published = publish_definition(
        &application,
        reviewed_geometry_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        60_000,
    )
    .expect("definition prepares");
    let WorkflowDefinitionPublicationOutcome::Published(published) = published else {
        panic!("definition did not publish: {published:?}");
    };
    let instances = [60_001, 60_002].map(|key| {
        let started = start_instance(&application, published.definition().clone(), key)
            .expect("instance prepares");
        let WorkflowInstanceStartOutcome::Started(started) = started else {
            panic!("instance did not start: {started:?}");
        };
        propose_instance(&application, started.instance().clone(), key + 10)
            .expect("each instance proposes");
        started.instance().clone()
    });
    let required = [0, 1].map(|index| {
        match advance_instance(
            &application,
            instances[index].clone(),
            60_020 + index as u64,
        )
        .expect("assessment requirement prepares")
        {
            WorkflowProgressOutcome::AwaitingAssessment(required) => required,
            other => panic!("expected assessment requirement: {other:?}"),
        }
    });
    assert_eq!(required[0].node_path(), required[1].node_path());
    assert_ne!(required[0].instance(), required[1].instance());

    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let prepare = |key: u64| {
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&key)
            .prepare_workflow_advance(&application, instances[1].clone())
            .expect("second instance has a current assessment")
    };
    let denied = prepare(60_030)
        .into_assessment_demand_for(&required[0], PartAssessmentDemand::new(PART_IDENTITY))
        .err()
        .expect("foreign requirement cannot start a demand");
    assert_eq!(
        denied.kind(),
        WorthQueryWorkflowAssessmentDemandPreparationDenialKind::RequirementMismatch
    );
    let admitted = prepare(60_031)
        .into_assessment_demand_for(&required[1], PartAssessmentDemand::new(PART_IDENTITY))
        .expect("the current owner-issued requirement matches exactly");
    assert_eq!(admitted.required(), &required[1]);
}

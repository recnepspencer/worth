//! Public-host component authoring compiles into ordinary published progression.

use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorthQueryApplicationRequestExt,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentBuilder, ApplicationWorkflowControlOutcome,
        ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowEvidenceJoinPolicy,
        AuthoredWorkflowDefinition,
    },
};

use super::bounded_dimension_model::{
    dimension_entry::{ReviewedSetPartDimensionBinding, PART_IDENTITY},
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    schema::PartDimensionQuery,
    workflow::{
        advance_instance, definition_limits, propose_instance, start_instance,
        ReviewedGeometryWorkflow, WorkflowApprovalCapability, WorkflowDefinitionAuthoringInput,
        WorkflowDefinitionAuthoringIntent, WorkflowDefinitionAuthoringOperation,
    },
};

fn reviewed_component_draft() -> AuthoredWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut review = ApplicationWorkflowComponentBuilder::<ReviewedGeometryWorkflow>::new(
        "required-geometry-review",
    )
    .expect("component identity is valid");
    let structural = review
        .assessment::<PartDimensionQuery>("structural")
        .expect("structural assessment is valid");
    let independent = review
        .assessment::<PartDimensionQuery>("independent")
        .expect("independent assessment is valid");
    let evidence = review
        .evidence_join(
            "evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("join is valid");
    review
        .control(
            &structural,
            ApplicationWorkflowControlOutcome::Completed,
            &independent,
        )
        .expect("structural control is valid")
        .control(
            &independent,
            ApplicationWorkflowControlOutcome::Completed,
            &evidence,
        )
        .expect("independent control is valid")
        .assessment_evidence(&structural, &evidence)
        .expect("structural evidence is valid")
        .assessment_evidence(&independent, &evidence)
        .expect("independent evidence is valid");
    let structural_input = review
        .input_port("structural-subject", &structural)
        .expect("structural port is valid");
    let independent_input = review
        .input_port("independent-subject", &independent)
        .expect("independent port is valid");
    let evidence_output = review
        .output_port("reviewed-evidence", &evidence)
        .expect("evidence port is valid");
    let review = review.finish().expect("component closes");

    let mut workflow = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "ordinary-reviewed-geometry",
        definition_limits(),
    )
    .expect("workflow identity is valid");
    let proposal = workflow
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .expect("proposal is valid");
    let approval = workflow
        .approval::<WorkflowApprovalCapability>("approval")
        .expect("approval is valid");
    let apply = workflow
        .operation_binding::<ReviewedSetPartDimensionBinding>("apply")
        .expect("effect is valid");
    let completed = workflow.terminal("completed").expect("terminal is valid");
    let rejected = workflow.terminal("rejected").expect("terminal is valid");
    let checks = workflow
        .expand_component("checks", &review)
        .expect("component expands");
    let structural = checks
        .input(&structural_input)
        .expect("exported input binds");
    let independent = checks
        .input(&independent_input)
        .expect("exported input binds");
    let evidence = checks
        .output(&evidence_output)
        .expect("exported output binds");
    workflow
        .start(&proposal)
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &structural,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &evidence,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &rejected,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Approved,
            &apply,
        )
        .control(
            &approval,
            ApplicationWorkflowControlOutcome::Rejected,
            &rejected,
        )
        .control(
            &apply,
            ApplicationWorkflowControlOutcome::Completed,
            &completed,
        )
        .proposal_for_assessment(&proposal, &structural)
        .proposal_for_assessment(&proposal, &independent)
        .proposal_for_approval(&proposal, &approval)
        .joined_evidence(&evidence, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&proposal, &apply);
    workflow.finish().expect("reviewed workflow closes")
}

#[test]
fn public_component_ports_validate_and_compile_into_ordinary_progression() {
    let validated = reviewed_component_draft()
        .validate()
        .expect("the public component definition validates");
    let expansions = validated.component_expansions();
    assert_eq!(expansions.len(), 1);
    let review = &expansions[0];
    assert_eq!(review.component().as_str(), "required-geometry-review");
    assert_eq!(review.occurrence_path(), "checks");
    assert_eq!(review.ports().len(), 3);
    for (port, node) in [
        ("structural-subject", "checks/structural"),
        ("reviewed-evidence", "checks/evidence"),
    ] {
        let binding = review
            .ports()
            .iter()
            .find(|binding| binding.identity() == port)
            .expect("the public port remains in expansion provenance");
        assert_eq!(binding.expanded_node().as_str(), node);
    }
    assert_eq!(review.nodes().len(), 3);
    assert_eq!(review.nodes()[0].authored().as_str(), "structural");
    assert_eq!(review.nodes()[0].expanded().as_str(), "checks/structural");

    let application = publish_workflow_on_first_program();
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let published = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&application, reviewed_component_draft())
        .expect("the component binds to installed vocabulary")
        .publish(WorkflowDefinitionExpectedPredecessor::Absent)
        .idempotency(&918_300)
        .execute()
        .expect("the ordinary publication prepares");
    let definition = match published {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("the definition must publish: {other:?}"),
    };
    let instance = match start_instance(&application, definition, 918_301)
        .expect("the published definition starts")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("the instance must start: {other:?}"),
    };
    propose_instance(&application, instance.clone(), 918_302).expect("the proposal prepares");
    let progress = advance_instance(&application, instance, 918_303)
        .expect("the expanded definition advances");
    match progress {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "checks/structural");
        }
        other => panic!("the published component must demand structural review: {other:?}"),
    }
}

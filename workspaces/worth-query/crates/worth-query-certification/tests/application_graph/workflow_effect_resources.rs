//! Real workflow effects must fit the installed operation's candidate envelope.

use worth_query_host::facade::{
    application_entry::{
        WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
        WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome,
        WorkflowProgressOutcome, WorkflowProposalOutcome, WorkflowTransitionPreparationDenial,
        WorthQueryWorkflowAdvancePreparationDenial,
    },
    declaration::application_program::{
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
        ApplicationWorkflowEvidenceJoinPolicy, ValidatedWorkflowDefinition,
    },
    primary_graph::WorthQueryApplicationAttemptDenialKind,
};

use super::bounded_dimension_model::{
    dimension_entry::ReviewedSetPartDimensionBinding,
    host::{publish_workflow_on_first_program, BoundedDimensionWorkflowRuntime},
    schema::PartDimensionQuery,
    workflow::{
        accept_assessment, advance_instance, approve_instance, definition_limits, propose_instance,
        publish_definition, settle_assessment, start_instance, ReviewedGeometryWorkflow,
        WorkflowApprovalCapability, WorkflowDefinitionAuthoringOperation,
    },
};

#[test]
fn approval_effect_links_are_admitted_against_the_installed_operation_ceiling() {
    // The installed approval binding allows eight links. Approval emits four
    // fixed links plus one per retained evidence entity: four reviews fit;
    // five exceed the real candidate envelope during transition preparation.
    for (review_count, key) in [(4, 52_000), (5, 53_000)] {
        let (application, instance, proposal, required) = approval_ready(review_count, key);
        let result = approve_instance(
            &application,
            instance,
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
            key + 100,
        );
        if review_count == 4 {
            assert!(matches!(result, Ok(WorkflowProgressOutcome::Completed(_))));
        } else {
            assert!(
                matches!(
                    &result,
                    Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                        WorkflowTransitionPreparationDenial::Attempt(attempt)
                    )) if attempt.kind() == WorthQueryApplicationAttemptDenialKind::CandidateCapacityExceeded
                ),
                "expected the installed candidate ceiling to deny approval, got {result:?}"
            );
        }
    }
}

fn approval_ready(
    review_count: usize,
    key: u64,
) -> (
    BoundedDimensionWorkflowRuntime,
    worth_query_host::facade::application_entry::PublishedWorkflowInstanceRef,
    worth_query_host::facade::application_entry::PublishedWorkflowProposalRef,
    worth_query_host::facade::application_entry::RequiredWorkflowApproval,
) {
    let application = publish_workflow_on_first_program();
    let definition = publish_definition(
        &application,
        reviewed_definition(review_count),
        WorkflowDefinitionExpectedPredecessor::Absent,
        key,
    )
    .expect("the bounded definition must prepare");
    let WorkflowDefinitionPublicationOutcome::Published(definition) = definition else {
        panic!("the bounded definition must publish: {definition:?}");
    };
    let started = start_instance(&application, definition.definition().clone(), key + 1)
        .expect("the bounded instance must prepare");
    let WorkflowInstanceStartOutcome::Started(started) = started else {
        panic!("the bounded instance must start: {started:?}");
    };
    let instance = started.instance().clone();
    let proposed = propose_instance(&application, instance.clone(), key + 2)
        .expect("the proposal must prepare");
    let WorkflowProposalOutcome::Published(proposed) = proposed else {
        panic!("the proposal must publish: {proposed:?}");
    };
    for index in 0..review_count {
        let settlement =
            settle_assessment(&application, instance.clone(), key + 3 + 2 * index as u64);
        assert!(matches!(
            accept_assessment(
                &application,
                instance.clone(),
                &settlement,
                key + 4 + 2 * index as u64,
            ),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    let joined = advance_instance(
        &application,
        instance.clone(),
        key + 3 + 2 * review_count as u64,
    );
    assert!(
        matches!(joined, Ok(WorkflowProgressOutcome::Completed(_))),
        "expected joined evidence, got {joined:?}"
    );
    let required = match advance_instance(
        &application,
        instance.clone(),
        key + 4 + 2 * review_count as u64,
    )
    .expect("the approval requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected approval requirement, got {other:?}"),
    };
    (application, instance, proposed.proposal().clone(), required)
}

fn reviewed_definition(
    review_count: usize,
) -> ValidatedWorkflowDefinition<ReviewedGeometryWorkflow> {
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        format!("effect-budget-{review_count}"),
        definition_limits(),
    )
    .expect("the workflow identity is valid");
    let propose = builder
        .operation::<WorkflowDefinitionAuthoringOperation>("propose", false)
        .unwrap();
    let reviews = (0..review_count)
        .map(|index| {
            builder
                .assessment::<PartDimensionQuery>(format!("review/{index}"))
                .unwrap()
        })
        .collect::<Vec<_>>();
    let join = builder
        .evidence_join(
            "reviews",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .unwrap();
    let approval = builder
        .approval::<WorkflowApprovalCapability>("approval")
        .unwrap();
    let apply = builder
        .operation_binding::<ReviewedSetPartDimensionBinding>("apply")
        .unwrap();
    let completed = builder.terminal("completed").unwrap();
    let rejected = builder.terminal("rejected").unwrap();
    builder.start(&propose).control(
        &propose,
        ApplicationWorkflowControlOutcome::Completed,
        &reviews[0],
    );
    for pair in reviews.windows(2) {
        builder.control(
            &pair[0],
            ApplicationWorkflowControlOutcome::Completed,
            &pair[1],
        );
    }
    builder.control(
        reviews.last().unwrap(),
        ApplicationWorkflowControlOutcome::Completed,
        &join,
    );
    builder
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &approval,
        )
        .control(
            &join,
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
        .proposal_for_approval(&propose, &approval)
        .joined_evidence(&join, &approval)
        .approval_authority(&approval, &apply)
        .operation_input(&propose, &apply);
    for review in &reviews {
        builder
            .proposal_for_assessment(&propose, review)
            .assessment_evidence(review, &join);
    }
    builder
        .finish()
        .expect("the bounded authored graph is complete")
        .validate()
        .expect("the bounded graph is valid before runtime effects")
}

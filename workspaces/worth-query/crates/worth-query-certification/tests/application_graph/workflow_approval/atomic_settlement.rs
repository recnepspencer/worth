//! Commit-boundary refusal cannot publish half of a guarded operation.

use super::*;

#[test]
fn invariant_rejection_after_handler_stages_neither_mutation_nor_transition() {
    const REJECTED_DIMENSION: u64 = 21;
    let application =
        super::super::bounded_dimension_model::host::publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition("applied"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        2_100,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("expected a published definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition, 2_101).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected a started instance, got {other:?}"),
    };
    let proposal = match super::super::bounded_dimension_model::workflow::propose_authoring_instance_with_dimension(
        &application,
        instance.clone(),
        2_102,
        REJECTED_DIMENSION,
    )
    .unwrap()
    {
        WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
        other => panic!("expected a published proposal, got {other:?}"),
    };
    for (settlement_key, acceptance_key) in [(2_103, 2_104), (2_105, 2_106)] {
        let settlement = settle_assessment(&application, instance.clone(), settlement_key);
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &settlement, acceptance_key),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    assert!(matches!(
        advance_instance(&application, instance.clone(), 2_107),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let approval = match advance_instance(&application, instance.clone(), 2_108).unwrap() {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected an approval requirement, got {other:?}"),
    };
    assert!(matches!(
        approve_instance(
            &application,
            instance.clone(),
            &approval,
            &proposal,
            WorkflowApprovalDecision::Approve,
            2_109,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 2_110).unwrap() {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an operation requirement, got {other:?}"),
    };
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    reset_candidate_count(REJECTED_DIMENSION);
    let result = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: REJECTED_DIMENSION,
            },
        })
        .without_source()
        .idempotency(&2_111_u64)
        .for_workflow_operation(&application, &required)
        .expect("the exact proposed operation binds")
        .execute_in_program(application.program_runtime())
        .expect("the handler must reach the commit boundary");
    assert!(matches!(
        result,
        WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationCommitOutcome::Denied(denial))
            if denial.kind() == WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    ));
    assert_eq!(
        candidate_count(REJECTED_DIMENSION),
        1,
        "the handler staged its write before commit refusal"
    );
    assert_eq!(read_dimension(runtime, instance.branch()), SEED_DIMENSION);
    assert!(matches!(
        advance_instance(&application, instance, 2_112),
        Ok(WorkflowProgressOutcome::AwaitingOperation(_))
    ));
}

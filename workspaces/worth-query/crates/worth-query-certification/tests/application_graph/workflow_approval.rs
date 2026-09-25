//! Certification of admitted workflow approval decisions and exact replay.

use worth_query_host::facade::application_entry::{
    PublishedWorkflowDefinitionRef, PublishedWorkflowInstanceRef, PublishedWorkflowProposalRef,
    RequiredWorkflowApproval, WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorkflowTransitionPreparationDenial,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitOutcome,
};

use super::bounded_dimension_model::{
    dimension_entry::{
        ReviewedSetPartDimensionBinding, ReviewedSetPartDimensionIntent, SetPartDimensionIntent,
        PART_IDENTITY,
    },
    host::{BoundedDimensionWorkflowRuntime, SEED_DIMENSION},
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_dimension,
    readback::read_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        accept_assessment, advance_instance, approve_instance, propose_instance,
        publish_definition, reviewed_geometry_definition,
        reviewed_geometry_definition_with_join_policy, settle_assessment, start_instance,
        WorkflowAdvanceInput, WorkflowAdvanceIntent,
    },
};

#[path = "workflow_approval/authentication.rs"]
mod authentication;
#[path = "workflow_approval/commit_boundary.rs"]
mod commit_boundary;
#[path = "workflow_approval/custody.rs"]
mod custody;
#[path = "workflow_approval/effect_currentness.rs"]
mod effect_currentness;
#[path = "workflow_approval/journey.rs"]
mod journey;
#[path = "workflow_approval/operation_requirement.rs"]
mod operation_requirement;
use journey::approval_journey;
#[path = "workflow_approval/proposal.rs"]
mod proposal;
#[path = "workflow_approval/rejection.rs"]
mod rejection;
#[path = "workflow_approval/replay.rs"]
mod replay;
#[path = "workflow_approval/wait_revocation.rs"]
mod wait_revocation;

#[test]
fn approved_operation_requires_and_consumes_the_exact_performed_effect() {
    let (application, _, instance, proposal, approval, _) = approval_journey("applied", 800);
    match approve_instance(
        &application,
        instance.clone(),
        &approval,
        &proposal,
        WorkflowApprovalDecision::Approve,
        810,
    ) {
        Ok(WorkflowProgressOutcome::Completed(_)) => {}
        other => panic!("expected approval to complete, got {other:?}"),
    }
    let before = application.runtime().workflow_instance_progress_counters();
    let warm_required = match advance_instance(&application, instance.clone(), 811)
        .expect("the approved effect requirement must prepare")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected an operation requirement, got {other:?}"),
    };
    let after = application.runtime().workflow_instance_progress_counters();
    assert_eq!(after.warm_core_hits(), before.warm_core_hits() + 1);
    assert_eq!(
        after.warm_history_transition_visits(),
        before.warm_history_transition_visits()
    );
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let before_reconstruction = application.runtime().workflow_instance_progress_counters();
    let required = match advance_instance(&application, instance.clone(), 818)
        .expect("the operation requirement must reconstruct after projection loss")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected a reconstructed operation requirement, got {other:?}"),
    };
    let after_reconstruction = application.runtime().workflow_instance_progress_counters();
    operation_requirement::assert_same(&warm_required, &required);
    assert_eq!(
        after_reconstruction.cold_misses(),
        before_reconstruction.cold_misses() + 1
    );
    assert!(
        after_reconstruction.cold_reconstruction_transition_visits()
            > before_reconstruction.cold_reconstruction_transition_visits()
    );
    let runtime = application.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let unbound = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&812_u64)
        .execute_in_program(application.program_runtime());
    assert!(matches!(
        unbound,
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::RequiresWorkflowTransition)
    ));
    assert_eq!(read_dimension(runtime, instance.branch()), SEED_DIMENSION);
    let sibling = runtime
        .branches()
        .fork(instance.branch())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("the sibling branch publishes");
    let wrong_branch = runtime
        .request(&principal, &scope)
        .on_branch(sibling)
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&812_u64)
        .for_workflow_operation(&application, &required);
    assert!(matches!(
        wrong_branch,
        Err(worth_query_host::facade::application_entry::WorthQueryWorkflowOperationBindingDenial::RequirementMismatch)
    ));
    let wrong_binding = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(SetPartDimensionIntent {
            input: super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&813_u64)
        .for_workflow_operation(&application, &required);
    assert!(matches!(
        wrong_binding,
        Err(worth_query_host::facade::application_entry::WorthQueryWorkflowOperationBindingDenial::RequirementMismatch)
    ));
    let wrong_input = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 9,
            },
        })
        .without_source()
        .idempotency(&814_u64)
        .for_workflow_operation(&application, &required);
    match wrong_input {
        Err(denial) => assert_eq!(
            denial,
            worth_query_host::facade::application_entry::WorthQueryWorkflowOperationBindingDenial::RequirementMismatch
        ),
        Ok(_) => panic!("a different proposed input must not bind"),
    }
    let effect = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&815_u64)
        .for_workflow_operation(&application, &required)
        .expect("the effect request matches the durable operation requirement")
        .execute_in_program(application.program_runtime())
        .expect("the approved effect request must execute");
    let receipt = match effect {
        WorthQueryApplicationMutationOutcome::Committed { receipt, .. } => receipt,
        other => panic!("expected the approved effect to commit, got {other:?}"),
    };
    assert_eq!(read_dimension(runtime, instance.branch()), 8);
    let completed = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&816_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("the effect acceptance must prepare")
        .accept_operation::<ReviewedSetPartDimensionBinding>(&required, &receipt)
        .expect("the exact owner-issued effect receipt must be accepted");
    let durable_receipt_identity = match completed {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "apply");
            assert!(!performed.replayed());
            *performed
                .operation_receipt_identity()
                .expect("the performed effect receipt identity must be durable")
        }
        other => panic!("expected the apply transition to complete, got {other:?}"),
    };
    let replayed = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&816_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("the operation acceptance replay must prepare")
        .accept_operation::<ReviewedSetPartDimensionBinding>(&required, &receipt)
        .expect("the exact operation acceptance must replay");
    match replayed {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "apply");
            assert!(performed.replayed());
            assert_eq!(
                performed.operation_receipt_identity(),
                Some(&durable_receipt_identity)
            );
        }
        other => panic!("expected the apply transition replay, got {other:?}"),
    }
    match advance_instance(&application, instance.clone(), 817)
        .expect("the completion terminal must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "applied")
        }
        other => panic!("expected the completion terminal, got {other:?}"),
    }
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            instance.branch(),
            SEED_DIMENSION,
            819,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION)
    );
    let stale = runtime
        .request(&principal, &scope)
        .on_branch(instance.branch())
        .mutate(ReviewedSetPartDimensionIntent {
            input: super::bounded_dimension_model::schema::SetPartDimensionInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .without_source()
        .idempotency(&820_u64)
        .for_workflow_operation(&application, &required)
        .expect("the descriptor still identifies the old operation")
        .execute_in_program(application.program_runtime());
    assert!(matches!(
        stale,
        Ok(WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift)
    ));
    assert_eq!(read_dimension(runtime, instance.branch()), SEED_DIMENSION);
}

#[test]
fn all_completed_join_does_not_turn_failing_evidence_into_approval_authority() {
    const FAILING_DIMENSION: u64 = 6;
    let application = super::bounded_dimension_model::host::publish_workflow_on_first_program();
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.program_runtime().current_world(),
            FAILING_DIMENSION,
            720,
        )),
        DimensionVerdict::Performed(FAILING_DIMENSION)
    );
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition_with_join_policy(
            "completed",
            worth_query_host::facade::declaration::application_program::ApplicationWorkflowEvidenceJoinPolicy::AllRequiredCompleted,
        ),
        WorkflowDefinitionExpectedPredecessor::Absent,
        721,
    )
    .expect("all-completed approval definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected a published definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), 722)
        .expect("all-completed instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected a started instance, got {other:?}"),
    };
    let proposal = proposal::published_proposal(&application, instance.clone(), 723);
    for (settlement_key, acceptance_key) in [(724, 725), (726, 727)] {
        let settlement = settle_assessment(&application, instance.clone(), settlement_key);
        assert_eq!(
            settlement.posture(),
            worth_query_host::facade::application_contribution::WorthQueryWorkflowAssessmentPosture::Failing
        );
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &settlement, acceptance_key),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    assert!(matches!(
        advance_instance(&application, instance.clone(), 728),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let required = match advance_instance(&application, instance.clone(), 729)
        .expect("all-completed join may expose the declared approval node")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected approval requirement, got {other:?}"),
    };
    assert!(matches!(
        approve_instance(
            &application,
            instance,
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
            730,
        ),
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt)
        )) if attempt.kind()
            == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch
    ));
}

#[test]
fn approval_rejects_assessment_evidence_after_native_source_aba() {
    let (application, _, instance, proposal, required, _) = approval_journey("approved", 700);
    for (dimension, key) in [(SEED_DIMENSION + 1, 710), (SEED_DIMENSION, 711)] {
        assert_eq!(
            settle(set_dimension(
                application.program_runtime(),
                instance.branch(),
                dimension,
                key,
            )),
            DimensionVerdict::Performed(dimension)
        );
    }
    assert!(matches!(
        approve_instance(
            &application,
            instance,
            &required,
            &proposal,
            WorkflowApprovalDecision::Approve,
            712,
        ),
        Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt)
        )) if attempt.kind()
            == WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch
    ));
}

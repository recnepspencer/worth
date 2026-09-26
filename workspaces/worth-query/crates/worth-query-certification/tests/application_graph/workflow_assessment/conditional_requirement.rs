use worth_query_host::facade::application_entry::{
    WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome, WorkflowTransitionPreparationDenial,
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryOrdinaryWorkflowRunStop, WorthQueryWorkflowAdvancePreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationAttemptDenialKind;

use super::super::bounded_dimension_model::{
    dimension_entry::{
        PartDimensionConditionQueryBinding, PartDimensionConditionRead, PART_IDENTITY,
        RELATED_PART_IDENTITY,
    },
    host::publish_workflow_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    presented_request::set_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        accept_assessment, accept_early_assessment, advance_instance, approve_instance,
        conditionally_required_related_assessment_definition, link_review_requirement,
        prepare_early_assessment_denial, propose_instance, publish_definition, run_instance,
        settle_assessment, settle_early_assessment_for, start_instance, WorkflowAdvanceInput,
        WorkflowAdvanceIntent,
    },
};

#[test]
fn native_relation_insertion_makes_an_authored_related_review_newly_required() {
    let application = publish_workflow_on_first_program();
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.program_runtime().current_world(),
            0,
            680,
        )),
        DimensionVerdict::Performed(0),
    );
    let definition = match publish_definition(
        &application,
        conditionally_required_related_assessment_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        681,
    )
    .expect("authored conditional definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("expected definition publication, got {other:?}"),
    };
    let instance = match start_instance(&application, definition, 682)
        .expect("conditional instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected instance start, got {other:?}"),
    };
    propose_instance(&application, instance.clone(), 683).expect("proposal must publish");

    let required_condition = match advance_instance(&application, instance.clone(), 684)
        .expect("condition must prepare")
    {
        WorkflowProgressOutcome::AwaitingCondition(required) => required,
        other => panic!("expected condition demand, got {other:?}"),
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
        .expect("typed condition query must execute");
    assert_eq!(condition_result.rows(), &[false]);
    let condition = runtime
        .request(&principal, &scope)
        .mutate(WorkflowAdvanceIntent {
            input: WorkflowAdvanceInput {
                part_identity: PART_IDENTITY.to_owned(),
            },
        })
        .without_source()
        .idempotency(&685_u64)
        .prepare_workflow_advance(&application, instance.clone())
        .expect("condition acceptance must prepare")
        .accept_condition::<PartDimensionConditionQueryBinding>(
            &required_condition,
            condition_result,
        )
        .expect("typed false condition must settle");
    assert!(matches!(condition, WorkflowProgressOutcome::Completed(_)));

    assert!(matches!(
        prepare_early_assessment_denial(&application, instance.clone(), "checks/related", 686),
        WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
            WorkflowTransitionPreparationDenial::Attempt(attempt)
        ) if attempt.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
    ));
    for (demand, acceptance) in [(687, 688), (689, 690)] {
        let assessment = settle_assessment(&application, instance.clone(), demand);
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &assessment, acceptance),
            Ok(WorkflowProgressOutcome::Completed(_)),
        ));
    }

    let before_link = application.runtime().workflow_instance_progress_counters();
    assert!(matches!(
        link_review_requirement(&application, 691).expect("native relation mutation must execute"),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    assert_eq!(
        application.runtime().workflow_instance_progress_counters(),
        before_link,
        "native publication does not contact waiting workflow progress",
    );
    let ordinary = run_instance(&application, instance.clone(), &[703]);
    assert_eq!(ordinary.attempted_steps(), 1);
    assert!(ordinary.transitions().is_empty());
    match ordinary.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingEvidence(
            required,
        )) => {
            assert_eq!(required.node_path(), "checks/join");
            assert_eq!(required.required_assessments(), 3);
            assert_eq!(required.completed_assessments(), 2);
        }
        other => panic!("ordinary run missed newly required review: {other:?}"),
    }
    let after_ordinary = application.runtime().workflow_instance_progress_counters();
    assert_eq!(after_ordinary.cold_misses(), before_link.cold_misses());
    assert!(after_ordinary.warm_hits() > before_link.warm_hits());
    match advance_instance(&application, instance.clone(), 692)
        .expect("join must re-evaluate independent authored inventory")
    {
        WorkflowProgressOutcome::AwaitingEvidence(required) => {
            assert_eq!(required.node_path(), "checks/join");
            assert_eq!(required.required_assessments(), 3);
            assert_eq!(required.completed_assessments(), 2);
        }
        other => panic!("old two-review coverage incorrectly completed join: {other:?}"),
    }
    let warm = application.runtime().workflow_instance_progress_counters();
    assert_eq!(warm.cold_misses(), after_ordinary.cold_misses());
    assert!(warm.warm_hits() > after_ordinary.warm_hits());
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    match advance_instance(&application, instance.clone(), 702)
        .expect("cold join must reconstruct the same required inventory")
    {
        WorkflowProgressOutcome::AwaitingEvidence(required) => {
            assert_eq!(required.required_assessments(), 3);
            assert_eq!(required.completed_assessments(), 2);
        }
        other => panic!("cold join lost the newly required review: {other:?}"),
    }
    let cold = application.runtime().workflow_instance_progress_counters();
    assert_eq!(cold.cold_misses(), warm.cold_misses() + 1);

    let related = settle_early_assessment_for(
        &application,
        instance.clone(),
        "checks/related",
        693,
        RELATED_PART_IDENTITY,
    );
    assert!(matches!(
        accept_early_assessment(
            &application,
            instance.clone(),
            "checks/related",
            &related,
            694
        ),
        Ok(WorkflowProgressOutcome::Completed(_)),
    ));
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.program_runtime().current_world(),
            8,
            695,
        )),
        DimensionVerdict::Performed(8),
    );
    match advance_instance(&application, instance.clone(), 696)
        .expect("resource edit must leave the unrelated Related review reusable")
    {
        WorkflowProgressOutcome::AwaitingEvidence(required) => {
            assert_eq!(required.required_assessments(), 3);
            assert_eq!(required.completed_assessments(), 1);
        }
        other => panic!("resource edit should only stale its two reviews: {other:?}"),
    }
    for (path, demand, acceptance) in [("checks/first", 697, 698), ("checks/second", 699, 700)] {
        let evidence = settle_early_assessment_for(
            &application,
            instance.clone(),
            path,
            demand,
            PART_IDENTITY,
        );
        assert!(matches!(
            accept_early_assessment(&application, instance.clone(), path, &evidence, acceptance),
            Ok(WorkflowProgressOutcome::Completed(_)),
        ));
    }
    match advance_instance(&application, instance.clone(), 701)
        .expect("complete applicable inventory must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join")
        }
        other => panic!("expected completed join after new review, got {other:?}"),
    }
}

#[test]
fn settled_join_cannot_authorize_approval_after_requirement_insertion() {
    let application = publish_workflow_on_first_program();
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.program_runtime().current_world(),
            0,
            710,
        )),
        DimensionVerdict::Performed(0),
    );
    let definition = match publish_definition(
        &application,
        conditionally_required_related_assessment_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        711,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => {
            performed.definition().clone()
        }
        other => panic!("expected definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition, 712).unwrap() {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected instance, got {other:?}"),
    };
    let proposal = match propose_instance(&application, instance.clone(), 713).unwrap() {
        WorkflowProposalOutcome::Published(performed) => performed.proposal().clone(),
        other => panic!("expected proposal, got {other:?}"),
    };
    let condition = match advance_instance(&application, instance.clone(), 714).unwrap() {
        WorkflowProgressOutcome::AwaitingCondition(required) => required,
        other => panic!("expected condition, got {other:?}"),
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
        .unwrap();
    assert_eq!(condition_result.rows(), &[false]);
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .mutate(WorkflowAdvanceIntent {
                input: WorkflowAdvanceInput {
                    part_identity: PART_IDENTITY.to_owned(),
                },
            })
            .without_source()
            .idempotency(&715_u64)
            .prepare_workflow_advance(&application, instance.clone())
            .unwrap()
            .accept_condition::<PartDimensionConditionQueryBinding>(&condition, condition_result)
            .unwrap(),
        WorkflowProgressOutcome::Completed(_),
    ));
    for (demand, acceptance) in [(716, 717), (718, 719)] {
        let evidence = settle_assessment(&application, instance.clone(), demand);
        assert!(matches!(
            accept_assessment(&application, instance.clone(), &evidence, acceptance),
            Ok(WorkflowProgressOutcome::Completed(_)),
        ));
    }
    assert!(matches!(
        advance_instance(&application, instance.clone(), 720),
        Ok(WorkflowProgressOutcome::Completed(_)),
    ));
    let required = match advance_instance(&application, instance.clone(), 721).unwrap() {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected approval demand, got {other:?}"),
    };
    assert!(matches!(
        link_review_requirement(&application, 722).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. },
    ));
    let warm_approval = approve_instance(
        &application,
        instance.clone(),
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        723,
    );
    assert!(
        matches!(
            &warm_approval,
            Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                WorkflowTransitionPreparationDenial::Attempt(attempt)
            )) if attempt.kind()
                == WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceIncomplete,
        ),
        "unexpected warm approval outcome: {warm_approval:?}"
    );
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let cold_approval = approve_instance(
        &application,
        instance,
        &required,
        &proposal,
        WorkflowApprovalDecision::Approve,
        724,
    );
    assert!(
        matches!(
            &cold_approval,
            Err(WorthQueryWorkflowAdvancePreparationDenial::TransitionPreparation(
                WorkflowTransitionPreparationDenial::Attempt(attempt)
            )) if attempt.kind()
                == WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceIncomplete,
        ),
        "unexpected cold approval outcome: {cold_approval:?}"
    );
}

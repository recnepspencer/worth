//! Certification of workflow assessment-head reconstruction without settlement.

use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowTransitionPreparationDenial,
    WorthQueryApplicationOutputDemandDenial, WorthQueryWorkflowAdvancePreparationDenial,
    WorthQueryWorkflowAssessmentAcceptanceDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationOutputRole,
    WorthQueryOutputDemandDenialKind, WorthQueryPreserveOutput,
};
use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;

use super::bounded_dimension_model::{
    assessment_output::PartAssessmentBinding,
    dimension_entry::{PART_IDENTITY, RELATED_PART_IDENTITY},
    host::{publish_workflow_on_first_program, SEED_DIMENSION},
    presented_request::set_dimension,
    schema::Part,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        accept_assessment, advance_instance, assessment_join_terminal_definition,
        assessment_join_terminal_definition_with_policy, assessment_retry_definition,
        multi_subject_assessment_retry_definition, propose_instance, publish_definition,
        reviewed_geometry_definition, settle_assessment, settle_assessment_for,
        spoofed_assessment_denial, start_instance,
    },
};

#[path = "workflow_assessment/currentness.rs"]
mod currentness;
#[path = "workflow_assessment/join_policy.rs"]
mod join_policy;
#[path = "workflow_assessment/multi_subject_currentness.rs"]
mod multi_subject_currentness;

#[test]
fn authenticated_advance_reconstructs_the_exact_required_assessment_without_settling_it() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        reviewed_geometry_definition("completed"),
        WorkflowDefinitionExpectedPredecessor::Absent,
        441,
    )
    .expect("assessment definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected a published assessment definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 442)
        .expect("assessment instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected a started assessment instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 443)
        .expect("assessment proposal preparation must succeed");
    assert!(matches!(
        spoofed_assessment_denial(&application, started.instance().clone(), 4431),
        WorthQueryApplicationOutputDemandDenial::Demand(denial)
            if denial.kind() == WorthQueryOutputDemandDenialKind::MissingApplicableProducer
    ));

    for key in [444, 445] {
        let required = match advance_instance(&application, started.instance().clone(), key)
            .expect("assessment-head advance must prepare under fresh authority")
        {
            WorkflowProgressOutcome::AwaitingAssessment(required) => required,
            other => panic!("expected an assessment requirement, got {other:?}"),
        };
        assert_eq!(required.instance(), started.instance().entity_id());
        assert_eq!(required.node_path(), "checks/structural");
        assert_eq!(required.query(), "part_dimension_query");
        assert_eq!(
            required.binding(),
            "worth.query.certification.bounded-dimension.read-binding.v1"
        );
        assert_eq!(
            required.result_type(),
            "worth.query.certification.bounded-dimension.row.v1"
        );
    }

    let settled = settle_assessment(&application, started.instance().clone(), 446);
    assert_eq!(settled.required().node_path(), "checks/structural");
    assert_eq!(settled.required().query(), "part_dimension_query");
    assert_eq!(
        settled
            .settlement()
            .application_commit_receipt()
            .expect("fresh assessment has a commit receipt")
            .product_branch(),
        started.instance().branch()
    );

    match advance_instance(&application, started.instance().clone(), 447)
        .expect("performed assessment output alone must not advance workflow evidence")
    {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "checks/structural");
        }
        other => panic!("output publication must not settle workflow evidence: {other:?}"),
    }

    let accepted = accept_assessment(&application, started.instance().clone(), &settled, 448)
        .expect("the exact retained output must publish assessment evidence");
    match accepted {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/structural");
            assert!(!performed.terminal());
            assert!(!performed.replayed());
            let evidence = performed
                .assessment_evidence()
                .expect("assessment acceptance must project its committed evidence fact");
            assert_eq!(
                evidence.producer(),
                "worth.query.certification.part-assessment.producer.v1"
            );
            assert_eq!(
                evidence.family(),
                "worth.query.certification.part-assessment.output.v1"
            );
            assert_eq!(evidence.query(), "part_dimension_query");
            assert_eq!(evidence.parameter_type(), "PartQueryParameters");
            assert_eq!(
                evidence.result_type(),
                "worth.query.certification.bounded-dimension.row.v1"
            );
            assert_eq!(
                evidence.binding(),
                "worth.query.certification.bounded-dimension.read-binding.v1"
            );
            let output = settled
                .settlement()
                .application_commit_receipt()
                .expect("fresh assessment has a commit receipt")
                .output_correspondence()
                .entity(WorthQueryApplicationOutputRole::<
                    PartAssessmentBinding,
                    Part,
                    WorthQueryPreserveOutput,
                >::from_static("assessment"))
                .expect("the assessment output role must project from its performed receipt");
            assert_eq!(evidence.subject(), output.entity_id());
            assert!(evidence.passing());
            assert_eq!(evidence.identity().len(), 64);
            assert_eq!(evidence.source_identity().len(), 64);
            assert!(!evidence.publication_identity().is_empty());
            assert_eq!(evidence.output_content_identity().len(), 64);
        }
        other => panic!("expected performed assessment evidence, got {other:?}"),
    }

    let before_assessment = application.runtime().workflow_instance_progress_counters();
    match advance_instance(&application, started.instance().clone(), 449)
        .expect("accepted assessment must advance to the next exact requirement")
    {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), "checks/manufacturability");
            assert_eq!(required.occurrence(), 2);
        }
        other => panic!("expected the second assessment requirement, got {other:?}"),
    }
    let after_assessment = application.runtime().workflow_instance_progress_counters();
    assert_eq!(
        after_assessment.warm_core_hits(),
        before_assessment.warm_core_hits() + 1,
    );
    assert_eq!(
        after_assessment.warm_history_transition_visits(),
        before_assessment.warm_history_transition_visits(),
    );

    match accept_assessment(&application, started.instance().clone(), &settled, 448)
        .expect("the original acceptance key must replay its exact evidence")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/structural");
            assert!(performed.replayed());
            assert!(!performed.terminal());
            assert!(performed.assessment_evidence().is_some());
        }
        other => panic!("expected replayed assessment evidence, got {other:?}"),
    }

    assert!(matches!(
        accept_assessment(&application, started.instance().clone(), &settled, 450,),
        Err(WorthQueryWorkflowAssessmentAcceptanceDenial::RequirementMismatch)
    ));

    let second_settlement = settle_assessment(&application, started.instance().clone(), 451);
    assert!(matches!(
        accept_assessment(
            &application,
            started.instance().clone(),
            &second_settlement,
            452,
        ),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match accept_assessment(
        &application,
        started.instance().clone(),
        &second_settlement,
        452,
    )
    .expect("an exact assessment retry must resolve through a non-assessment successor head")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/manufacturability");
            assert!(performed.replayed());
        }
        other => panic!("expected replay through successor head, got {other:?}"),
    }

    let before_join = application.runtime().workflow_instance_progress_counters();
    match advance_instance(&application, started.instance().clone(), 453)
        .expect("the evidence join must reconstruct from committed assessment facts")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/evidence");
            assert!(!performed.replayed());
            assert!(performed.assessment_evidence().is_none());
        }
        other => panic!("expected the complete evidence join to settle, got {other:?}"),
    }
    let after_join = application.runtime().workflow_instance_progress_counters();
    assert_eq!(
        after_join.warm_core_hits(),
        before_join.warm_core_hits() + 1,
    );
    assert_eq!(
        after_join.warm_history_transition_visits(),
        before_join.warm_history_transition_visits(),
    );
    match advance_instance(&application, started.instance().clone(), 453)
        .expect("the committed evidence join key must replay beyond its successor head")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/evidence");
            assert!(performed.replayed());
        }
        other => panic!("expected the evidence join replay, got {other:?}"),
    }
    let before_fresh_head = application.runtime().workflow_instance_progress_counters();
    let fresh_head_cost = application
        .runtime()
        .capture_certification_cost_scope(started.instance().branch())
        .expect("the workflow branch admits cost observation");
    match advance_instance(&application, started.instance().clone(), 454)
        .expect("the successor must remain reconstructible after the join settles")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => {
            assert_eq!(required.instance(), started.instance().entity_id());
            assert_eq!(required.node_path(), "approval");
            assert_eq!(required.operation(), "WorkflowAdvanceOperation");
            assert_eq!(required.target_operation(), "SetPartDimension");
            assert_eq!(required.installed_capability_identity().len(), 64);
        }
        other => panic!("expected the exact approval requirement, got {other:?}"),
    }
    let fresh_head_cost = application
        .runtime()
        .observe_certification_cost(&fresh_head_cost)
        .expect("the workflow branch remains available for cost observation");
    assert_eq!(
        fresh_head_cost.application_work().retained_resolutions(),
        1,
        "a fresh current-head key probes once instead of resolving three historical transitions",
    );
    let after_fresh_head = application.runtime().workflow_instance_progress_counters();
    assert_eq!(
        after_fresh_head.warm_core_hits(),
        before_fresh_head.warm_core_hits() + 1,
        "the exact retained revision must serve the current-head selection",
    );
    assert_eq!(
        after_fresh_head.warm_history_transition_visits(),
        before_fresh_head.warm_history_transition_visits(),
        "the warm current-head selection must not enumerate its three-transition prefix",
    );

    let foreign_definition = match publish_definition(
        &application,
        reviewed_geometry_definition("separate-instance"),
        WorkflowDefinitionExpectedPredecessor::Published(definition.definition().clone()),
        460,
    )
    .expect("second workflow definition publication must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected a second workflow definition, got {other:?}"),
    };
    let foreign = match start_instance(&application, foreign_definition.definition().clone(), 461)
        .expect("second workflow instance start must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected a second workflow instance, got {other:?}"),
    };
    propose_instance(&application, foreign.instance().clone(), 462)
        .expect("second workflow proposal must prepare");
    assert!(matches!(
        accept_assessment(&application, foreign.instance().clone(), &settled, 448),
        Err(WorthQueryWorkflowAssessmentAcceptanceDenial::RequirementMismatch)
    ));
}

#[test]
fn evidence_join_replays_before_a_supported_terminal_successor() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        assessment_join_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        470,
    )
    .expect("join-terminal definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        other => panic!("expected published definition, got {other:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 471)
        .expect("join-terminal instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected started instance, got {other:?}"),
    };
    propose_instance(&application, started.instance().clone(), 472).expect("proposal must settle");
    for (settlement_key, acceptance_key) in [(473, 474), (475, 476)] {
        let settled = settle_assessment(&application, started.instance().clone(), settlement_key);
        assert!(matches!(
            accept_assessment(
                &application,
                started.instance().clone(),
                &settled,
                acceptance_key
            ),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let before_reconstruction = application.runtime().workflow_instance_progress_counters();
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 477),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    let after_reconstruction = application.runtime().workflow_instance_progress_counters();
    assert_eq!(
        after_reconstruction.cold_misses(),
        before_reconstruction.cold_misses() + 1,
    );
    assert!(
        after_reconstruction.cold_reconstruction_transition_visits()
            > before_reconstruction.cold_reconstruction_transition_visits()
    );
    match advance_instance(&application, started.instance().clone(), 477)
        .expect("join key must replay before its supported successor")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join");
            assert!(performed.replayed());
        }
        other => panic!("expected replayed join, got {other:?}"),
    }
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 478),
        Ok(WorkflowProgressOutcome::Completed(_))
    ));
    match advance_instance(&application, started.instance().clone(), 477)
        .expect("join key must remain replayable after terminal completion")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "checks/join");
            assert!(performed.replayed());
            assert!(!performed.terminal());
        }
        other => panic!("expected the historical join replay, got {other:?}"),
    }
}

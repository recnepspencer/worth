//! Selective related-subject evidence survives the ordinary control path.

use worth_query_host::facade::{
    application_contribution::WorthQueryWorkflowAssessmentPosture,
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
        WorthQueryOrdinaryWorkflowRunStop,
    },
};

use super::bounded_dimension_model::{
    dimension_entry::{PART_IDENTITY, RELATED_PART_IDENTITY},
    host::{publish_workflow_on_first_program, SEED_DIMENSION},
    presented_request::set_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        accept_assessment, advance_instance, multi_subject_assessment_retry_definition,
        propose_instance, publish_definition, run_instance, settle_assessment,
        settle_assessment_for, start_instance,
    },
};

#[test]
fn ordinary_and_advanced_preserve_only_the_unedited_subjects_review() {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        multi_subject_assessment_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        919_300,
    )
    .expect("multi-subject definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("multi-subject definition did not publish: {other:?}"),
    };
    let start = |key| match start_instance(&application, published.definition().clone(), key)
        .expect("multi-subject instance starts")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("multi-subject instance did not start: {other:?}"),
    };
    let advanced = start(919_301);
    let ordinary = start(919_302);
    for (instance, key) in [(advanced.clone(), 919_303), (ordinary.clone(), 919_304)] {
        assert!(matches!(
            propose_instance(&application, instance, key),
            Ok(WorkflowProposalOutcome::Published(_))
        ));
    }
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.current_world(),
            6,
            919_305,
        )),
        DimensionVerdict::Performed(6)
    );
    for (instance, base) in [
        (advanced.clone(), 919_310_u64),
        (ordinary.clone(), 919_320_u64),
    ] {
        for (subject, posture, settle_key, accept_key) in [
            (
                PART_IDENTITY,
                WorthQueryWorkflowAssessmentPosture::Failing,
                base,
                base + 1,
            ),
            (
                RELATED_PART_IDENTITY,
                WorthQueryWorkflowAssessmentPosture::Passing,
                base + 2,
                base + 3,
            ),
        ] {
            let settled =
                settle_assessment_for(&application, instance.clone(), settle_key, subject);
            assert_eq!(settled.posture(), posture);
            assert!(matches!(
                accept_assessment(&application, instance.clone(), &settled, accept_key),
                Ok(WorkflowProgressOutcome::Completed(_))
            ));
        }
        assert!(matches!(
            advance_instance(&application, instance, base + 4),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    assert_eq!(
        settle(set_dimension(
            application.program_runtime(),
            application.current_world(),
            SEED_DIMENSION,
            919_330,
        )),
        DimensionVerdict::Performed(SEED_DIMENSION)
    );
    let advanced_required = match advance_instance(&application, advanced.clone(), 919_331)
        .expect("advanced changed-subject wait prepares")
    {
        WorkflowProgressOutcome::AwaitingAssessment(required) => required,
        other => panic!("advanced path reused changed resource evidence: {other:?}"),
    };
    let ordinary_wait = run_instance(&application, ordinary.clone(), &[919_332]);
    assert_eq!(ordinary_wait.attempted_steps(), 1);
    assert!(ordinary_wait.transitions().is_empty());
    let ordinary_required = match ordinary_wait.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(
            WorkflowProgressOutcome::AwaitingAssessment(required),
        ) => required,
        other => panic!("ordinary path reused changed resource evidence: {other:?}"),
    };
    assert_eq!(advanced_required.node_path(), "checks/first");
    assert_eq!(ordinary_required.node_path(), advanced_required.node_path());
    // Instance and proposal authority differ; coverage names the same reviewed subject.
    assert_ne!(ordinary_required.instance(), advanced_required.instance());
    assert_ne!(
        ordinary_required.transition_identity(),
        advanced_required.transition_identity()
    );
    assert_ne!(
        ordinary_required.proposal_identity(),
        advanced_required.proposal_identity()
    );
    assert_eq!(
        ordinary_required.coverage_identity(),
        advanced_required.coverage_identity()
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
    for (instance, base) in [
        (advanced.clone(), 919_340_u64),
        (ordinary.clone(), 919_350_u64),
    ] {
        let replacement = settle_assessment(&application, instance.clone(), base);
        assert_eq!(
            replacement.posture(),
            WorthQueryWorkflowAssessmentPosture::Passing
        );
        assert!(matches!(
            accept_assessment(&application, instance, &replacement, base + 1),
            Ok(WorkflowProgressOutcome::Completed(_))
        ));
    }
    let WorkflowProgressOutcome::Completed(advanced_related) =
        advance_instance(&application, advanced.clone(), 919_360)
            .expect("advanced related-subject reuse prepares")
    else {
        panic!("advanced related-subject evidence was not reused");
    };
    let ordinary_related = run_instance(&application, ordinary.clone(), &[919_361]);
    assert!(matches!(
        ordinary_related.stop(),
        WorthQueryOrdinaryWorkflowRunStop::CallerKeysExhausted
    ));
    assert_eq!(ordinary_related.attempted_steps(), 1);
    assert_eq!(ordinary_related.transitions().len(), 1);
    assert_eq!(advanced_related.node_path(), "checks/second");
    assert_eq!(
        ordinary_related.transitions()[0].node_path(),
        advanced_related.node_path()
    );
    assert!(advanced_related.assessment_evidence().is_none());
    assert!(ordinary_related.transitions()[0]
        .assessment_evidence()
        .is_none());
    let WorkflowProgressOutcome::Completed(advanced_join) =
        advance_instance(&application, advanced.clone(), 919_362)
            .expect("advanced passing join prepares")
    else {
        panic!("advanced join did not settle");
    };
    let WorkflowProgressOutcome::Completed(advanced_terminal) =
        advance_instance(&application, advanced, 919_363).expect("advanced completion prepares")
    else {
        panic!("advanced terminal did not settle");
    };
    let ordinary_completion = run_instance(&application, ordinary, &[919_364, 919_365]);
    assert!(matches!(
        ordinary_completion.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert_eq!(ordinary_completion.transitions().len(), 2);
    assert_eq!(
        ordinary_completion.transitions()[0].node_path(),
        advanced_join.node_path()
    );
    assert_eq!(
        ordinary_completion.transitions()[1].node_path(),
        advanced_terminal.node_path()
    );
    assert_eq!(advanced_terminal.node_path(), "completed");
}

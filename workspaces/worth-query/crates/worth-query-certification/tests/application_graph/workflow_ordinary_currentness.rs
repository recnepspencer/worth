//! The ordinary workflow entry preserves native evidence currentness across ABA.

use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
    WorthQueryOrdinaryWorkflowRunStop,
};

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::{publish_workflow_on_first_program, SEED_DIMENSION},
    presented_request::set_dimension,
    settled_verdict::{settle, DimensionVerdict},
    workflow::{
        accept_assessment, accept_early_assessment, advance_instance,
        assessment_join_terminal_definition, propose_instance, publish_definition, run_instance,
        settle_assessment, settle_early_assessment_for, start_instance,
    },
};

#[test]
fn ordinary_and_advanced_require_fresh_evidence_after_native_aba() {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        assessment_join_terminal_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        919_400,
    )
    .expect("ABA definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("ABA definition did not publish: {other:?}"),
    };
    let start = |key| match start_instance(&application, published.definition().clone(), key)
        .expect("ABA instance starts")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("ABA instance did not start: {other:?}"),
    };
    let advanced = start(919_401);
    let ordinary = start(919_402);
    for (instance, base) in [
        (advanced.clone(), 919_410_u64),
        (ordinary.clone(), 919_420_u64),
    ] {
        assert!(matches!(
            propose_instance(&application, instance.clone(), base),
            Ok(WorkflowProposalOutcome::Published(_))
        ));
        for (settle_key, accept_key) in [(base + 1, base + 2), (base + 3, base + 4)] {
            let assessment = settle_assessment(&application, instance.clone(), settle_key);
            assert!(matches!(
                accept_assessment(&application, instance.clone(), &assessment, accept_key),
                Ok(WorkflowProgressOutcome::Completed(_))
            ));
        }
    }
    for (dimension, key) in [(SEED_DIMENSION + 1, 919_430), (SEED_DIMENSION, 919_431)] {
        assert_eq!(
            settle(set_dimension(
                application.program_runtime(),
                application.current_world(),
                dimension,
                key,
            )),
            DimensionVerdict::Performed(dimension)
        );
    }

    let advanced_required = match advance_instance(&application, advanced.clone(), 919_432)
        .expect("advanced ABA wait prepares")
    {
        WorkflowProgressOutcome::AwaitingEvidence(required) => required,
        other => panic!("advanced path reused pre-ABA evidence: {other:?}"),
    };
    let ordinary_wait = run_instance(&application, ordinary.clone(), &[919_433]);
    assert_eq!(ordinary_wait.attempted_steps(), 1);
    assert!(ordinary_wait.transitions().is_empty());
    let ordinary_required = match ordinary_wait.stop() {
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingEvidence(
            required,
        )) => required,
        other => panic!("ordinary path reused pre-ABA evidence: {other:?}"),
    };
    assert_eq!(advanced_required.node_path(), "checks/join");
    assert_eq!(ordinary_required.node_path(), advanced_required.node_path());
    assert_eq!(ordinary_required.required_assessments(), 2);
    assert_eq!(ordinary_required.completed_assessments(), 0);
    assert_eq!(advanced_required.required_assessments(), 2);
    assert_eq!(advanced_required.completed_assessments(), 0);
    assert_ne!(ordinary_required.instance(), advanced_required.instance());

    for (instance, base) in [
        (advanced.clone(), 919_440_u64),
        (ordinary.clone(), 919_450_u64),
    ] {
        for (path, settle_key, accept_key) in [
            ("checks/first", base, base + 1),
            ("checks/second", base + 2, base + 3),
        ] {
            let replacement = settle_early_assessment_for(
                &application,
                instance.clone(),
                path,
                settle_key,
                PART_IDENTITY,
            );
            assert!(matches!(
                accept_early_assessment(
                    &application,
                    instance.clone(),
                    path,
                    &replacement,
                    accept_key,
                ),
                Ok(WorkflowProgressOutcome::Completed(_))
            ));
        }
    }
    for (key, path) in [(919_460, "checks/join"), (919_461, "completed")] {
        let WorkflowProgressOutcome::Completed(performed) =
            advance_instance(&application, advanced.clone(), key)
                .expect("advanced recovery transition prepares")
        else {
            panic!("advanced recovery did not complete {path}");
        };
        assert_eq!(performed.node_path(), path);
    }
    let ordinary_completion = run_instance(&application, ordinary, &[919_462, 919_463]);
    assert!(matches!(
        ordinary_completion.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert_eq!(ordinary_completion.transitions().len(), 2);
    assert_eq!(
        ordinary_completion.transitions()[0].node_path(),
        "checks/join"
    );
    assert_eq!(
        ordinary_completion.transitions()[1].node_path(),
        "completed"
    );
}

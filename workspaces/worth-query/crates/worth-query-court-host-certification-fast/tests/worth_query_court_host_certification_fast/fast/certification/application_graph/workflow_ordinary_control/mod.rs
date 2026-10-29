pub(crate) use worth_query_worlds::facade::certification::application_graph::workflow_ordinary_control::*;

#[test]
fn ordinary_and_advanced_satisfied_condition_share_meaning_wait_and_terminal() {
    assert_condition_path_parity(true);
}

#[test]
fn ordinary_and_advanced_unsatisfied_condition_share_meaning_wait_and_terminal() {
    assert_condition_path_parity(false);
}

#[test]
fn ordinary_and_advanced_retry_share_typed_denials_and_exhaustion() {
    let application = publish_workflow_on_first_program();
    let published = match publish_definition(
        &application,
        bounded_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        919_100,
    )
    .expect("bounded retry publication prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("bounded retry did not publish: {other:?}"),
    };
    let start = |key| match start_instance(&application, published.definition().clone(), key)
        .expect("retry instance starts")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("retry instance did not start: {other:?}"),
    };
    let advanced = start(919_101);
    let ordinary = start(919_102);
    let expected_paths = [
        "proposal/first",
        "proposal/revise",
        "proposal/first",
        "proposal/revise",
        "proposal/first",
        "proposal/revise",
    ];
    for (index, expected) in expected_paths.iter().enumerate() {
        let advanced_outcome =
            advance_instance(&application, advanced.clone(), 919_120 + index as u64)
                .expect("advanced retry advance prepares");
        let ordinary_outcome =
            run_instance(&application, ordinary.clone(), &[919_130 + index as u64]);
        assert_eq!(ordinary_outcome.attempted_steps(), 1);
        assert!(ordinary_outcome.transitions().is_empty());
        let WorkflowProgressOutcome::AwaitingProposal(advanced_need) = advanced_outcome else {
            panic!("advanced retry must identify its proposal: {advanced_outcome:?}");
        };
        let WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingProposal(
            ordinary_need,
        )) = ordinary_outcome.stop()
        else {
            panic!(
                "ordinary retry must identify its proposal: {:?}",
                ordinary_outcome.stop()
            );
        };
        assert_eq!(advanced_need.instance(), advanced.entity_id());
        assert_eq!(ordinary_need.instance(), ordinary.entity_id());
        for need in [&advanced_need, ordinary_need] {
            assert_eq!(need.node_path(), *expected);
            assert_eq!(need.operation(), "WorkflowDefinitionAuthoringOperation");
            assert_eq!(
                need.input_type(),
                "worth.query.certification.document-retention.set-input.v1"
            );
            assert_eq!(need.occurrence(), index as u64);
        }
        for (instance, key) in [
            (advanced.clone(), 919_140 + index as u64),
            (ordinary.clone(), 919_150 + index as u64),
        ] {
            let WorkflowProposalOutcome::Published(performed) =
                propose_authoring_instance(&application, instance, key)
                    .expect("required retry proposal prepares")
            else {
                panic!("required retry proposal did not publish");
            };
            assert_eq!(performed.node_path(), *expected);
            assert!(!performed.replayed());
        }
    }
    let WorkflowProgressOutcome::Completed(advanced_terminal) =
        advance_instance(&application, advanced, 919_160)
            .expect("advanced retry exhaustion prepares")
    else {
        panic!("advanced retry did not exhaust");
    };
    let ordinary_terminal = run_instance(&application, ordinary, &[919_161]);
    assert!(matches!(
        ordinary_terminal.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
    ));
    assert_eq!(ordinary_terminal.transitions().len(), 1);
    assert_eq!(
        ordinary_terminal.transitions()[0].node_path(),
        advanced_terminal.node_path()
    );
    assert_eq!(advanced_terminal.node_path(), "retry-exhausted");
}

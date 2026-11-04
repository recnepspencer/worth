use super::recorded_workflow_receipt as recorded;
pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::workflow_retry::*;

#[test]
fn bounded_retry_spends_durable_attempts_and_duplicate_replay_spends_none() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        bounded_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        800,
    )
    .expect("bounded-retry definition must prepare")
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        other => panic!("expected published retry definition, got {other:?}"),
    };
    let instance = match start_instance(&application, definition.definition().clone(), 801)
        .expect("bounded-retry instance must prepare")
    {
        WorkflowInstanceStartOutcome::Started(performed) => performed.instance().clone(),
        other => panic!("expected started retry instance, got {other:?}"),
    };

    let mut paths = Vec::new();
    let mut originals = Vec::new();
    for key in 802..808 {
        match propose_authoring_instance(&application, instance.clone(), key)
            .expect("the current retry proposal must prepare")
        {
            WorkflowProposalOutcome::Published(performed) => {
                paths.push(performed.node_path().to_owned());
                assert!(performed.receipt().changed_record_count() > 0);
                originals.push(performed);
            }
            other => panic!("expected a published retry proposal, got {other:?}"),
        }
        if key == 803 {
            match propose_authoring_instance(&application, instance.clone(), key)
                .expect("the duplicate revision key must replay")
            {
                WorkflowProposalOutcome::RecoveredCommitted(receipt) => {
                    let original = originals.last().unwrap();
                    assert_eq!(original.node_path(), "proposal/revise");
                    recorded::original(&receipt, original.receipt());
                    recorded::creates(&receipt, original.proposal().entity_id());
                    recorded::creates(&receipt, original.transition());
                }
                other => panic!("expected replayed revision, got {other:?}"),
            }
        }
    }
    assert_eq!(
        paths,
        [
            "proposal/first",
            "proposal/revise",
            "proposal/first",
            "proposal/revise",
            "proposal/first",
            "proposal/revise",
        ]
    );
    match advance_instance(&application, instance, 808)
        .expect("the exhausted successor must prepare")
    {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), "retry-exhausted")
        }
        other => panic!("expected retry exhaustion terminal, got {other:?}"),
    }
}

use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProgressOutcome, WorkflowProposalOutcome,
};

use super::bounded_dimension_model::{
    host::publish_workflow_on_first_program,
    workflow::{
        advance_instance, bounded_retry_definition, propose_authoring_instance, publish_definition,
        start_instance,
    },
};

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
    for key in 802..808 {
        match propose_authoring_instance(&application, instance.clone(), key)
            .expect("the current retry proposal must prepare")
        {
            WorkflowProposalOutcome::Published(performed) => {
                paths.push(performed.node_path().to_owned());
                assert!(!performed.replayed());
            }
            other => panic!("expected a published retry proposal, got {other:?}"),
        }
        if key == 803 {
            match propose_authoring_instance(&application, instance.clone(), key)
                .expect("the duplicate revision key must replay")
            {
                WorkflowProposalOutcome::Published(performed) => {
                    assert_eq!(performed.node_path(), "proposal/revise");
                    assert!(performed.replayed());
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

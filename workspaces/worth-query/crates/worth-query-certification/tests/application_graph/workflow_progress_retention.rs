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
fn progress_reuses_exact_revisions_and_rebuilds_after_release() {
    let application = publish_workflow_on_first_program();
    let definition = match publish_definition(
        &application,
        bounded_retry_definition(),
        WorkflowDefinitionExpectedPredecessor::Absent,
        1_211,
    )
    .expect("the terminal definition prepares")
    {
        WorkflowDefinitionPublicationOutcome::Published(publication) => publication,
        unexpected => panic!("expected a published definition, got {unexpected:?}"),
    };
    let started = match start_instance(&application, definition.definition().clone(), 1_212)
        .expect("the terminal instance prepares")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        unexpected => panic!("expected a started instance, got {unexpected:?}"),
    };

    let before = application.runtime().workflow_instance_progress_counters();
    let first = propose_authoring_instance(&application, started.instance().clone(), 1_213)
        .expect("the first proposal prepares");
    let WorkflowProposalOutcome::Published(first) = first else {
        panic!("expected the first proposal to publish");
    };
    assert_eq!(first.node_path(), "proposal/first");
    let cold = application.runtime().workflow_instance_progress_counters();
    assert_eq!(cold.cold_misses(), before.cold_misses() + 1);
    assert_eq!(cold.cold_retains(), before.cold_retains() + 1);

    let second = propose_authoring_instance(&application, started.instance().clone(), 1_214)
        .expect("the retry proposal prepares");
    let WorkflowProposalOutcome::Published(second) = second else {
        panic!("expected the retry proposal to publish");
    };
    assert_eq!(second.node_path(), "proposal/revise");
    let changed_revision = application.runtime().workflow_instance_progress_counters();
    assert_eq!(changed_revision.cold_misses(), cold.cold_misses() + 1);
    assert_eq!(changed_revision.cold_retains(), cold.cold_retains() + 1);
    assert_eq!(
        changed_revision.cold_reconstruction_transition_visits(),
        cold.cold_reconstruction_transition_visits() + 1
    );

    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 1_215),
        Ok(WorkflowProgressOutcome::PreparationDenied(_))
    ));
    let reconstructed = application.runtime().workflow_instance_progress_counters();
    assert_eq!(
        reconstructed.cold_misses(),
        changed_revision.cold_misses() + 1
    );
    assert_eq!(
        reconstructed.cold_reconstruction_transition_visits(),
        changed_revision.cold_reconstruction_transition_visits() + 2
    );

    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 1_216),
        Ok(WorkflowProgressOutcome::PreparationDenied(_))
    ));
    let warm = application.runtime().workflow_instance_progress_counters();
    assert_eq!(warm.warm_hits(), reconstructed.warm_hits() + 1);

    application
        .runtime()
        .release_workflow_instance_progress_for_test();
    let released = application.runtime().workflow_instance_progress_counters();
    assert_eq!(released.retained_charge_bytes(), 0);
    assert!(released.releases() > warm.releases());
    assert!(matches!(
        advance_instance(&application, started.instance().clone(), 1_217),
        Ok(WorkflowProgressOutcome::PreparationDenied(_))
    ));
    let rebuilt = application.runtime().workflow_instance_progress_counters();
    assert_eq!(rebuilt.cold_misses(), released.cold_misses() + 1);
    assert_eq!(rebuilt.cold_retains(), released.cold_retains() + 1);
    assert_eq!(
        rebuilt.cold_reconstruction_transition_visits(),
        released.cold_reconstruction_transition_visits() + 2
    );
}

use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceStartOutcome, WorkflowProposalOutcome,
};
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowComponentLimits;
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;

use super::bounded_dimension_model::{
    host::publish_on_first_program,
    workflow::{
        bounded_retry_definition_with_attempts, propose_authoring_instance, publish_definition,
        retain_workflow_with_resources, start_instance,
    },
};

#[test]
fn historical_workflow_receipts_do_not_retain_live_world_observations() {
    // The production host admits 128 simultaneous observations, not 200.
    // Keep every historical receipt alive while issuing sequential effects.
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        32,
        64,
        4,
        ApplicationWorkflowComponentLimits::new(32, 4, 128, 256, 256).unwrap(),
        64 * 1024,
        32,
        256,
        256 * 1024,
    )
    .unwrap();
    let application = retain_workflow_with_resources(publish_on_first_program(), resources);
    let definition = match publish_definition(
        &application,
        bounded_retry_definition_with_attempts(256),
        WorkflowDefinitionExpectedPredecessor::Absent,
        20_000,
    )
    .unwrap()
    {
        WorkflowDefinitionPublicationOutcome::Published(performed) => performed,
        _ => panic!("lifecycle definition must publish"),
    };
    let instance =
        match start_instance(&application, definition.definition().clone(), 20_001).unwrap() {
            WorkflowInstanceStartOutcome::Started(performed) => performed,
            _ => panic!("lifecycle instance must start"),
        };
    let mut receipts = Vec::new();
    for occurrence in 0..200 {
        let outcome = propose_authoring_instance(
            &application,
            instance.instance().clone(),
            20_002 + occurrence,
        )
        .unwrap_or_else(|denial| panic!("occurrence {occurrence}: {denial:?}"));
        let WorkflowProposalOutcome::Published(performed) = outcome else {
            panic!("occurrence {occurrence} must publish");
        };
        assert!(!performed.replayed());
        receipts.push(performed);
    }
    let before = application.runtime().workflow_instance_progress_counters();
    for occurrence in [0, 199] {
        let outcome = propose_authoring_instance(
            &application,
            instance.instance().clone(),
            20_002 + occurrence,
        )
        .expect("historical retry retains no live admission and must reauthorize");
        let WorkflowProposalOutcome::Published(replayed) = outcome else {
            panic!("historical receipt must replay");
        };
        assert!(replayed.replayed());
        assert_eq!(
            replayed.transition(),
            receipts[occurrence as usize].transition()
        );
        assert_eq!(
            replayed.proposal(),
            receipts[occurrence as usize].proposal()
        );
    }
    assert_eq!(
        application
            .runtime()
            .workflow_instance_progress_counters()
            .incremental_advances(),
        before.incremental_advances()
    );
    assert_eq!(receipts.len(), 200);
}

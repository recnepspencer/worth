//! Discovery preserves the actual lookup's request stop at the public door.
use super::super::super::tests::fixture::{installed_authorization_world, IdentityExecutionSchema};
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
struct Workflow;
impl ApplicationWorkflowSpec for Workflow {
    fn observation_policy(
    ) -> worth_query_declaration::facade::application_program::ApplicationWorkflowObservationPolicy
    {
        worth_query_declaration::facade::application_program::ApplicationWorkflowObservationPolicy::disabled()
    }
    fn actor_policy(
    ) -> worth_query_declaration::facade::application_program::ApplicationWorkflowActorPolicy {
        worth_query_declaration::facade::application_program::ApplicationWorkflowActorPolicy::not_applicable()
    }

    type Schema = IdentityExecutionSchema;
    const IDENTITY:
        worth_query_declaration::facade::application_program::ApplicationWorkflowSpecIdentity =
        worth_query_declaration::facade::application_program::ApplicationWorkflowSpecIdentity::new(
            "stop-court",
        );
}
#[test]
fn workflow_actor_contract_disabled_discovery_preserves_request_stops() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .unwrap();
    let identity = ApplicationWorkflowDefinitionIdentity::new("unpublished-stop-court").unwrap();
    let source = CancellationSource::new();
    let serial =
        SerialRequest::from_memory(SerialMemoryBudget::new(64 << 20), source.token(), None);
    let request = ExecutionRequest::serial(&serial);
    assert_eq!(
        selected.discover_workflow_definition::<Workflow>(&request, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ObservationDisabled)
    );
    source.cancel();
    assert_eq!(
        selected.discover_workflow_definition::<Workflow>(&request, &identity),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::Cancelled)
    );
    let expired = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 << 20),
        worth_execution::CancellationToken::new(),
        Some(std::time::Instant::now()),
    );
    assert_eq!(
        selected.discover_workflow_definition::<Workflow>(
            &ExecutionRequest::serial(&expired),
            &identity
        ),
        Err(WorthQueryWorkflowDefinitionDiscoveryDenial::DeadlineExceeded)
    );
}

//! Discovery preserves the actual lookup's request stop at the public door.
use super::super::super::tests::fixture::{installed_authorization_world, IdentityExecutionSchema};
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
struct Workflow;
impl ApplicationWorkflowSpec for Workflow {
    type Schema = IdentityExecutionSchema;
    const IDENTITY:
        worth_query_declaration::facade::application_program::ApplicationWorkflowSpecIdentity =
        worth_query_declaration::facade::application_program::ApplicationWorkflowSpecIdentity::new(
            "stop-court",
        );
}
#[test]
fn discovery_names_cancellation_deadline_and_work_at_its_door() {
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
        Ok(WorthQueryWorkflowDefinitionDiscovery::Unpublished)
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
    use crate::domain_computation::primary_graph::application_attempt::consult_observation::{
        Door, ReadConsultObservationForTest,
    };
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 << 20),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let probe =
        ReadConsultObservationForTest::install_on_this_thread_for_test(Door::Discovery, None);
    assert_eq!(
        selected.discover_workflow_definition::<Workflow>(&request, &identity),
        Ok(WorthQueryWorkflowDefinitionDiscovery::Unpublished)
    );
    let count = probe.consults();
    assert!(count > 1);
    println!("discovery door: {count} consults");
    drop(probe);
    for k in 1..=count {
        let serial = SerialRequest::from_memory(
            SerialMemoryBudget::new(64 << 20),
            worth_execution::CancellationToken::new(),
            None,
        );
        let request = ExecutionRequest::serial(&serial);
        let _probe = ReadConsultObservationForTest::install_with_work_on_this_thread_for_test(
            Door::Discovery,
            Some(k),
            move |context| {
                assert_eq!(
                    {
                        let context = context.expect("the caller's active scope");
                        context.checkpoint(context.remaining_work() + 1)
                    },
                    Err(worth_execution::MapKernelStop::WorkCeiling)
                );
            },
        );
        let (answer, _) = request
            .run(
                worth_execution::ExecutionWorkCeiling::new(u64::MAX - 1),
                |_| selected.discover_workflow_definition::<Workflow>(&request, &identity),
            )
            .unwrap();
        assert_eq!(answer, Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource { denial: super::super::super::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None })));
    }
    // The absent warm lookup borrows shared state and declares no result bytes.
    // A zero byte allowance cannot manufacture a memory-admission refusal.
    let empty = SerialRequest::from_memory(
        SerialMemoryBudget::new(0),
        worth_execution::CancellationToken::new(),
        None,
    );
    assert_eq!(
        selected
            .discover_workflow_definition::<Workflow>(&ExecutionRequest::serial(&empty), &identity),
        Ok(WorthQueryWorkflowDefinitionDiscovery::Unpublished)
    );
}

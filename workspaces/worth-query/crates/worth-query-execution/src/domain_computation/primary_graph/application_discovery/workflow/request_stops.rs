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
fn discovery_names_cancellation_deadline_work_and_memory_at_its_door() {
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
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(64 << 20),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let (result, _) = request
        .run(worth_execution::ExecutionWorkCeiling::new(0), |_| {
            selected.discover_workflow_definition::<Workflow>(&request, &identity)
        })
        .unwrap();
    assert_eq!(result, Err(WorthQueryWorkflowDefinitionDiscoveryDenial::ExecutionRequest(crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource { denial: super::super::super::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None })));
    // The lower door supplies its owned exact memory quote; Query must retain
    // that entire value, including requested bytes, without reclassifying it.
    let empty = SerialRequest::from_memory(
        SerialMemoryBudget::new(0),
        worth_execution::CancellationToken::new(),
        None,
    );
    let empty = ExecutionRequest::serial(&empty);
    let layout = world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .workflow();
    let native = world
        .application
        .primary_provider
        .graph
        .with_runtime(&empty, |runtime| {
            let request = worth_relational::facade::indexes::BoundedEntityFieldLookupRequest::new(
                selected.application_basis().snapshot_handle().clone(),
                layout.lineage.identity_index_id,
                layout.lineage.entity_kind,
                layout.lineage.identity.clone(),
                text(identity.as_str()),
                LINEAGE_LOOKUP_LIMIT,
            )
            .unwrap();
            runtime
                .index_access()
                .execute_bounded_entity_field_lookup(
                    request,
                    worth_relational::facade::indexes::BoundedIndexParityMode::Production,
                    &empty,
                )
                .unwrap_err()
                .kind()
        });
    let worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(cause) =
        native
    else {
        panic!("memory admission is an execution stop");
    };
    use crate::domain_computation::primary_graph::{
        WorthQueryManagedComputationResourceDenial as Resource, WorthQueryMemoryLimitLevel as Level,
    };
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    let (requested, admitted, level, ancestor) = match cause {
        Cause::DeclaredMemoryExhausted {
            requested,
            admitted,
        } => (requested, admitted, Level::Declared, None),
        Cause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor,
        } => (requested, admitted, Level::Policy, Some(ancestor)),
        Cause::ProcessMemoryExhausted {
            requested,
            admitted,
        } => (requested, admitted, Level::Process, None),
        other => panic!("the actual native door must refuse memory: {other:?}"),
    };
    assert_eq!(admitted, 0);
    assert_eq!(
        selected.discover_workflow_definition::<Workflow>(&empty, &identity),
        Err(
            WorthQueryWorkflowDefinitionDiscoveryDenial::ExecutionRequest(
                crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: Resource::MemoryLimit {
                        requested,
                        admitted,
                        level
                    },
                    partition_identity: None,
                    policy_ancestor: ancestor
                }
            )
        )
    );
}

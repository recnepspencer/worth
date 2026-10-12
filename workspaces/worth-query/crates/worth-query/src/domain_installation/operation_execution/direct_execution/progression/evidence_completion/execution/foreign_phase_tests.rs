use super::*;
use crate::consumer_kit::test_backend::{
    in_memory_test_product_world_resources, in_memory_test_runtime, task_schema,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
};

#[test]
fn direct_request_refuses_foreign_phase_without_panicking() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = |name| {
        in_memory_test_runtime()
            .with_schema(task_schema())
            .product_world_resources(in_memory_test_product_world_resources())
            .workspace(&resource_request, name)
            .unwrap()
    };
    let counters = crate::domain_installation::WorthQueryOperationExecutionCounters {
        runtime_authority_checks: 1,
        graph_provider_contacts: 2,
        ..Default::default()
    };
    let first = workspace("direct-phase-owner");
    let second = workspace("direct-phase-foreign");
    first
        .advancement_owner()
        .with_advancement(|phase| {
            assert!(admit_direct_request(&phase, &first, counters).is_ok());
            let refusal = admit_direct_request(&phase, &second, counters)
                .err()
                .unwrap();
            assert_eq!(refusal.counters(), counters);
            assert_eq!(
                refusal.kind(),
                &WorthQueryBoundExecutionDenialKind::ExecutionRequest(
                    WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::ForeignAdvancementPhase,
                    ),
                ),
            );
        })
        .unwrap();
}

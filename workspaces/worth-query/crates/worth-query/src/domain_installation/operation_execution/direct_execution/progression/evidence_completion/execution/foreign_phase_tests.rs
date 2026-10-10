use super::*;
use crate::consumer_kit::test_backend::{
    in_memory_test_product_world_resources, in_memory_test_runtime, task_schema,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationResourceDenial,
};

#[test]
fn direct_request_refuses_foreign_phase_without_panicking() {
    let workspace = |name| {
        in_memory_test_runtime()
            .with_schema(task_schema())
            .product_world_resources(in_memory_test_product_world_resources())
            .workspace(name)
            .unwrap()
    };
    let first = workspace("direct-phase-owner");
    let second = workspace("direct-phase-foreign");
    first
        .advancement_owner()
        .with_advancement(|phase| {
            assert!(admit_direct_request(&phase, &first).is_ok());
            let refusal = admit_direct_request(&phase, &second).err().unwrap();
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

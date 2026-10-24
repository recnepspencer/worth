//! The returned result, rather than the producing stack frame, owns its charge.
use super::*;
use crate::domain_computation::primary_graph::{
    application_contribution::{
        QueryRequestExecution, WorthQueryManagedComputationResourceDenial as Resource,
    },
    bound_advancement_requests_on_this_thread_for_test as bound,
    place_managed_computations_on_this_thread_for_test as place,
    tests::fixture::installed_authorization_world,
    WorthQueryExecutionPlacementForTest as Placement,
};
use std::num::NonZeroUsize;
use worth_foundational::ExecutionBudget;
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

fn returned_findings(
    execution: &QueryRequestExecution<'_>,
) -> WorthQueryCurrentBodyRequirementAssessment {
    let memory = execution
        .reserve(1024)
        .expect("bounded finding reservation");
    WorthQueryCurrentBodyRequirementAssessment::new(
        vec![WorthQueryOperationBodyRequirementFinding::new(
            ApplicationFieldBindingLocus::new("Account", "AccountPolicy", "AccountNote"),
            false,
        )],
        memory,
    )
}

struct Restore(Placement, Option<ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[test]
fn returned_findings_retain_native_request_memory_until_the_result_drops() {
    let _observation =
        crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    // The World policy declares no memory ceiling (absence is unlimited), so
    // this memory proof declares its own finite request budget.
    let _placement = Restore(
        place(Placement::Serial),
        bound(Some(ExecutionBudget::new(
            NonZeroUsize::MIN,
            Some(64 * 1024 * 1024),
            Some(8_000_000),
        ))),
    );
    world
        .application
        .with_host_advancement(|phase| {
            let execution = phase
                .execution_for(&world.application.product_runtime)
                .unwrap();
            let available = match execution.reserve(1_u64 << 60) {
                Err(Resource::MemoryLimit { admitted, .. }) => admitted,
                Err(cause) => panic!("expected policy quote: {cause:?}"),
                Ok(_) => panic!("oversized quote must refuse without allocating"),
            };
            assert!(available > 1024);
            let result = returned_findings(execution);
            assert_eq!(result.findings().len(), 1);
            match execution.reserve(available) {
                Err(Resource::MemoryLimit { admitted, .. }) => {
                    assert_eq!(admitted + 1024, available)
                }
                Err(cause) => panic!("unexpected reservation cause: {cause:?}"),
                Ok(_) => panic!("returned findings released their native claim too early"),
            }
            drop(result);
            assert!(
                execution.reserve(available).is_ok(),
                "dropping findings releases exactly their charge"
            );
        })
        .unwrap();
}

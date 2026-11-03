//! Actual installed decision projections; admission precedes decoder allocation.

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world_with_label;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_operation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_execution::facade::domain_computation::WorthQueryInvariantDecisionPlanDenialKind;
pub use std::cell::Cell;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;

// The compile-time marker alone must not broaden the installed decision plan.
worth_query_declaration::worth_query_operation_reads!(
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ProgramRequiredOperation => [AccountLabel]
);

pub fn ordinary_field_allocation(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
) -> (String, usize) {
    world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase.test_host_request();
            let request = live_scope();
            let principal = authenticated_principal(phase_custody, world, &request);
            let account = resolved_account(phase_custody, world, "open", &request);
            let admission = admitted_operation(
                &phase_custody.phase_for_application(&world.application),
                world,
                &principal,
                &account,
                &request,
            );
            let mut allocated = 0;
            let completed = world
                .invariant
                .project_admitted_operation(
                    &phase_custody.phase_for_application(&world.application),
                    &admission,
                    |reader, scope| {
                        // Measure only this ordinary field call, not installation, authorization,
                        // completion or retained read-set capture. This is not a peak-byte proof.
                        let region = stats_alloc::Region::new(&stats_alloc::INSTRUMENTED_SYSTEM);
                        let actual = reader
                            .decision_field(scope, AccountLabel::reference())
                            .unwrap()
                            .unwrap();
                        allocated = region.change().bytes_allocated;
                        actual
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            assert_eq!(completed.work().field_reads(), 1);
            let (actual, projection, _) = completed.into_parts();
            let _reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase_custody.phase_for_application(&world.application),
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase_custody.phase_for_application(&world.application),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            (actual, allocated)
        })
        .expect("the fixture read owns its installed host call")
}

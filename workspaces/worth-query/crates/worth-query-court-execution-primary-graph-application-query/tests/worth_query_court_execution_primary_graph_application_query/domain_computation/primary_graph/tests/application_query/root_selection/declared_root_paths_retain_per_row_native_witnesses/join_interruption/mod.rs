use crate::domain_computation::primary_graph::tests::application_query::root_selection::declared_root_paths_retain_per_row_native_witnesses::*;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption;
use worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial;
use worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub fn assert_original_join_cancellation(
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    original: &WorthQueryRetainedQueryDependencies,
    access: &worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess,
) {
    let empty = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&phase.test_host_request_for_test())
                .combine_query_dependencies(&phase, access, std::iter::empty())
                .expect("an original empty group is lawfully retained")
        })
        .unwrap();
    // Stop at the first empty part of count, validation and backing traversal.
    // Each pass traverses these same two original groups; this is no work quota.
    for stop_at in [1, 3, 5] {
        let cancel = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancel.token());
        let visits = std::cell::Cell::new(0);
        let attempt = world.application.with_application_advancement(&scope, |phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let groups = [&empty, original].into_iter().inspect(|_| {
                visits.set(visits.get() + 1);
                if visits.get() == stop_at { cancel.cancel(); }
            });
            let refused = selected.combine_query_dependencies(&phase, access, groups).err()
                .expect("the original request stops this original join");
            assert!(matches!(refused, WorthQueryRequiredSetCurrentnessDenial::Output(ref cause)
                if cause.kind() == WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    WorthQueryAdvancementDenial::Interrupted(WorthQueryManagedComputationInterruption::Cancelled))));
        });
        assert!(matches!(
            attempt,
            Ok(())
                | Err(WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                ))
        ));
        assert_eq!(visits.get(), stop_at);
    }
    world
        .application
        .with_host_advancement(|phase| {
            let selected = world.selected_product(&phase.test_host_request_for_test());
            let joined = selected
                .combine_query_dependencies(&phase, access, [&empty, original].into_iter())
                .expect("a fresh original request retains the same complete groups");
            let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
            selected
                .require_current_required_set(&phase, access, &request, std::iter::empty(), &joined)
                .expect("the real joined proof remains current");
        })
        .unwrap();
}

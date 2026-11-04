//! A native-current read cannot admit compute before managed delivery commits.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::owner::SourceInvalidationOwner;
use worth_query_execution::facade::domain_computation::WorthQueryInvalidationResources;
#[test]
fn uncommitted_upstream_never_admits_a_prepared_member() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::consumed_output::ConsumedOutputEvidence;
    let registry = WorthQueryOutputDemandRegistry::default();
    let _owner = SourceInvalidationOwner::new(WorthQueryInvalidationResources::new(), 1);
    let world =
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(
            true,
        );
    let selected = world
        .application
        .with_host_advancement(|phase| {
            (*world.application.runtime_for_test())
                .primary_graph()
                .unwrap()
                .with_runtime_for_test(
                    &phase
                        .execution_request_for_for_test(
                            &(*world.application.product_runtime_for_test()),
                        )
                        .unwrap(),
                    |runtime| {
                        let basis = runtime
                            .admit_branch_basis(&runtime.main_branch_identity())
                            .unwrap();
                        let snapshot = runtime
                            .snapshots()
                            .snapshot_for_observation(&basis.observation())
                            .unwrap();
                        Arc::new(runtime.read_truth().positioned_snapshot(&snapshot).unwrap())
                    },
                )
        })
        .expect("the wave-admission proof owns its request");
    let (_lineage, identity) = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let consumed = ConsumedOutputEvidence::retained_for_test(
        Arc::clone(&identity),
        Arc::from([]),
        Vec::new(),
        None,
        Arc::clone(&selected),
    );
    let demand_key = key("upstream", 1, 1);

    {
        let mut state = (*registry.state_for_test()).lock().unwrap();
        let (posting, _cleanup) = state
            .reserve_settlement_vacancy_for_test(&identity)
            .unwrap();
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::settlement_index::SettlementIndex::fill_prepared_for_test(
            &posting,
            &identity,
            Arc::clone(&identity),
            Arc::new(demand_key.clone()),
        );
    }
    for state in [
        DemandState::Admitted,
        DemandState::Scheduled,
        DemandState::Running,
    ] {
        (*(*registry.state_for_test())
            .lock()
            .unwrap()
            .records_mut_for_test())
        .insert_for_test(demand_key.clone(), record(occurrence(), state, 1));
        let denied = registry
            .admit_current_outputs(
                (selected.runtime_instance_id(), selected.root_id()),
                std::iter::once(&consumed),
            )
            .err()
            .unwrap();
        assert_eq!(
            denied.kind(),
            WorthQueryOutputDemandDenialKind::SchedulingDeferred,
            "a sealed read alone cannot substitute for committed upstream delivery"
        );
        let requested = denied.clone().take_requested_output_for_test().expect(
            "an unfinished delivery retains its actual read for owner-side checkpoint resumption",
        );
        assert_eq!(requested.identity(), consumed.identity().as_ref());
        assert_eq!(requested.selected(), selected.as_ref());
    }
}

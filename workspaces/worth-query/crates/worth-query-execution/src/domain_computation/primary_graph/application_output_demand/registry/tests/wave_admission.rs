//! A native-current read cannot admit compute before managed delivery commits.
use super::*;
#[test]
fn uncommitted_upstream_never_admits_a_prepared_member() {
    use crate::domain_computation::execution_runtime::source_invalidation::{
        WorthQueryInvalidationResourceInstallation, WorthQueryInvalidationResources,
    };
    use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
    use crate::domain_computation::primary_graph::output_lineage::SourceInvalidationOwner;
    let registry = WorthQueryOutputDemandRegistry::default();
    let owner = SourceInvalidationOwner::new(
        WorthQueryInvalidationResources::install(
            WorthQueryInvalidationResourceInstallation::bounded(1_000_000, 1 << 20, 1 << 20, 1),
        )
        .unwrap(),
        1,
    );
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let execution = phase
                .execution_request_for(&world.application.product_runtime)
                .unwrap();
            world
                .application
                .runtime
                .primary_graph()
                .unwrap()
                .with_runtime(&execution, |runtime| {
                    let basis = runtime
                        .admit_branch_basis(&runtime.main_branch_identity())
                        .unwrap();
                    let snapshot = runtime
                        .snapshots()
                        .snapshot_for_observation(&basis.observation())
                        .unwrap();
                    Arc::new(runtime.read_truth().positioned_snapshot(&snapshot).unwrap())
                })
        })
        .unwrap();
    let (_lineage, identity) = crate::domain_computation::primary_graph::output_lineage::registry_fixture::recorded_settlement();
    let consumed = ConsumedOutputEvidence::retained_for_test(
        &owner,
        Arc::clone(&identity),
        Arc::from([]),
        Vec::new(),
        None,
        Arc::clone(&selected),
    );
    let demand_key = key("upstream", 1, 1);
    let mut admission = record_admission();
    {
        let mut state = registry.state.lock().unwrap();
        state.required_budget_bytes = 1 << 20;
        let (posting, _cleanup) = state
            .reserve_settlement_vacancy(&identity, &mut admission)
            .unwrap();
        super::super::settlement_index::SettlementIndex::fill_prepared(
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
        registry
            .state
            .lock()
            .unwrap()
            .records
            .insert(demand_key.clone(), record(occurrence(), state, 1));
        let denied = registry
            .admit_current_outputs(
                (selected.runtime_instance_id(), selected.root_id()),
                std::iter::once(&consumed),
                &owner,
                &mut admission,
            )
            .err()
            .unwrap();
        assert_eq!(
            denied.kind(),
            WorthQueryOutputDemandDenialKind::SchedulingDeferred,
            "a sealed read alone cannot substitute for committed upstream delivery"
        );
        let requested = denied.clone().take_requested_output().expect(
            "an unfinished delivery retains its actual read for owner-side checkpoint resumption",
        );
        assert_eq!(requested.identity(), consumed.identity().as_ref());
        assert_eq!(requested.selected(), selected.as_ref());
    }
}

use super::*;

#[test]
fn cancelled_duplicate_foreign_and_capacity_bootstrap_are_exact_no_effect() {
    let mut court = CompositeSupplyChainCourt::compile();
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before_signal = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    let before_rel = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .0;
    let cancel = RuntimeWorldCancellationSource::new();
    cancel.cancel();
    let cancelled = court
        .world
        .lifecycle_port()
        .bootstrap_root(court.initial.clone().with_cancellation(cancel.token()))
        .unwrap();
    let RuntimeWorldBootstrapOutcome::NoEffect(denied) = cancelled else {
        panic!("cancelled bootstrap")
    };
    assert_eq!(
        denied.cause(),
        RuntimeWorldBootstrapNoEffectCause::Cancelled
    );
    assert!(court.world.inspection_port().history_snapshot().is_err());
    let root = court.bootstrap();
    let duplicate = court
        .world
        .lifecycle_port()
        .bootstrap_root(court.initial.clone())
        .unwrap();
    let RuntimeWorldBootstrapOutcome::NoEffect(denied) = duplicate else {
        panic!("duplicate bootstrap")
    };
    assert_eq!(
        denied.cause(),
        RuntimeWorldBootstrapNoEffectCause::AlreadyBootstrapped
    );
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before_signal
    );
    assert_eq!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .0,
        before_rel
    );
    assert_eq!(
        court
            .world
            .inspection_port()
            .history_snapshot()
            .unwrap()
            .installed_commits(),
        1
    );
    let other = CompositeSupplyChainCourt::compile();
    let foreign = other
        .world
        .lifecycle_port()
        .bootstrap_root(court.initial.clone())
        .unwrap();
    let RuntimeWorldBootstrapOutcome::NoEffect(denied) = foreign else {
        panic!("foreign bootstrap")
    };
    assert_eq!(
        denied.cause(),
        RuntimeWorldBootstrapNoEffectCause::ForeignBasis
    );
    drop(other.bootstrap());
    other.finish();
    drop(root);
    court.finish();
    let mut limited = CompositeSupplyChainCourt::compile_config(true, budgets::limited(1, 1));
    let rejected = limited
        .world
        .lifecycle_port()
        .bootstrap_root(limited.initial.clone())
        .unwrap();
    let RuntimeWorldBootstrapOutcome::NoEffect(denied) = rejected else {
        panic!("bootstrap unique-pin capacity")
    };
    assert_eq!(
        denied.cause(),
        RuntimeWorldBootstrapNoEffectCause::CapacityExhausted
    );
    assert_eq!(
        limited
            .records
            .runtime
            .branch_basis_cost_counters()
            .external_retention_acquires,
        0
    );
    assert!(limited.world.inspection_port().history_snapshot().is_err());
    let report = limited.world.lifecycle_port().close().unwrap();
    assert!(report.retained_records().is_empty());
    let relational = limited
        .records
        .runtime
        .owner_component_services()
        .lifecycle_port();
    let signal = limited
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        0
    );
    drop(limited);
    assert_eq!(
        relational.owner_lifecycle_observation(),
        worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Closed
    );
    assert_eq!(
        signal.owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
    );
}

#[test]
fn foreign_installed_bridge_witness_cannot_substitute_for_real_graph_correspondence() {
    let court = CompositeSupplyChainCourt::compile();
    let other = CompositeSupplyChainCourt::compile();
    assert!(court
        .bridge
        .runtime_world_correspondence_port()
        .admit_installed_basis(&other.installed)
        .is_err());
    court
        .bridge
        .runtime_world_correspondence_port()
        .admit_installed_basis(&court.installed)
        .unwrap();
    drop(court.bootstrap());
    drop(other.bootstrap());
    court.finish();
    other.finish();
}

#[test]
fn incompatible_correspondence_with_own_components_denies_then_healthy_binding_bootstraps() {
    let mut court = CompositeSupplyChainCourt::compile();
    let other = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let foreign = other.bootstrap();
    let world = RuntimeWorldOwner::builder()
        .with_relational_services(court.records.runtime.owner_component_services())
        .with_signal_services(court.signal.owner_component_services().unwrap())
        .with_bridge_correspondence(court.bridge.runtime_world_correspondence_port())
        .with_budgets(budgets::court())
        .with_clock(RuntimeWorldClock::from_source(budgets::CourtClock))
        .build()
        .unwrap();
    let intent = RuntimeWorldBootstrapIntent::new(
        ProductBranchCreationIntent::named("root").unwrap(),
        root.basis().relational_basis().clone(),
        root.basis().signal_basis().clone(),
        foreign.basis().correspondence_basis().clone(),
    );
    let RuntimeWorldBootstrapOutcome::NoEffect(denied) =
        world.lifecycle_port().bootstrap_root(intent).unwrap()
    else {
        panic!("foreign installed correspondence cannot compose own components")
    };
    assert_eq!(
        denied.cause(),
        RuntimeWorldBootstrapNoEffectCause::IncompatibleCorrespondence
    );
    let intent = RuntimeWorldBootstrapIntent::new(
        ProductBranchCreationIntent::named("root").unwrap(),
        root.basis().relational_basis().clone(),
        root.basis().signal_basis().clone(),
        root.basis().correspondence_basis().clone(),
    );
    let RuntimeWorldBootstrapOutcome::Performed(healthy) =
        world.lifecycle_port().bootstrap_root(intent).unwrap()
    else {
        panic!("healthy exact binding twin")
    };
    drop(healthy);
    let report = world.lifecycle_port().close().unwrap();
    assert_eq!(report.outstanding_observations(), 0);
    drop(world);
    drop((root, foreign));
    court.finish();
    other.finish();
}

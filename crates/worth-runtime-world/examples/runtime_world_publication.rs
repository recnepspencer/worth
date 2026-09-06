//! Run with `cargo run -p worth-runtime-world --example runtime_world_publication`.
#[path = "runtime_world_publication/correspondence.rs"]
mod correspondence;
#[path = "runtime_world_publication/relational.rs"]
mod relational;

use worth_runtime_world::facade::*;
use worth_signal::facade::{SignalGraph, SignalRuntime};

struct Clock;
impl RuntimeWorldClockSource for Clock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(100)
    }
}
fn budgets() -> RuntimeWorldBudgets {
    RuntimeWorldBudgets::install(RuntimeWorldBudgetInstallation {
        branches: RuntimeWorldBranchBudgetInstallation {
            live_product_branches: 4,
        },
        history: RuntimeWorldHistoryBudgetInstallation {
            retained_composite_commits: 16,
            history_metadata_bytes: 1024 * 1024,
        },
        observations: RuntimeWorldObservationBudgetInstallation {
            active_observations: 16,
        },
        publication: RuntimeWorldPublicationBudgetInstallation {
            active_publication_attempts: 4,
        },
        recovery: RuntimeWorldRecoveryBudgetInstallation {
            retained_product_unpublished_records: 4,
            retained_partial_metadata_bytes: 1024 * 1024,
        },
        retention: RuntimeWorldRetentionBudgetInstallation {
            unique_exact_component_pins: 32,
            in_flight_pin_acquisition_reservations: 8,
        },
        custody: RuntimeWorldCustodyBudgetInstallation {
            owner_created_component_custody_records: 8,
        },
    })
    .unwrap()
}
fn main() {
    // Match an explicit host stack budget; Windows' main-thread default is too
    // small for the real component owners' unoptimized construction frames.
    std::thread::Builder::new()
        .name("world-publication".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(publication_workflow)
        .unwrap()
        .join()
        .unwrap();
}
fn publication_workflow() {
    let records = relational::CargoRecords::install();
    let mut graph = SignalGraph::new();
    let source_node = graph.node().build();
    // Install correspondence into the SAME graph that the Signal owner seals.
    let (bridge, installed) = correspondence::install(&records, &mut graph, source_node);
    let mut signal = SignalRuntime::builder(graph)
        .with_domains::<()>()
        .with_impacts::<()>()
        .with_events::<()>()
        .with_context::<()>()
        .with_tiers::<()>()
        .with_kernel_defaults()
        .build();
    let initial_signal = signal
        .observe_signal_branch_basis(signal.current_branch())
        .unwrap();
    let signal_services = signal.owner_component_services().unwrap();
    let reference = signal_services
        .basis_port()
        .issue_managed_branch_reference(&initial_signal)
        .unwrap();
    let initial_signal = signal_services
        .basis_port()
        .readmit_exact(&reference, initial_signal.descriptor())
        .unwrap();
    let relational_services = records.runtime.owner_component_services();
    let initial_relational = records
        .runtime
        .observe_branch(&records.runtime.main_branch_identity())
        .unwrap()
        .1;
    let bridge_port = bridge.runtime_world_correspondence_port();
    let admitted_correspondence = bridge_port.admit_installed_basis(&installed).unwrap();
    let world = RuntimeWorldOwner::builder()
        .with_bridge_correspondence(bridge_port)
        .with_relational_services(relational_services.clone())
        .with_signal_services(signal_services.clone())
        .with_budgets(budgets())
        .with_clock(RuntimeWorldClock::from_source(Clock))
        .build()
        .unwrap();
    let RuntimeWorldBootstrapOutcome::Performed(bootstrap) = world
        .lifecycle_port()
        .bootstrap_root(RuntimeWorldBootstrapIntent::new(
            ProductBranchCreationIntent::named("main").unwrap(),
            initial_relational,
            initial_signal,
            admitted_correspondence,
        ))
        .unwrap()
    else {
        panic!("fresh World must bootstrap");
    };
    let root = bootstrap.product_branch().clone();
    drop(bootstrap);
    let publication = world.publication_port();
    let token = RuntimeWorldCancellationSource::new().token();
    let intent = |head: &ProductBranchObservation, amount: &str| {
        CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
            .with_prepared_relational_candidate(
                records.candidate(head.basis().relational_basis(), amount),
            )
    };
    let stale = publication
        .prepare_without_signal(root.clone(), intent(&root, "6"), &token, None)
        .unwrap();
    let ordinary = publication
        .prepare_without_signal(root.clone(), intent(&root, "5"), &token, None)
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(done) =
        publication.execute_without_signal(ordinary, &token)
    else {
        panic!("ordinary publication must perform");
    };
    assert_eq!(done.cost_counters().signal_owner_contacts(), 0);
    println!("Performed: {:?}", done.commit().identity());
    drop(done.consume()); // Consume exactly once; borrowed results are not new authority.
    let RuntimeWorldPublicationOutcome::NoEffect(denied) =
        publication.execute_without_signal(stale, &token)
    else {
        panic!("old observation must be stale");
    };
    assert_eq!(denied.cause(), NoEffectCause::StaleExpectedProductHead);
    drop(denied);
    let head = world
        .observation_port()
        .observe_product_branch(root.branch_identity())
        .unwrap();
    let combined =
        CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
            .with_prepared_relational_candidate(
                records.candidate(head.basis().relational_basis(), "7"),
            );
    let prepared = publication
        .prepare_with_signal(head.clone(), combined, &token, None)
        .unwrap();
    let RuntimeWorldPublicationOutcome::ProductUnpublished(effects) = publication
        .execute_with_signal(prepared, &mut (), &token, |_| {
            Err(SignalError::InvalidInput {
                message: "example application declines the Signal step".into(),
                context: None,
            })
        })
    else {
        panic!("Relational moved but product publication did not happen");
    };
    assert_eq!(effects.owner_effect_count(), 1);
    let recovery = world.recovery_port();
    let handle = effects.recovery_handle();
    println!(
        "ProductUnpublished: {:?}; explicit cleanup follows",
        effects.cause()
    );
    recovery.continue_effects(effects).unwrap(); // Settlement/cleanup only; never call a sibling or CAS.
    recovery.release_effects(&handle, 0).unwrap();
    assert_eq!(
        world
            .observation_port()
            .observe_product_branch(head.branch_identity())
            .unwrap(),
        head
    );
    assert_ne!(
        records
            .runtime
            .observe_branch(head.basis().relational_basis().identity())
            .unwrap()
            .1
            .admission_identity(),
        head.basis().relational_basis().admission_identity()
    ); // Cleanup did not roll back owner truth.
    drop((root, head));
    let close = world.lifecycle_port().close().unwrap();
    assert_eq!(close.outstanding_observations(), 0);
    assert!(close.retained_records().is_empty());
    drop((close, world, installed, bridge, reference));
    let pins = records.runtime.branch_basis_cost_counters();
    assert_eq!(
        pins.external_retention_acquires,
        pins.external_retention_releases
    );
    drop((records, signal));
    assert_eq!(
        relational_services
            .lifecycle_port()
            .owner_lifecycle_observation(),
        worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Closed
    );
    assert!(matches!(
        signal_services
            .lifecycle_port()
            .owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
    ));
    println!("Owners closed; no retained partial or observation remains.");
}

//! Original owner artifacts survive unrelated unsealed work and real live-history retirement.
use crate::court::ordinary_context::CompositeSupplyChainCourt;
#[test]
fn older_memory_original_survives_an_unrelated_unsealed_publication() {
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let policy = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let first = court.publish_cargo(&root, "6");
    let receipt = first
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt()
        .clone();
    let original_head = court.observe(&root);
    let original_commit = original_head.selected_commit().clone();
    let generation = original_head.reference_generation().get();
    drop(first);
    let effects = court.partial_after_relational(&execution, &original_head);
    assert_eq!(effects.owner_effect_count(), 1);
    assert_eq!(court.observe(&original_head), original_head);
    let installed = || {
        court
            .world
            .inspection_port()
            .history_snapshot()
            .unwrap()
            .installed_commits()
    };
    assert_eq!(installed(), 3);
    let native = court
        .records
        .runtime
        .durability_authority()
        .observe_memory_commit(&execution, &receipt)
        .unwrap();
    let observed = court
        .world
        .observation_port()
        .observe_memory_original_publication(&execution, native)
        .unwrap();
    assert_eq!(
        observed.after().selected_commit(),
        original_commit.ordinal()
    );
    assert_eq!(observed.after().generation(), generation);
    assert_eq!(observed.native().receipt(), &receipt);
    assert_eq!(
        installed(),
        3,
        "a read cannot republish or remove the unsealed event"
    );
    let recovery = court.world.recovery_port();
    let handle = effects.recovery_handle();
    drop(effects);
    let cleanup = recovery
        .release_effects(
            &execution,
            &handle,
            0,
            court.observe_recovery_original(&execution, &handle),
        )
        .unwrap()
        .into_parts();
    let (_original, cleanup) = cleanup;
    assert!(cleanup.owner_retirement_work().is_empty());
    drop(cleanup);
    drop((observed, root, original_head));
    court.finish();
}
#[test]
fn memory_original_remains_readable_after_its_live_history_is_retired() {
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let policy = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&policy);
    let first = court.publish_cargo(&root, "5");
    let receipt = first
        .component_results()
        .relational_commit_result()
        .unwrap()
        .semantic_result()
        .receipt()
        .clone();
    let first_head = court.observe(&root);
    let original = first_head.selected_commit().clone();
    let generation = first_head.reference_generation().get();
    let later = court.publish_cargo(&first_head, "6");
    let head = court.observe(&first_head);
    drop((first, later, first_head));
    let retired = court
        .world
        .lifecycle_port()
        .retire_unprotected_history(head.selected_commit())
        .unwrap();
    assert!(
        retired.contains(&original),
        "the original live World history really retired"
    );
    assert!(court
        .world
        .recovery_port()
        .retained_commits(&[original.clone()])
        .unwrap()
        .is_empty());
    let native = court
        .records
        .runtime
        .durability_authority()
        .observe_memory_commit(&execution, &receipt)
        .unwrap();
    let observed = court
        .world
        .observation_port()
        .observe_memory_original_publication(&execution, native)
        .unwrap();
    assert_eq!(observed.after().selected_commit(), original.ordinal());
    assert_eq!(observed.after().generation(), generation);
    assert_eq!(observed.native().receipt(), &receipt);
    let recorded = court
        .records
        .runtime
        .owner_component_services()
        .basis_port()
        .export_memory_component(&execution, observed.after_component())
        .unwrap();
    let worth_relational::facade::branch::RecordedRelationalComponentTarget::Canonical(component) =
        recorded.target()
    else {
        panic!("actual original is canonical")
    };
    assert_eq!(component.receipt(), &receipt);
    drop((recorded, observed, head, root));
    court.finish();
}

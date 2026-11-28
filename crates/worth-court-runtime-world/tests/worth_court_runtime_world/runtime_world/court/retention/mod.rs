pub(crate) use worth_runtime_world::facade::*;
#[test]
fn history_and_observation_dependencies_hold_exact_pins_without_clone_amplification() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let inspection = court.world.inspection_port();
    let keys = [
        RuntimeWorldRetentionKey::relational(root.basis()),
        RuntimeWorldRetentionKey::signal(root.basis()),
    ];
    let live_claims = || {
        keys.each_ref().map(|key| {
            inspection
                .inspect_retention(key)
                .unwrap()
                .unwrap()
                .live_claims()
        })
    };
    let before = live_claims();
    // Each component is held by the head, history, and this observation.
    assert_eq!(before, [3, 3]);
    let clone = root.clone();
    assert_eq!(live_claims(), before);
    let prepared = court.prepare_cargo(execution, &root, "5");
    assert_eq!(
        inspection.recovery_snapshot().unwrap().reserved(),
        1,
        "prepared attempt reserves bounded recovery capacity"
    );
    drop(prepared);
    assert_eq!(live_claims(), before);
    let receipt = court.publish_cargo(&root, "5");
    let head = court.observe(&root);
    let traversal = inspection
        .trace_ancestry(
            head.selected_commit().clone(),
            std::num::NonZeroUsize::new(1).unwrap(),
        )
        .unwrap();
    assert_eq!(traversal.visited_count(), 1);
    assert_eq!(traversal.next_parent(), Some(root.selected_commit()));
    // The live branch owns its head and everything behind it.
    let released = court
        .world
        .lifecycle_port()
        .release_retired_branch_history(head.selected_commit())
        .unwrap();
    assert!(released.reclaimed_commits().is_empty());
    assert_eq!(released.held_commit(), None);
    // The successor has a new Relational basis and reuses the original Signal basis.
    assert_eq!(live_claims(), [2, 5]);
    drop((root, clone, head, traversal, receipt));
    court.finish();
}
#[test]
fn full_history_denies_before_owner_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile_config(
        true,
        crate::court::ordinary_context::budgets::limited(1, 8),
    );
    let root = court.bootstrap();
    let before = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .0;
    let candidate =
        court
            .records
            .candidate(execution, root.basis().relational_basis(), "grain", "5");
    let denied = court
        .world
        .publication_port()
        .prepare_without_signal(
            root.clone(),
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(candidate),
            &RuntimeWorldCancellationSource::new().token(),
            None,
        )
        .unwrap_err();
    assert_eq!(denied.cause(), NoEffectCause::CapacityExhausted);
    assert_eq!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
    assert_eq!(
        court
            .world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .reserved(),
        0
    );
    drop((root, denied));
    court.finish();
}

#[test]
fn a_retired_root_walk_releases_its_history_and_the_pins_only_it_held() {
    let court = crate::court::ordinary_context::CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let identity = root.selected_commit().clone();
    let keys = [
        RuntimeWorldRetentionKey::relational(root.basis()),
        RuntimeWorldRetentionKey::signal(root.basis()),
    ];
    let retired = court
        .world
        .branch_port()
        .retire_product_branch(&root)
        .unwrap();
    assert!(retired.owner_retirement_work().is_empty());
    drop((root, retired));
    let result = court
        .world
        .lifecycle_port()
        .release_retired_branch_history(&identity)
        .unwrap();
    assert_eq!(result.reclaimed_commits(), &[identity]);
    assert_eq!(result.held_commit(), None);
    let inspection = court.world.inspection_port();
    for key in keys {
        assert!(inspection.inspect_retention(&key).unwrap().is_none());
    }
    assert_eq!(inspection.retention_snapshot().unwrap().unique_pins(), 0);
    court.finish();
}

mod artifact_retention;
mod artifact_source;

mod branch_data;

mod owner_evidence;

mod durable_siblings;

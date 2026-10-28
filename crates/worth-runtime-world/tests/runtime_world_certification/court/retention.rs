use super::*;
use std::num::NonZeroUsize;
#[test]
fn history_and_observation_dependencies_hold_exact_pins_without_clone_amplification() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let inspection = court.world.inspection_port();
    let key = RuntimeWorldRetentionKey::signal(root.basis());
    let before = inspection
        .inspect_retention(&key)
        .unwrap()
        .unwrap()
        .dependencies();
    assert_eq!(
        before.get(ComponentBasisDependencyClass::ProductBranchHead),
        1
    );
    assert_eq!(
        before.get(ComponentBasisDependencyClass::RetainedCompositeHistory),
        1
    );
    assert_eq!(
        before.get(ComponentBasisDependencyClass::AdmittedObservation),
        1
    );
    let clone = root.clone();
    assert_eq!(
        inspection
            .inspect_retention(&key)
            .unwrap()
            .unwrap()
            .dependencies(),
        before
    );
    let prepared = court.prepare_cargo(execution, &root, "5");
    assert_eq!(
        inspection.recovery_snapshot().unwrap().reserved(),
        1,
        "prepared attempt reserves bounded recovery capacity"
    );
    drop(prepared);
    assert_eq!(
        inspection
            .inspect_retention(&key)
            .unwrap()
            .unwrap()
            .dependencies(),
        before
    );
    let receipt = court.publish_cargo(&root, "5");
    let head = court.observe(&root);
    let traversal = inspection
        .trace_ancestry(
            head.selected_commit().clone(),
            NonZeroUsize::new(1).unwrap(),
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
    assert_eq!(
        inspection
            .inspect_retention(&key)
            .unwrap()
            .unwrap()
            .dependencies()
            .get(ComponentBasisDependencyClass::RetainedCompositeHistory),
        2
    );
    drop((root, clone, head, traversal, receipt));
    court.finish();
}
#[test]
fn full_history_denies_before_owner_publication() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);
    let court = CompositeSupplyChainCourt::compile_config(true, budgets::limited(1, 8));
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
    let court = CompositeSupplyChainCourt::compile();
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
    // The walk reclaims the exact pins only that history held; nothing is
    // left for a later retention pass.
    let after = court
        .world
        .lifecycle_port()
        .reclaim_retention(&keys, 2)
        .unwrap();
    assert_eq!(after.reclaimed(), 0);
    assert_eq!(after.remaining_unique_pins(), 0);
    court.finish();
}

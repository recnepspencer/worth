use super::*;
use std::num::NonZeroUsize;
#[test]
fn history_and_observation_dependencies_hold_exact_pins_without_clone_amplification() {
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
    let prepared = court.prepare_cargo(&root, "5");
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
    let reclaimed = court
        .world
        .lifecycle_port()
        .reclaim_history(CompositeHistoryReclamationRequest::new(
            court.world.owner_identity(),
            vec![
                root.selected_commit().clone(),
                head.selected_commit().clone(),
            ],
            1,
        ))
        .unwrap();
    assert!(reclaimed.reclaimed_commits().is_empty());
    assert!(reclaimed.examined() <= 1);
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
    let court = CompositeSupplyChainCourt::compile_config(true, budgets::limited(1, 8));
    let root = court.bootstrap();
    let before = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .0;
    let candidate = court
        .records
        .candidate(root.basis().relational_basis(), "grain", "5");
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
fn unreachable_root_reclamation_is_bounded_and_releases_named_exact_pins() {
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
        .reclaim_history(CompositeHistoryReclamationRequest::new(
            court.world.owner_identity(),
            vec![identity.clone()],
            1,
        ))
        .unwrap();
    assert_eq!(result.reclaimed_commits(), &[identity]);
    assert_eq!(result.examined(), 1);
    let first = court
        .world
        .lifecycle_port()
        .reclaim_retention(&keys, 1)
        .unwrap();
    assert_eq!(first.examined(), 1);
    assert_eq!(first.reclaimed(), 1);
    let second = court
        .world
        .lifecycle_port()
        .reclaim_retention(&keys[1..], 1)
        .unwrap();
    assert_eq!(second.reclaimed(), 1);
    assert_eq!(second.remaining_unique_pins(), 0);
    court.finish();
}

use super::*;
use std::num::NonZeroUsize;
#[test]
fn partial_recovery_preserves_movement_rejects_foreign_and_only_releases_obligations() {
    let mut court = CompositeSupplyChainCourt::compile();
    let other = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let foreign_root = other.bootstrap();
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    let effects = court.partial_after_relational(&root);
    assert_eq!(effects.cause(), ProductUnpublishedCause::SiblingOwnerDenied);
    assert_eq!(effects.owner_effect_count(), 1);
    let moved = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .1;
    assert_ne!(
        moved.admission_identity(),
        root.basis().relational_basis().admission_identity()
    );
    assert_eq!(court.records.read(&moved).records["grain"], "5");
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before
    );
    assert_eq!(
        court.observe(&root).selected_commit(),
        root.selected_commit()
    );
    let handle = effects.recovery_handle();
    let recovery = court.world.recovery_port();
    assert!(matches!(
        other.world.recovery_port().inspect_effects(&handle),
        Err(RuntimeWorldRecoveryDenial::ForeignHandle)
    ));
    assert!(matches!(
        recovery.release_effects(&handle, 0),
        Err(RuntimeWorldRecoveryDenial::CallerCapabilityLive)
    ));
    let continuation = recovery.continue_effects(effects).unwrap();
    assert!(continuation
        .actions()
        .contains(&ProductUnpublishedNextAction::ReleaseObligations));
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before,
        "resume never calls unfinished sibling"
    );
    let page = court
        .world
        .inspection_port()
        .recovery_page(None, NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(page.examined(), 1);
    assert_eq!(page.rows()[0].age_ticks(), Some(0));
    assert!(matches!(
        recovery.release_effects(&handle, 1),
        Err(RuntimeWorldRecoveryDenial::TooYoung)
    ));
    let inspected = recovery.inspect_effects(&handle).unwrap();
    drop(inspected);
    let cleanup = recovery.release_effects(&handle, 0).unwrap();
    assert!(cleanup.owner_retirement_work().is_empty());
    assert_eq!(cleanup.unpublished_history_candidates().len(), 1);
    assert!(matches!(
        recovery.inspect_effects(&handle),
        Err(RuntimeWorldRecoveryDenial::MissingRecord)
    ));
    assert_eq!(
        court.observe(&root).selected_commit(),
        root.selected_commit(),
        "cleanup cannot publish product truth"
    );
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before
    );
    assert_eq!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .1
            .admission_identity(),
        moved.admission_identity()
    );
    drop((root, foreign_root, moved));
    court.finish();
    other.finish();
}
#[test]
fn performed_capability_loss_reopens_one_claim_and_consumption_closes_it() {
    let court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let prepared = court.prepare_cargo(&root, "5");
    let RuntimeWorldPublicationOutcome::Performed(done) = court
        .world
        .publication_port()
        .execute_without_signal(prepared, &RuntimeWorldCancellationSource::new().token())
    else {
        panic!("healthy publication")
    };
    let identity = done.commit().identity().clone();
    assert!(court
        .world
        .recovery_port()
        .recover_performed(&identity)
        .is_err());
    drop(done);
    let recovered = court
        .world
        .recovery_port()
        .recover_performed(&identity)
        .unwrap();
    let receipt = recovered.consume();
    assert!(court
        .world
        .recovery_port()
        .recover_performed(&identity)
        .is_err());
    assert_eq!(court.observe(&root).selected_commit(), &identity);
    drop((root, receipt));
    court.finish();
}

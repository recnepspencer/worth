use super::*;

#[test]
fn cancellation_prepared_drop_and_stale_head_do_not_move_components() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let signal_before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    let before = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .0;
    drop(court.prepare_cargo(&root, "5"));
    assert_eq!(
        court
            .world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .reserved(),
        0
    );
    let prepared = court.prepare_cargo(&root, "5");
    let cancel = RuntimeWorldCancellationSource::new();
    cancel.cancel();
    let RuntimeWorldPublicationOutcome::NoEffect(denied) = court
        .world
        .publication_port()
        .execute_without_signal(prepared, &cancel.token())
    else {
        panic!("pre-effect cancellation")
    };
    assert_eq!(denied.cause(), NoEffectCause::CancelledBeforeEffect);
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
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        signal_before
    );
    drop(denied);
    let stale_prepared = court.prepare_cargo(&root, "7");
    let healthy = court.publish_cargo(&root, "5");
    let current = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .0;
    let RuntimeWorldPublicationOutcome::NoEffect(denied) =
        court.world.publication_port().execute_without_signal(
            stale_prepared,
            &RuntimeWorldCancellationSource::new().token(),
        )
    else {
        panic!("stale expected product head")
    };
    assert_eq!(denied.cause(), NoEffectCause::StaleExpectedProductHead);
    assert_eq!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .0,
        current
    );
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        signal_before
    );
    drop((root, healthy, denied));
    court.finish();
}

#[test]
fn foreign_candidate_denial_has_a_healthy_candidate_twin() {
    let court = CompositeSupplyChainCourt::compile();
    let other = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let foreign = other.bootstrap();
    let before = court
        .records
        .runtime
        .observe_branch(&court.records.runtime.main_branch_identity())
        .unwrap()
        .0;
    let candidate = other
        .records
        .candidate(foreign.basis().relational_basis(), "grain", "8");
    let prepared = court
        .world
        .publication_port()
        .prepare_without_signal(
            root.clone(),
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(candidate),
            &RuntimeWorldCancellationSource::new().token(),
            None,
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::NoEffect(denied) = court
        .world
        .publication_port()
        .execute_without_signal(prepared, &RuntimeWorldCancellationSource::new().token())
    else {
        panic!("foreign candidate must not move either owner")
    };
    assert_eq!(denied.cause(), NoEffectCause::OwnerDeniedBeforeEffect);
    assert_eq!(
        court
            .records
            .runtime
            .observe_branch(&court.records.runtime.main_branch_identity())
            .unwrap()
            .0,
        before
    );
    drop(court.publish_cargo(&root, "5"));
    drop((root, foreign, denied));
    court.finish();
    other.finish();
}

#[test]
fn signal_denial_before_movement_has_no_partial_and_healthy_twin() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let port = court.world.publication_port();
    let token = RuntimeWorldCancellationSource::new().token();
    let signal = court
        .signal
        .owner_component_services()
        .unwrap()
        .basis_port();
    let before = signal
        .owner_service_cost_snapshot()
        .unwrap()
        .canonical_movements();
    let prepared = port
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(None),
            &token,
            None,
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::NoEffect(denied) =
        port.execute_with_signal(prepared, &mut court.context(&root), &token, |_| {
            Err(SignalError::InvalidInput {
                message: "routing input rejected".into(),
                context: None,
            })
        })
    else {
        panic!("Signal pre-movement denial")
    };
    assert_eq!(denied.cause(), NoEffectCause::PreEffectFailure);
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before
    );
    assert_eq!(
        court
            .world
            .inspection_port()
            .recovery_snapshot()
            .unwrap()
            .installed(),
        0
    );
    let prepared = port
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(None),
            &token,
            None,
        )
        .unwrap();
    let RuntimeWorldPublicationOutcome::Performed(done) =
        port.execute_with_signal(prepared, &mut court.context(&root), &token, |_| Ok(()))
    else {
        panic!("healthy Signal twin")
    };
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before + 1
    );
    drop((root, denied, done.consume()));
    court.finish();
}

#[test]
fn public_owner_loss_denies_reserved_execution_without_component_movement() {
    let mut court = CompositeSupplyChainCourt::compile();
    let root = court.bootstrap();
    let prepared = court.prepare_cargo(&root, "5");
    let port = court.world.publication_port();
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
    let CompositeSupplyChainCourt {
        world,
        records,
        signal: signal_owner,
        bridge,
        installed,
        initial,
        ..
    } = court;
    drop(world);
    let RuntimeWorldPublicationOutcome::NoEffect(denied) =
        port.execute_without_signal(prepared, &RuntimeWorldCancellationSource::new().token())
    else {
        panic!("World owner loss must deny reserved execution")
    };
    assert_eq!(denied.cause(), NoEffectCause::OwnerUnavailable);
    assert_eq!(
        signal
            .owner_service_cost_snapshot()
            .unwrap()
            .canonical_movements(),
        before_signal
    );
    assert_eq!(
        records
            .runtime
            .observe_branch(&records.runtime.main_branch_identity())
            .unwrap()
            .0,
        before_rel
    );
    drop((root, denied, bridge, installed, initial));
    let pins = records.runtime.branch_basis_cost_counters();
    assert_eq!(
        pins.external_retention_acquires,
        pins.external_retention_releases
    );
    let rel_lifecycle = records.runtime.owner_component_services().lifecycle_port();
    drop((records, signal_owner));
    assert_eq!(
        rel_lifecycle.owner_lifecycle_observation(),
        worth_relational::facade::branch::RelationalOwnerLifecycleObservation::Closed
    );
    assert_eq!(
        signal.owner_lifecycle_observation(),
        worth_signal::facade::branch::SignalOwnerLifecycleObservation::Closed
    );
}

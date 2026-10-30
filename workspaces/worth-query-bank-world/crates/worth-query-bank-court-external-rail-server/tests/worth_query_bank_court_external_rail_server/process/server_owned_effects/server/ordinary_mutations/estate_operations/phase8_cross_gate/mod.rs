pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::phase8_cross_gate::*;

#[test]
fn lost_response_recovery_through_real_rail_and_aftermath() {
    let world = cross_gate_world("lost-response-recovery");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, world::PATIENT);
    let revision_before = world.estate_account_revision();
    let receipt = world.commit_with(world::idempotency(71));
    assert_lost_response_commit(&world, &receipt);
    assert_unresolved_recovery(&world, &receipt, revision_before);
}

#[test]
fn unrelated_aftermath_lookup_cannot_substitute_the_handle_contract() {
    let world = cross_gate_world("aftermath-slot-substitution");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, world::PATIENT);
    let receipt = world.commit_notification(85);
    let aftermath = installed_notify_death_aftermath(&world.fixture.world.runtime);
    let substituted = installed_disburse_aftermath(&world.fixture.world.runtime);
    assert_ne!(substituted.identity(), aftermath.identity());
    let specialist = world.fixture.authenticate_specialist();
    let admitted = world
        .fixture
        .world
        .runtime
        .reconcile_commit_recovery(
            world.open_recovery(&receipt),
            &specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect("handle-carried NotifyDeath aftermath admits reconcile");
    assert_eq!(admitted.installed_operation(), aftermath.operation_slot());
}

#[test]
fn already_completed_resolve_returns_inherited_taxonomy() {
    let world = cross_gate_world("already-completed");
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let receipt = world.commit_with(world::idempotency(76));
    let specialist = world.fixture.authenticate_specialist();
    let resolution = world.fixture.world.runtime.resolve_commit_recovery(
        world.open_recovery(&receipt),
        &specialist,
        world.specialist_action(),
        &request_scope(),
    );
    match resolution {
        Ok(BankRecoveryIdempotencyResolution::AlreadyCommitted) => {}
        Err(BankEstateProgressionDenial::Recovery(d))
            if d.kind() == BankRecoveryDenialKind::UnresolvedExternalPosture => {}
        other => panic!("expected AlreadyCommitted or unresolved posture, got {other:?}"),
    }
}

#[test]
fn inspect_requires_disclosure_proof_not_boolean() {
    let world = cross_gate_world("inspect-disclosure");
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let receipt = world.commit_notification(74);
    let specialist = world.fixture.authenticate_specialist();
    let view = world
        .fixture
        .world
        .runtime
        .inspect_commit_recovery(
            &world.open_recovery(&receipt),
            &specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect("disclosure-backed inspect");
    assert_eq!(view.recovery_inspection_work().basis_preparations(), 0);
}

use worth_query_declaration::facade::application_schema::ApplicationRetainedEffectBinding;
use worth_runtime_bridge::facade::TruthBranchHeadSource;

use super::{admitted_program_with_emit, authenticated_principal, idempotency, resolved_account};
use crate::domain_computation::primary_graph::tests::fixture::{
    installed_authorization_world, live_scope, AccountActivityBinding,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedChange, WorthQueryApplicationCommitOutcome,
};

#[test]
fn one_missing_batch_slot_denies_before_world_or_bridge_movement() {
    {
        assert_live_reservation_denial(0, u64::MAX, "slot-short", 211);
    }
}

#[test]
fn one_missing_payload_byte_denies_before_world_or_bridge_movement() {
    {
        let payload = "byte-short".to_owned();
        let required = AccountActivityBinding::retained_bytes(&payload);
        assert!(required > 0);
        assert_live_reservation_denial(1, required - 1, &payload, 212);
    }
}

fn assert_live_reservation_denial(
    batch_capacity: usize,
    byte_capacity: u64,
    payload: &str,
    identity: u8,
) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (program, branch, product_before, bridge_before, _live) = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let phase_custody = &custody;
            let principal = authenticated_principal(phase_custody, &world, &request);
            let account = resolved_account(phase_custody, &world, "open", &request);
            let branch = world.application.current_world();
            let selected = world
                .application
                .on_branch(branch)
                .select(&phase_custody.phase_for_application(&world.application))
                .unwrap();
            let product_before = selected.product().selected_commit().clone();
            let bridge_before = bridge_head_commit(&world);
            let _live = world
                .application
                .primary_provider
                .observe_application_commit_causality(selected.product());
            world
                .application
                .primary_provider
                .set_application_commit_causality_limits(batch_capacity, byte_capacity);
            let program = admitted_program_with_emit(
                &phase_custody.phase_for_application(&world.application),
                &world,
                &principal,
                &account,
                &request,
                "must-not-move",
                Some(payload),
            );

            (program, branch, product_before, bridge_before, _live)
        })
        .expect("the fixture decision owns its host call");
    let outcome = world
        .application
        .on_branch(branch)
        .transaction()
        .apply(WorthQueryAdmittedChange::new(
            program,
            idempotency(identity, identity),
        ))
        .commit()
        .expect("the public product token remains admitted");
    let Err(outcome) = outcome.require_committed() else {
        panic!("fanout reservation failure cannot satisfy require_committed");
    };
    assert!(matches!(
        outcome,
        WorthQueryApplicationCommitOutcome::Aborted
    ));
    let product_after = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let phase_custody = &custody;
            world
                .application
                .on_branch(branch)
                .select(&phase_custody.phase_for_application(&world.application))
                .unwrap()
        })
        .expect("the fixture aftermath read owns its host call");
    assert_eq!(product_after.product().selected_commit(), &product_before);
    assert_eq!(bridge_head_commit(&world), bridge_before);
    assert_eq!(
        world
            .application
            .primary_provider
            .published_application_commit_count(),
        0
    );
    world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&phase.test_host_request(), &world, "open", &request);
        })
        .expect("the fixture entity read owns its host call");
}

fn bridge_head_commit(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
) -> worth_runtime_bridge::facade::TruthCommitIdentity {
    world
        .application
        .primary_provider
        .graph
        .relational_bridge_source()
        .load_branch_head_patch(
            &crate::domain_computation::primary_graph::primary_truth_branch_identity(),
        )
        .expect("the bridge truth head remains available")
        .commit_identity()
        .clone()
}

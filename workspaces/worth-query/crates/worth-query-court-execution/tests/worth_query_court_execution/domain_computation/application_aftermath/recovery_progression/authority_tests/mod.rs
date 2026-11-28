//! Recovery authority remains affine to a live, exact handle.

use super::fixture::{
    AuthorizationWorld, RecoveryAdmission, committed_handle, committed_status_handle,
    effect_authority, world,
};
use std::time::{Duration, SystemTime};
use worth_foundational::facade::RecoveryClaimDisposition;
use worth_query_execution::facade::domain_computation::{
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenialKind as Denial,
    WorthQueryRecoveryInspectAuthority,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::{
    application_attempt::authenticated_principal,
    fixture::{admit_touch_account_capability, installed_inspection_capability_world, schema_types::live_scope},
};

#[test]
fn concurrent_force_termination_cannot_be_reported_as_a_consumed_transition() {
    let world = world();
    let (handle, _, _) = committed_handle(&world);
    let registry = handle.registry_arc_for_test();
    let slot = handle.registry_slot_for_test();
    assert!(registry.mark_terminal_for_test(slot, RecoveryClaimDisposition::ForceTerminated));

    let denied = world
        .application
        .with_host_advancement(|phase| {
            handle.consume_for_test(&phase, RecoveryClaimDisposition::Consumed)
        })
        .unwrap()
        .expect_err("force termination wins the terminal race");
    assert_eq!(denied.kind(), Denial::AlreadyTerminal);
    assert_eq!(
        registry.terminal_of_for_test(slot),
        Some(RecoveryClaimDisposition::ForceTerminated)
    );
    registry.assert_no_live_handles_for_test();
}

#[test]
fn effect_authority_minted_before_expiry_denies_after_the_deadline() {
    let world = world();
    let now = SystemTime::now();
    world.authorization_time.hold(now);
    let (handle, admission, _) = committed_handle(&world);
    let authority = effect_authority(&world, &handle, &admission);
    authority.ensure_for(&handle).unwrap();

    world
        .authorization_time
        .hold(now + Duration::from_secs(3_601));
    assert_eq!(
        authority.ensure_for(&handle).unwrap_err().kind(),
        Denial::Expired
    );
}

#[test]
fn inspect_authority_minted_before_expiry_denies_after_the_deadline() {
    let (world, handle, admission, _) = inspection_handles();
    let authority = inspect_authority(&world, &handle, &admission);
    authority.ensure_for(&handle).unwrap();

    world
        .authorization_time
        .hold(inspection_time() + Duration::from_secs(3_601));
    assert_eq!(
        authority.ensure_for(&handle).unwrap_err().kind(),
        Denial::Expired
    );
}

#[test]
fn admission_and_proof_use_distinguish_foreign_runtimes_with_equal_slots() {
    let (world_a, handle_a, admission_a, _) = inspection_handles();
    let world_b = world();
    let (handle_b, _, _) = committed_handle(&world_b);
    assert_eq!(
        handle_a.registry_slot_for_test(),
        handle_b.registry_slot_for_test()
    );

    let effect = effect_authority(&world_a, &handle_a, &admission_a);
    let inspect = inspect_authority(&world_a, &handle_a, &admission_a);
    effect.ensure_for(&handle_a).unwrap();
    inspect.ensure_for(&handle_a).unwrap();
    assert_eq!(
        world_a
            .application
            .admit_recovery_effect_authority(&handle_b, &admission_a)
            .unwrap_err()
            .kind(),
        Denial::ForeignRuntime
    );
    assert_eq!(
        effect.ensure_for(&handle_b).unwrap_err().kind(),
        Denial::FreshAuthorityDenied
    );
    assert_eq!(
        inspect.ensure_for(&handle_b).unwrap_err().kind(),
        Denial::FreshAuthorityDenied
    );
}

#[test]
fn effect_and_inspect_authority_deny_another_handle_in_the_same_runtime() {
    let (world, handle_a, admission, handle_b) = inspection_handles();
    assert_ne!(
        handle_a.registry_slot_for_test(),
        handle_b.registry_slot_for_test()
    );

    let effect = effect_authority(&world, &handle_a, &admission);
    let inspect = inspect_authority(&world, &handle_a, &admission);
    effect.ensure_for(&handle_a).unwrap();
    inspect.ensure_for(&handle_a).unwrap();
    assert_eq!(
        effect.ensure_for(&handle_b).unwrap_err().kind(),
        Denial::FreshAuthorityDenied
    );
    assert_eq!(
        inspect.ensure_for(&handle_b).unwrap_err().kind(),
        Denial::FreshAuthorityDenied
    );
}

fn inspection_time() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(100)
}

fn inspection_handles() -> (
    AuthorizationWorld,
    WorthQueryRecoveryHandle,
    RecoveryAdmission,
    WorthQueryRecoveryHandle,
) {
    let world = installed_inspection_capability_world();
    world.authorization_time.hold(inspection_time());
    let (original, admission, _) = committed_handle(&world);
    // The disclosure grant is current only while this account is open.
    let (reopened, _, _) = committed_status_handle(&world, 191, "closed", "open");
    (world, original, admission, reopened)
}

fn inspect_authority(
    world: &AuthorizationWorld,
    handle: &WorthQueryRecoveryHandle,
    admission: &RecoveryAdmission,
) -> WorthQueryRecoveryInspectAuthority {
    let request = live_scope();
    let disclosure = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let principal = authenticated_principal(&host, world, &request);
            let access =
                admit_touch_account_capability(&host, world, &principal, &request).unwrap();
            world
                .application
                .admit_recovery_inspection_disclosure(&access)
                .unwrap()
        })
        .unwrap();
    world
        .application
        .admit_recovery_inspect_authority(handle, admission, &disclosure)
        .unwrap()
}

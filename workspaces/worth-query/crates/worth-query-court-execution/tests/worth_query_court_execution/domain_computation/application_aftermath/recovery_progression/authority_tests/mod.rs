//! Q8.20 — recovery authority is affine to its owning runtime.

use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::recovery_progression::authority::require_inspect_disclosure;
use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::recovery_progression::authority::require_fresh_effect_authority;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryEffectAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryInspectAuthority;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandle;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::application_aftermath::recovery_handle::binding::WorthQueryRecoveryHandleBindingAxisProbe;
use worth_query_execution::facade::domain_computation::WorthQueryRecoveryHandleDenialKind;
use worth_query_execution::court_doors_for_test::domain_computation::managed_run::recovery_registry::WorthQueryRecoveryResourceTerminal;

pub fn probe_handle_with_runtime_instance(runtime_instance_id: u64) -> WorthQueryRecoveryHandle {
    probe_handle_expiring_at(runtime_instance_id, u64::MAX)
}

pub fn probe_handle_expiring_at(
    runtime_instance_id: u64,
    expires_at_unix_ms: u64,
) -> WorthQueryRecoveryHandle {
    WorthQueryRecoveryHandle::axis_probe_for_test(
        WorthQueryRecoveryHandleBindingAxisProbe::real()
            .runtime_instance_id(runtime_instance_id)
            .expires_at_unix_ms(Some(expires_at_unix_ms))
            .finish(),
    )
}

#[test]
fn concurrent_force_termination_cannot_be_reported_as_a_consumed_transition() {
    let handle = probe_handle_with_runtime_instance(91);
    let registry = handle.registry_arc_for_test();
    let slot = handle.registry_slot_for_test();
    assert!(
        registry.mark_terminal_for_test(slot, WorthQueryRecoveryResourceTerminal::ForceTerminated)
    );

    let denied = handle
        .consume_for_test(WorthQueryRecoveryResourceTerminal::Consumed)
        .expect_err("force termination wins the terminal race");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::AlreadyTerminal
    );
    assert_eq!(
        registry.terminal_of_for_test(slot),
        Some(WorthQueryRecoveryResourceTerminal::ForceTerminated)
    );
    registry.assert_no_live_handles_for_test();
}

// R8.31 — the "fresh" in `require_fresh_effect_authority` has to mean fresh at
// *use*, not fresh at mint. Expiry was checked once, inside
// `admit_recovery_effect_authority`, and never again; the authority object is
// not `Clone` but it is also not consumed, so holding one across the handle's
// deadline kept every transition open indefinitely. Both tests below mint the
// authority exactly the way the runtime does and then present it against a
// handle whose deadline has passed.
#[test]
fn effect_authority_minted_before_expiry_denies_after_the_deadline() {
    let handle = probe_handle_expiring_at(31, 1);
    let authority = WorthQueryRecoveryEffectAuthority::mint_for_test(
        handle.runtime_authority_for_test(),
        handle.authority_identity_for_test(),
    );
    let registry = handle.registry_arc_for_test();

    // The handle really is past its deadline against the runtime's own clock,
    // so this is not a test that passes because nothing expired.
    assert!(matches!(
        worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::recovery_progression::expiry::evaluate_expiry(&handle, registry.clock_for_test()).expect("clock samples"),
        worth_query_execution::facade::domain_computation::WorthQueryRecoveryExpiryEvaluation::Expired(_)
    ));

    let denied = require_fresh_effect_authority(&handle, &authority)
        .expect_err("authority minted before the deadline cannot transition after it");
    assert_eq!(denied.kind(), WorthQueryRecoveryHandleDenialKind::Expired);

    drop(handle);
    registry.assert_no_live_handles_for_test();
}

#[test]
fn inspect_authority_minted_before_expiry_denies_after_the_deadline() {
    let handle = probe_handle_expiring_at(32, 1);
    let authority = WorthQueryRecoveryInspectAuthority::mint_for_test(
        handle.runtime_authority_for_test(),
        handle.authority_identity_for_test(),
    );
    let registry = handle.registry_arc_for_test();

    let denied = require_inspect_disclosure(&handle, &authority)
        .expect_err("disclosure minted before the deadline cannot inspect after it");
    assert_eq!(denied.kind(), WorthQueryRecoveryHandleDenialKind::Expired);

    drop(handle);
    registry.assert_no_live_handles_for_test();
}

#[test]
fn effect_authority_denies_a_different_runtime_with_the_same_registry_slot() {
    let (handle_a, handle_b) = colliding_slot_handles();
    let authority_a = WorthQueryRecoveryEffectAuthority::mint_for_test(
        handle_a.runtime_authority_for_test(),
        handle_a.authority_identity_for_test(),
    );
    let registry_a = handle_a.registry_arc_for_test();
    let registry_b = handle_b.registry_arc_for_test();

    require_fresh_effect_authority(&handle_a, &authority_a).expect("owner runtime admits");
    let denied = require_fresh_effect_authority(&handle_b, &authority_a)
        .expect_err("equal registry slot cannot substitute for runtime ownership");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
    );

    drop(handle_a);
    drop(handle_b);
    registry_a.assert_no_live_handles_for_test();
    registry_b.assert_no_live_handles_for_test();
}

#[test]
fn inspect_authority_denies_a_different_runtime_with_the_same_registry_slot() {
    let (handle_a, handle_b) = colliding_slot_handles();
    let authority_a = WorthQueryRecoveryInspectAuthority::mint_for_test(
        handle_a.runtime_authority_for_test(),
        handle_a.authority_identity_for_test(),
    );
    let registry_a = handle_a.registry_arc_for_test();
    let registry_b = handle_b.registry_arc_for_test();

    require_inspect_disclosure(&handle_a, &authority_a).expect("owner runtime admits");
    let denied = require_inspect_disclosure(&handle_b, &authority_a)
        .expect_err("equal registry slot cannot substitute for runtime ownership");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
    );

    drop(handle_a);
    drop(handle_b);
    registry_a.assert_no_live_handles_for_test();
    registry_b.assert_no_live_handles_for_test();
}

#[test]
fn effect_authority_denies_a_different_handle_in_the_same_runtime() {
    let (handle_a, handle_b) = same_runtime_handles();
    let authority_a = WorthQueryRecoveryEffectAuthority::mint_for_test(
        handle_a.runtime_authority_for_test(),
        handle_a.authority_identity_for_test(),
    );
    let registry = handle_a.registry_arc_for_test();

    require_fresh_effect_authority(&handle_a, &authority_a).expect("exact handle admits");
    let denied = require_fresh_effect_authority(&handle_b, &authority_a)
        .expect_err("same-runtime handle substitution must deny");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
    );

    drop(handle_a);
    drop(handle_b);
    registry.assert_no_live_handles_for_test();
}

#[test]
fn inspect_authority_denies_a_different_handle_in_the_same_runtime() {
    let (handle_a, handle_b) = same_runtime_handles();
    let authority_a = WorthQueryRecoveryInspectAuthority::mint_for_test(
        handle_a.runtime_authority_for_test(),
        handle_a.authority_identity_for_test(),
    );
    let registry = handle_a.registry_arc_for_test();

    require_inspect_disclosure(&handle_a, &authority_a).expect("exact handle admits");
    let denied = require_inspect_disclosure(&handle_b, &authority_a)
        .expect_err("same-runtime handle substitution must deny");
    assert_eq!(
        denied.kind(),
        WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
    );

    drop(handle_a);
    drop(handle_b);
    registry.assert_no_live_handles_for_test();
}

// Q8.20-C4 — the two foreignness denials must not collapse into one another.
// The ledger cited `binding_axis::foreign_runtime_drift_denies_distinctly` for
// this, but slice 3B removed the runtime-instance axis and that test went with
// it, leaving the admission-time `ForeignRuntime` cause with no coverage at all
// while only the proof-use `FreshAuthorityDenied` half stayed tested.
#[test]
fn admission_time_foreignness_and_proof_use_foreignness_are_different_causes() {
    let (handle_a, handle_b) = colliding_slot_handles();
    let owner = handle_a.runtime_authority_for_test();
    let registry_a = handle_a.registry_arc_for_test();
    let registry_b = handle_b.registry_arc_for_test();

    // Admission time: the handle presented to obtain authority is not ours.
    let admission_denial =
        worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::recovery_progression::authority::ensure_same_runtime(owner, handle_b.runtime_authority_for_test())
            .expect_err("a handle from another runtime cannot be admitted here");
    assert_eq!(
        admission_denial.kind(),
        WorthQueryRecoveryHandleDenialKind::ForeignRuntime
    );
    worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::recovery_progression::authority::ensure_same_runtime(owner, handle_a.runtime_authority_for_test())
        .expect("our own handle admits");

    // Proof use: the authority *is* ours, but it is not this handle's.
    let authority_a = WorthQueryRecoveryEffectAuthority::mint_for_test(
        owner,
        handle_a.authority_identity_for_test(),
    );
    let use_denial = require_fresh_effect_authority(&handle_b, &authority_a)
        .expect_err("our authority does not carry to another handle");
    assert_eq!(
        use_denial.kind(),
        WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied
    );

    assert_ne!(admission_denial.kind(), use_denial.kind());

    drop(handle_a);
    drop(handle_b);
    registry_a.assert_no_live_handles_for_test();
    registry_b.assert_no_live_handles_for_test();
}

pub fn colliding_slot_handles() -> (WorthQueryRecoveryHandle, WorthQueryRecoveryHandle) {
    let handle_a = probe_handle_with_runtime_instance(11);
    let handle_b = probe_handle_with_runtime_instance(22);
    assert_eq!(handle_a.registry_slot_for_test().as_u64_for_test(), 1);
    assert_eq!(handle_b.registry_slot_for_test().as_u64_for_test(), 1);
    assert_ne!(
        handle_a.runtime_authority_for_test(),
        handle_b.runtime_authority_for_test()
    );
    (handle_a, handle_b)
}

pub fn same_runtime_handles() -> (WorthQueryRecoveryHandle, WorthQueryRecoveryHandle) {
    use std::sync::Arc;

    use worth_query_execution::court_doors_for_test::domain_computation::managed_run::recovery_registry::WorthQueryRecoveryHandleRegistry;

    let seed = probe_handle_with_runtime_instance(11);
    let binding = seed.binding().clone();
    drop(seed);
    let registry = Arc::new(WorthQueryRecoveryHandleRegistry::new_for_test());
    let handle_a = WorthQueryRecoveryHandle::axis_probe_in_registry_for_test(
        binding.clone(),
        Arc::clone(&registry),
    );
    let handle_b = WorthQueryRecoveryHandle::axis_probe_in_registry_for_test(binding, registry);
    assert_eq!(
        handle_a.runtime_authority_for_test(),
        handle_b.runtime_authority_for_test()
    );
    assert_ne!(
        handle_a.registry_slot_for_test(),
        handle_b.registry_slot_for_test()
    );
    (handle_a, handle_b)
}

//! Safe retry while the one terminal owner cannot yet answer.
//!
//! A completion the rail already performed may still be settling: its World
//! publication can wait for the estate rail source, or fail and leave the
//! terminal index unable to answer. Each state refuses with its own recovery
//! kind, returns the live handle, contacts nothing, and later answers
//! `AlreadyCompleted` once the completion settles.

use std::sync::Arc;

use bank_external_rail::test_control::FaultScript;
use bank_server::{
    BankCommitRecoveryHandle, BankEstateProgressionDenial, BankEstateRailCompletionRoute,
    BankRecoveryDenialKind,
};
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectPostureKind;

use super::phase8_cross_gate::world::{self, CrossGateWorld};
use super::phase8_safe_retry_completed::EstateRailSource;
use crate::support::request_scope;

/// Commits one notification whose transport completes synchronously while no
/// estate rail source is installed, so its completion publication waits.
fn completed_before_the_source_installs(
    label: &str,
    seed: u8,
) -> (CrossGateWorld, BankCommitRecoveryHandle, [u8; 32]) {
    let world = world::cross_gate_world(label);
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let receipt = world.commit_notification(seed);
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Completed)
    );
    let correlation = world.transport.attempts()[0].clone();
    let token = correlation.token().try_into().expect("32-byte token");
    // Any contact would run this fault path, so a refused retry that reached
    // the rail would show up in the counts below.
    world
        .transport
        .under(FaultScript::DuplicateAcknowledgement, world::PATIENT);
    let handle = world.open_recovery(&receipt);
    (world, handle, token)
}

/// Runs one safe retry that must be refused with `expected`, and proves the
/// refusal returned the live handle and added no rail contact.
fn refused_retry(
    world: &CrossGateWorld,
    handle: BankCommitRecoveryHandle,
    expected: BankRecoveryDenialKind,
) -> BankCommitRecoveryHandle {
    let specialist = world.fixture.authenticate_specialist();
    let denied = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(
            handle,
            &specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect_err("a settling completion admits no new physical attempt");
    let (denied, retained) = denied.into_parts();
    match denied {
        BankEstateProgressionDenial::Recovery(denied) => assert_eq!(denied.kind(), expected),
        other => panic!("expected {expected:?}, got {other:?}"),
    }
    // (rail contacts, adapter crossings, rail admissions, consequences)
    let observed = (
        world.transport.attempts().len(),
        world.transport.production_dispatches().len(),
        world.transport.admission_count(),
        world.transport.completed_effect_count(),
    );
    assert_eq!(observed, (1, 1, 1, 1), "{expected:?} contacted the rail");
    retained.expect("a settling refusal returns the live handle")
}

fn install_source(world: &CrossGateWorld) -> BankEstateRailCompletionRoute {
    world
        .fixture
        .world
        .runtime
        .install_estate_rail_completion_verifier(Arc::new(EstateRailSource))
        .expect("the estate rail source installs once")
}

#[test]
fn safe_retry_names_a_pending_completion_publication_then_the_completion() {
    let (world, handle, token) =
        completed_before_the_source_installs("safe-retry-publication-pending", 83);
    let handle = refused_retry(
        &world,
        handle,
        BankRecoveryDenialKind::CompletionPublicationPending,
    );
    let route = install_source(&world);
    refused_retry(&world, handle, BankRecoveryDenialKind::AlreadyCompleted);
    assert!(
        world
            .fixture
            .world
            .runtime
            .observe_estate_rail_completion(&route, token)
            .is_some(),
        "the retry published the retained completion into the terminal owner"
    );
}

#[test]
fn safe_retry_names_an_unavailable_terminal_index_until_maintenance_settles_it() {
    let (world, handle, token) =
        completed_before_the_source_installs("safe-retry-index-unavailable", 85);
    let route = install_source(&world);
    // The retained completion's World publication fails durably, so the
    // terminal index keeps the correlation pending until recovery settles it.
    world
        .fixture
        .world
        .runtime
        .fail_next_durable_append_for_test()
        .expect("the Bank fixture requires an open application owner");
    let handle = refused_retry(
        &world,
        handle,
        BankRecoveryDenialKind::CompletionPublicationPending,
    );
    let handle = refused_retry(
        &world,
        handle,
        BankRecoveryDenialKind::TerminalIndexUnavailable,
    );
    assert!(world
        .fixture
        .world
        .runtime
        .observe_estate_rail_completion(&route, token)
        .is_none());
    world
        .fixture
        .world
        .runtime
        .maintain_estate_rail_completion(&route, &request_scope())
        .expect("maintenance resumes the unpublished completion");
    refused_retry(&world, handle, BankRecoveryDenialKind::AlreadyCompleted);
    assert!(world
        .fixture
        .world
        .runtime
        .observe_estate_rail_completion(&route, token)
        .is_some());
}

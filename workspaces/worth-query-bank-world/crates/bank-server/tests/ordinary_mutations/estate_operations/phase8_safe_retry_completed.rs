//! Safe retry after the one terminal effect owner already holds completion.
//!
//! 9.17.7 converges synchronous completion, safe retry and inbound completion
//! on one terminal owner. Once that owner holds completion, no new physical
//! attempt is admitted: safe retry returns `AlreadyCompleted` with the live
//! handle, contacts nothing, and the consequence owner still holds one effect.

use std::sync::Arc;

use bank_external_rail::test_control::FaultScript;
use bank_external_rail::LedgerStatus;
use bank_server::{BankEstateProgressionDenial, BankRecoveryDenialKind};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundOccurrenceClaims, WorthQueryInboundOccurrenceVerifier,
    WorthQueryInboundVerificationDenial,
};
use worth_query_host::facade::publication::application_aftermath::WorthQueryPublishedExternalEffectPostureKind;

use super::phase8_cross_gate::world;
use crate::support::request_scope;

/// The Bank process installs its estate rail source before serving. This
/// journey completes through the transport, so no callback is authenticated.
struct EstateRailSource;

impl WorthQueryInboundOccurrenceVerifier for EstateRailSource {
    fn audience(&self) -> &str {
        "bank-estate-safe-retry"
    }

    fn source_identity(&self) -> &str {
        "rail-primary"
    }

    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        static IDENTITY: BoundaryProtocolIdentity =
            BoundaryProtocolIdentity::new("bank.estate.death-notification");
        &IDENTITY
    }

    fn protocol_version(&self) -> BoundaryProtocolVersion {
        BoundaryProtocolVersion::new(1)
    }

    fn verify(
        &self,
        _envelope: &[u8],
        _now_unix_seconds: u64,
        _maximum_work: std::num::NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        Err(WorthQueryInboundVerificationDenial::AuthenticationFailed)
    }
}

#[test]
fn safe_retry_of_already_completed_effect_adds_no_contact_and_no_physical_consequence() {
    let world = world::cross_gate_world("safe-retry-completed");
    let route = world
        .fixture
        .world
        .runtime
        .install_estate_rail_completion_verifier(Arc::new(EstateRailSource))
        .expect("the estate rail source installs once");
    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let receipt = world.commit_notification(82);
    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Completed)
    );
    let correlation = world.transport.attempts()[0].clone();
    let token: [u8; 32] = correlation.token().try_into().expect("32-byte token");
    let terminal = world
        .fixture
        .world
        .runtime
        .observe_estate_rail_completion(&route, token)
        .expect("synchronous completion entered the one terminal owner");

    let mut handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let action = world.specialist_action();
    let scope = request_scope();
    // Any contact would run this fault path; completion must win before
    // transport admission, so the rail is never asked (9.17.7 race court).
    world
        .transport
        .under(FaultScript::DuplicateAcknowledgement, world::PATIENT);
    for retry in 0..2 {
        let denied = world
            .fixture
            .world
            .runtime
            .safe_retry_commit_recovery(handle, &specialist, action, &scope)
            .expect_err("an already-completed effect admits no new physical attempt");
        let (denied, retained) = denied.into_parts();
        match denied {
            BankEstateProgressionDenial::Recovery(denied) => assert_eq!(
                denied.kind(),
                BankRecoveryDenialKind::AlreadyCompleted,
                "retry {retry} must name the terminal completion"
            ),
            other => panic!("expected AlreadyCompleted, got {other:?}"),
        }
        handle = retained.expect("an already-completed refusal returns the live handle");
        // (rail contacts, adapter crossings, rail admissions, consequences)
        let observed = (
            world.transport.attempts().len(),
            world.transport.production_dispatches().len(),
            world.transport.admission_count(),
            world.transport.completed_effect_count(),
        );
        assert_eq!(observed, (1, 1, 1, 1), "retry {retry} contacted the rail");
        let status = world.transport.ledger_status(&correlation);
        assert_eq!(status, LedgerStatus::Completed);
        let still = world
            .fixture
            .world
            .runtime
            .observe_estate_rail_completion(&route, token)
            .expect("the terminal owner keeps its completion");
        assert_eq!(
            (
                still.original_world_commit(),
                still.completion_world_commit()
            ),
            (
                terminal.original_world_commit(),
                terminal.completion_world_commit()
            ),
            "retry {retry} must keep the original terminal provenance"
        );
    }
}

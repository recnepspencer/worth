//! A claimed focus reveal must be an actual Scroll-owned participant.
//!
//! The reveal witness names a Scroll scope, but naming one proves nothing unless
//! that owner really holds an occupancy lease, stages its own family witness, and
//! acknowledges its own terminal outcome. These cases pin that end to end and
//! kill the shape where a portal proposal claims a reveal that no owner compiled.

use super::super::{
    fixture_service_request_coherence, UiServiceFamilyParticipation,
    UiServiceProposalOccupancyScopeIdentity,
};
use super::{
    UiServiceFamilyProposal, UiServiceProposalCandidate, UiServiceProposalCompiler,
    UiServiceProposalDemand, UiServiceProposalReservationOutcome, UiServiceProposalStage,
    UiServiceProposalStageIssuer, UiServiceProposalStageReceipt, UiServiceProposalStagingDenial,
};

type Family = crate::capability::UiRuntimeServiceFamily;

const PORTAL_SCOPE: u64 = 1;

fn portal_shaped_support() -> crate::capability::UiRuntimeServiceSupport {
    crate::capability::UiRuntimeServiceSupport::none_installed()
        .with_installed(Family::Portal)
        .with_installed(Family::Focus)
        .with_installed(Family::Scroll)
}

/// Portal + Focus + the Scroll reveal participant, all at the portal scope, which
/// is the exact shape a portal transition compiles when it carries no Motion.
fn portal_reveal_candidate(
    identity: u64,
    coherence: &super::super::UiServiceRequestCoherence,
    families: &[Family],
) -> UiServiceProposalCandidate {
    let proposals = families
        .iter()
        .map(|family| UiServiceFamilyProposal::recorded_fixture(*family, PORTAL_SCOPE, 1, 1, 0))
        .collect::<Vec<_>>();
    let count = families.len();
    UiServiceProposalCandidate::for_test(
        identity,
        UiServiceProposalDemand::recorded_fixture(
            UiServiceFamilyParticipation::from_families(families)
                .expect("distinct portal-shaped families"),
            u8::try_from(count).unwrap(),
            u16::try_from(count).unwrap(),
            0,
        ),
        coherence.clone(),
        proposals,
    )
}

fn scope() -> UiServiceProposalOccupancyScopeIdentity {
    UiServiceProposalOccupancyScopeIdentity::for_test(PORTAL_SCOPE)
}

fn family_witness(
    proposal: super::UiServiceProposalIdentity,
    family: Family,
) -> UiServiceProposalStageReceipt {
    UiServiceProposalStageReceipt::recorded_family_fixture(
        proposal,
        family,
        scope(),
        vec![super::UiServiceProducedFactReference::recorded_fixture(
            21,
            family,
            scope(),
        )],
        Vec::new(),
    )
}

#[test]
fn a_compiled_reveal_participant_holds_its_own_lease_witness_and_terminal_outcome() {
    let mut compiler = UiServiceProposalCompiler::new();
    let coherence = fixture_service_request_coherence(820);
    let candidate = portal_reveal_candidate(
        8_200,
        &coherence,
        &[Family::Portal, Family::Focus, Family::Scroll],
    );
    let preflighted = compiler
        .preflight(candidate, &coherence, portal_shaped_support())
        .expect("a portal-shaped proposal preflights against portal support");
    let UiServiceProposalReservationOutcome::Reserved(reservation) = compiler
        .reserve(preflighted)
        .expect("portal proposal reserves")
    else {
        unreachable!("an independent portal proposal cannot coalesce")
    };
    let proposal = reservation.identity();

    // One lease per participant: the reveal owner is compiled, not implied.
    assert_eq!(
        compiler.census().entries(),
        [
            ("proposals", 1),
            ("occupancy_leases", 3),
            ("cancellation_records", 1),
            ("stage_receipts", 0),
        ]
    );

    let mut staging = compiler
        .begin_staging(reservation)
        .unwrap_or_else(|(_, denial)| panic!("reserved portal proposal stages: {denial:?}"));
    for family in [Family::Portal, Family::Focus, Family::Scroll] {
        compiler
            .advance_staging(&mut staging, family_witness(proposal, family))
            .expect("every compiled participant stages its own witness");
    }
    compiler
        .advance_staging(
            &mut staging,
            UiServiceProposalStageReceipt::recorded_stage_fixture(
                proposal,
                UiServiceProposalStage::AssembleSuccessor,
                UiServiceProposalStageIssuer::ExistingPreparation,
            ),
        )
        .expect("existing preparation assembles the successor");
    compiler
        .advance_staging(
            &mut staging,
            UiServiceProposalStageReceipt::focus_resolution(proposal, Some(scope())),
        )
        .expect("the reveal names the Scroll owner that staged at this scope");
    let batch = compiler
        .finish_staging(staging)
        .unwrap_or_else(|(_, denial)| panic!("portal reveal batch finishes: {denial:?}"));

    assert_eq!(batch.reveal_refinement(), Some(scope()));
    assert_eq!(
        compiler.census().entries()[3].1,
        5,
        "three family witnesses plus assembly plus focus resolution"
    );

    // Every compiled participant, including the reveal owner, must acknowledge a
    // terminal outcome before the proposal releases.
    let mut teardown = compiler.cancel_staged(batch);
    for family in [Family::Portal, Family::Focus, Family::Scroll] {
        compiler
            .acknowledge_terminal_owner(
                &mut teardown,
                super::UiRecordedServiceProposalOwnerPort::recorded_fixture(family, scope())
                    .terminal_outcome(
                        proposal,
                        super::UiServiceProposalTerminalReason::CancelledBeforePublication,
                    ),
            )
            .expect("each compiled participant acknowledges its own teardown");
    }
    compiler
        .finish_teardown(teardown)
        .unwrap_or_else(|(_, denial)| panic!("complete teardown releases: {denial:?}"));
    assert!(compiler.census().is_zero());
    assert_eq!(compiler.live_occupancy_count(), 0);
}

#[test]
fn a_portal_proposal_without_scroll_support_is_denied_rather_than_dropping_the_reveal_owner() {
    let mut compiler = UiServiceProposalCompiler::new();
    let coherence = fixture_service_request_coherence(821);
    let candidate = portal_reveal_candidate(
        8_210,
        &coherence,
        &[Family::Portal, Family::Focus, Family::Scroll],
    );
    let without_scroll = crate::capability::UiRuntimeServiceSupport::none_installed()
        .with_installed(Family::Portal)
        .with_installed(Family::Focus);

    let denial = compiler
        .preflight(candidate, &coherence, without_scroll)
        .err()
        .expect("a world that cannot own the reveal must be denied");
    assert_eq!(
        denial,
        super::UiServiceProposalPreflightDenial::UnsupportedFamily(Family::Scroll),
        "the reveal participant is never silently dropped from a portal proposal"
    );
    assert!(compiler.census().is_zero());
}

#[test]
fn a_reveal_claimed_without_a_staged_scroll_owner_cannot_advance() {
    let mut compiler = UiServiceProposalCompiler::new();
    let coherence = fixture_service_request_coherence(822);
    let candidate = portal_reveal_candidate(8_220, &coherence, &[Family::Portal, Family::Focus]);
    let preflighted = compiler
        .preflight(
            candidate,
            &coherence,
            crate::capability::UiRuntimeServiceSupport::none_installed()
                .with_installed(Family::Portal)
                .with_installed(Family::Focus),
        )
        .expect("a two-family proposal preflights");
    let UiServiceProposalReservationOutcome::Reserved(reservation) =
        compiler.reserve(preflighted).expect("proposal reserves")
    else {
        unreachable!()
    };
    let proposal = reservation.identity();
    let mut staging = compiler
        .begin_staging(reservation)
        .unwrap_or_else(|(_, denial)| panic!("proposal stages: {denial:?}"));
    for family in [Family::Portal, Family::Focus] {
        compiler
            .advance_staging(&mut staging, family_witness(proposal, family))
            .expect("participating families stage");
    }
    compiler
        .advance_staging(
            &mut staging,
            UiServiceProposalStageReceipt::recorded_stage_fixture(
                proposal,
                UiServiceProposalStage::AssembleSuccessor,
                UiServiceProposalStageIssuer::ExistingPreparation,
            ),
        )
        .expect("existing preparation assembles the successor");

    assert_eq!(
        compiler.advance_staging(
            &mut staging,
            UiServiceProposalStageReceipt::focus_resolution(proposal, Some(scope())),
        ),
        Err(UiServiceProposalStagingDenial::UnbackedRevealRefinement),
        "a reveal with no compiled Scroll participant is a claim, not a replan"
    );

    let mut teardown = compiler.cancel_staging(staging);
    for family in [Family::Portal, Family::Focus] {
        compiler
            .acknowledge_terminal_owner(
                &mut teardown,
                super::UiRecordedServiceProposalOwnerPort::recorded_fixture(family, scope())
                    .terminal_outcome(
                        proposal,
                        super::UiServiceProposalTerminalReason::CancelledBeforePublication,
                    ),
            )
            .expect("each participant acknowledges teardown");
    }
    compiler
        .finish_teardown(teardown)
        .unwrap_or_else(|(_, denial)| panic!("teardown releases: {denial:?}"));
    assert!(compiler.census().is_zero());
}

use super::super::{
    UiRecordedServiceProposalOwnerPort, UiRecordedServiceProposalPublicationPort,
    UiServiceProposalPublicationDisposition, UiServiceProposalPublicationReceipt,
    UiServiceProposalSettlementDenial, UiServiceProposalStage, UiServiceProposalStageIssuer,
    UiServiceProposalStageReceipt, UiServiceProposalTerminalReason,
};
use crate::runtime::session::service_proposal::{
    fixture_service_request_coherence, UiServiceFamilyParticipation,
    UiServiceProposalOccupancyScopeIdentity,
};

#[test]
fn accept_and_reject_reach_every_owner_then_release_exactly_to_zero() {
    for (identity, disposition, reason) in [
        (
            70,
            UiServiceProposalPublicationDisposition::Accepted,
            UiServiceProposalTerminalReason::PublicationAccepted,
        ),
        (
            71,
            UiServiceProposalPublicationDisposition::Rejected,
            UiServiceProposalTerminalReason::PublicationRejected,
        ),
    ] {
        let (mut compiler, batch) = staged(identity, &[family::Portal, family::Focus]);
        let publication = UiRecordedServiceProposalPublicationPort::recorded_fixture()
            .report(&batch, disposition);
        let mut settlement = compiler.begin_settlement(batch, publication).unwrap();
        for (family, scope) in [(family::Focus, 2), (family::Portal, 1)] {
            let owner =
                UiRecordedServiceProposalOwnerPort::recorded_fixture(family, scope_id(scope));
            compiler
                .acknowledge_owner(&mut settlement, owner.acknowledge(publication))
                .unwrap();
        }
        let terminal = compiler.finish_settlement(settlement).unwrap();
        assert_eq!(terminal.reason(), reason);
        assert_eq!(terminal.released_leases(), 2);
        assert_eq!(terminal.released_receipts(), 7);
        assert!(compiler.census().is_zero());
        assert_eq!(compiler.live_occupancy_count(), 0);
        assert_eq!(compiler.live_cancellation_count(), 0);
    }
}

#[test]
fn malformed_publication_and_owner_evidence_are_atomic_and_recoverable() {
    let (mut compiler, batch) = staged(80, &[family::Portal]);
    let before = compiler.census();
    let foreign = UiServiceProposalPublicationReceipt::recorded_foreign_fixture(
        super::super::UiServiceProposalIdentity::for_test(81),
        batch.digest(),
        UiServiceProposalPublicationDisposition::Accepted,
    );
    let (batch, denial) = compiler.begin_settlement(batch, foreign).unwrap_err();
    assert_eq!(
        denial,
        super::super::UiServiceProposalPublicationDenial::ForeignProposal
    );
    assert_eq!(compiler.census(), before);

    let publication = UiRecordedServiceProposalPublicationPort::recorded_fixture()
        .report(&batch, UiServiceProposalPublicationDisposition::Accepted);
    let mut settlement = compiler.begin_settlement(batch, publication).unwrap();
    let portal = UiRecordedServiceProposalOwnerPort::recorded_fixture(family::Portal, scope_id(1));
    let mismatched_receipt = UiServiceProposalPublicationReceipt::recorded_foreign_fixture(
        publication.proposal(),
        publication.batch_digest(),
        UiServiceProposalPublicationDisposition::Rejected,
    );
    let before_ack = compiler.census();
    assert_eq!(
        compiler.acknowledge_owner(&mut settlement, portal.acknowledge(mismatched_receipt)),
        Err(UiServiceProposalSettlementDenial::PublicationDispositionMismatch)
    );
    assert_eq!(compiler.census(), before_ack);
    compiler
        .acknowledge_owner(&mut settlement, portal.acknowledge(publication))
        .unwrap();
    let before_duplicate = compiler.census();
    assert_eq!(
        compiler.acknowledge_owner(&mut settlement, portal.acknowledge(publication)),
        Err(UiServiceProposalSettlementDenial::DuplicateOwnerAcknowledgement)
    );
    assert_eq!(compiler.census(), before_duplicate);
    compiler.finish_settlement(settlement).unwrap();
    assert!(compiler.census().is_zero());
}

#[test]
fn cancellation_and_shutdown_cover_each_prepublication_phase_without_silence() {
    let (mut compiler, reservation) = reserved(90, &[family::Portal]);
    let receipt = compiler.shutdown_reservation(reservation).unwrap();
    assert_eq!(
        receipt.reason(),
        UiServiceProposalTerminalReason::AbandonedAtShutdown
    );
    assert!(compiler.census().is_zero());

    let (mut compiler, reservation) = reserved(91, &[family::Portal]);
    let mut staging = compiler.begin_staging(reservation).unwrap();
    stage_family(&mut compiler, &mut staging, 91, family::Portal, 1);
    let mut teardown = compiler.cancel_staging(staging);
    let owner = UiRecordedServiceProposalOwnerPort::recorded_fixture(family::Portal, scope_id(1));
    let before_missing = compiler.census();
    let (returned, denial) = compiler.finish_teardown(teardown).unwrap_err();
    assert_eq!(
        denial,
        super::super::UiServiceProposalTeardownDenial::IncompleteOwnerDiscard
    );
    assert_eq!(compiler.census(), before_missing);
    teardown = returned;
    compiler
        .acknowledge_terminal_owner(
            &mut teardown,
            owner.terminal_outcome(
                super::super::UiServiceProposalIdentity::for_test(91),
                UiServiceProposalTerminalReason::CancelledBeforePublication,
            ),
        )
        .unwrap();
    let receipt = compiler.finish_teardown(teardown).unwrap();
    assert_eq!(
        receipt.reason(),
        UiServiceProposalTerminalReason::CancelledBeforePublication
    );
    assert_eq!(receipt.released_receipts(), 2);
    assert!(compiler.census().is_zero());

    let (mut compiler, batch) = staged(92, &[family::Portal]);
    let proposal = batch.identity();
    let mut teardown = compiler.shutdown_staged(batch);
    let owner = UiRecordedServiceProposalOwnerPort::recorded_fixture(family::Portal, scope_id(1));
    compiler
        .acknowledge_terminal_owner(
            &mut teardown,
            owner.terminal_outcome(
                proposal,
                UiServiceProposalTerminalReason::AbandonedAtShutdown,
            ),
        )
        .unwrap();
    let receipt = compiler.finish_teardown(teardown).unwrap();
    assert_eq!(
        receipt.reason(),
        UiServiceProposalTerminalReason::AbandonedAtShutdown
    );
    assert_eq!(receipt.released_receipts(), 3);
    assert!(compiler.census().is_zero());
}

#[test]
fn settlement_shutdown_requires_typed_outcomes_from_every_remaining_owner() {
    let (mut compiler, batch) = staged(100, &[family::Portal, family::Focus]);
    let publication = UiRecordedServiceProposalPublicationPort::recorded_fixture()
        .report(&batch, UiServiceProposalPublicationDisposition::Rejected);
    let mut settlement = compiler.begin_settlement(batch, publication).unwrap();
    let portal = UiRecordedServiceProposalOwnerPort::recorded_fixture(family::Portal, scope_id(1));
    compiler
        .acknowledge_owner(&mut settlement, portal.acknowledge(publication))
        .unwrap();
    let before = compiler.census();
    let (returned, denial) = compiler.finish_settlement(settlement).unwrap_err();
    assert_eq!(
        denial,
        UiServiceProposalSettlementDenial::IncompleteOwnerSettlement
    );
    assert_eq!(compiler.census(), before);
    settlement = returned;
    let mut teardown = compiler.shutdown_awaiting_settlement(settlement);
    assert_eq!(compiler.census(), before);
    let focus = UiRecordedServiceProposalOwnerPort::recorded_fixture(family::Focus, scope_id(2));
    let (returned, denial) = compiler.finish_teardown(teardown).unwrap_err();
    assert_eq!(
        denial,
        super::super::UiServiceProposalTeardownDenial::IncompleteOwnerDiscard
    );
    teardown = returned;
    compiler
        .acknowledge_terminal_owner(
            &mut teardown,
            focus.terminal_outcome(
                super::super::UiServiceProposalIdentity::for_test(100),
                UiServiceProposalTerminalReason::AbandonedAtShutdown,
            ),
        )
        .unwrap();
    let receipt = compiler.finish_teardown(teardown).unwrap();
    assert_eq!(receipt.released_receipts(), 7);
    assert!(compiler.census().is_zero());
}

#[test]
fn digest_and_owner_scope_mismatches_are_atomic_denials() {
    let (mut compiler, batch) = staged(101, &[family::Portal]);
    let before = compiler.census();
    let bad_digest = UiServiceProposalPublicationReceipt::recorded_foreign_fixture(
        batch.identity(),
        batch.digest().wrapping_add(1),
        UiServiceProposalPublicationDisposition::Accepted,
    );
    let (batch, denial) = compiler.begin_settlement(batch, bad_digest).unwrap_err();
    assert_eq!(
        denial,
        super::super::UiServiceProposalPublicationDenial::BatchDigestMismatch
    );
    assert_eq!(compiler.census(), before);

    let publication = UiRecordedServiceProposalPublicationPort::recorded_fixture()
        .report(&batch, UiServiceProposalPublicationDisposition::Accepted);
    let mut settlement = compiler.begin_settlement(batch, publication).unwrap();
    let wrong_scope =
        UiRecordedServiceProposalOwnerPort::recorded_fixture(family::Portal, scope_id(9));
    let before_ack = compiler.census();
    assert_eq!(
        compiler.acknowledge_owner(&mut settlement, wrong_scope.acknowledge(publication)),
        Err(UiServiceProposalSettlementDenial::OwnerScopeMismatch)
    );
    assert_eq!(compiler.census(), before_ack);

    let wrong_digest_receipt = UiServiceProposalPublicationReceipt::recorded_foreign_fixture(
        publication.proposal(),
        publication.batch_digest().wrapping_add(1),
        publication.disposition(),
    );
    let correct_owner =
        UiRecordedServiceProposalOwnerPort::recorded_fixture(family::Portal, scope_id(1));
    assert_eq!(
        compiler.acknowledge_owner(
            &mut settlement,
            correct_owner.acknowledge(wrong_digest_receipt),
        ),
        Err(UiServiceProposalSettlementDenial::BatchDigestMismatch)
    );
    assert_eq!(compiler.census(), before_ack);
    compiler
        .acknowledge_owner(&mut settlement, correct_owner.acknowledge(publication))
        .unwrap();
    compiler.finish_settlement(settlement).unwrap();
    assert!(compiler.census().is_zero());
}

fn staged(
    identity: u64,
    families: &[family],
) -> (
    super::super::UiServiceProposalCompiler,
    super::super::UiServiceProposalStagedBatch,
) {
    let (mut compiler, reservation) = reserved(identity, families);
    let mut staging = compiler.begin_staging(reservation).unwrap();
    for (index, family) in families.iter().enumerate().rev() {
        stage_family(
            &mut compiler,
            &mut staging,
            identity,
            *family,
            index as u64 + 1,
        );
    }
    let proposal = super::super::UiServiceProposalIdentity::for_test(identity);
    compiler
        .advance_staging(
            &mut staging,
            UiServiceProposalStageReceipt::recorded_stage_fixture(
                proposal,
                UiServiceProposalStage::AssembleSuccessor,
                UiServiceProposalStageIssuer::ExistingPreparation,
            ),
        )
        .unwrap();
    if families.contains(&family::Focus) {
        compiler
            .advance_staging(
                &mut staging,
                UiServiceProposalStageReceipt::recorded_stage_fixture(
                    proposal,
                    UiServiceProposalStage::ResolveFocusAndReveal,
                    UiServiceProposalStageIssuer::FocusOwner {
                        reveal_refinement: None,
                    },
                ),
            )
            .unwrap();
    }
    if families.contains(&family::Motion) {
        compiler
            .advance_staging(
                &mut staging,
                UiServiceProposalStageReceipt::recorded_stage_fixture(
                    proposal,
                    UiServiceProposalStage::DeriveMotion,
                    UiServiceProposalStageIssuer::MotionOwner,
                ),
            )
            .unwrap();
    }
    let batch = compiler.finish_staging(staging).unwrap();
    (compiler, batch)
}

fn reserved(
    identity: u64,
    families: &[family],
) -> (
    super::super::UiServiceProposalCompiler,
    super::super::UiReservedServiceProposal,
) {
    let mut compiler = super::super::UiServiceProposalCompiler::new();
    let coherence = fixture_service_request_coherence(identity);
    let proposals = families
        .iter()
        .enumerate()
        .map(|(index, family)| {
            super::super::UiServiceFamilyProposal::recorded_fixture(
                *family,
                index as u64 + 1,
                1,
                1,
                1,
            )
        })
        .collect::<Vec<_>>();
    let participation = UiServiceFamilyParticipation::from_families(families).unwrap();
    let candidate = super::super::UiServiceProposalCandidate::for_test(
        identity,
        super::super::UiServiceProposalDemand::recorded_fixture(
            participation,
            u8::try_from(families.len()).unwrap(),
            u16::try_from(families.len()).unwrap(),
            u16::try_from(families.len()).unwrap(),
        ),
        coherence.clone(),
        proposals,
    );
    let support = families.iter().fold(
        crate::capability::UiRuntimeServiceSupport::none_installed(),
        |support, family| support.with_installed(*family),
    );
    let preflighted = compiler.preflight(candidate, &coherence, support).unwrap();
    let reservation = match compiler.reserve(preflighted).unwrap() {
        super::super::UiServiceProposalReservationOutcome::Reserved(reservation) => reservation,
        super::super::UiServiceProposalReservationOutcome::Coalesced { .. } => unreachable!(),
    };
    (compiler, reservation)
}

fn stage_family(
    compiler: &mut super::super::UiServiceProposalCompiler,
    staging: &mut super::super::UiServiceProposalStaging,
    identity: u64,
    family: family,
    scope: u64,
) {
    compiler
        .advance_staging(
            staging,
            UiServiceProposalStageReceipt::recorded_family_fixture(
                super::super::UiServiceProposalIdentity::for_test(identity),
                family,
                scope_id(scope),
                vec![
                    super::super::UiServiceProducedFactReference::recorded_fixture(
                        200 + scope,
                        family,
                        scope_id(scope),
                    ),
                ],
                vec![
                    super::super::UiServiceMountedWorkReference::recorded_fixture(
                        300 + scope,
                        family,
                        scope_id(scope),
                    ),
                ],
            ),
        )
        .unwrap();
}

#[allow(non_camel_case_types)]
type family = crate::capability::UiRuntimeServiceFamily;

fn scope_id(value: u64) -> UiServiceProposalOccupancyScopeIdentity {
    UiServiceProposalOccupancyScopeIdentity::for_test(value)
}

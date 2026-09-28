use super::{
    UiServiceProposalStageIssuer, UiServiceProposalStageReceipt, UiServiceProposalStagingDenial,
};

#[test]
fn owner_witnesses_stage_in_any_family_order_then_finish_in_fixed_order() {
    let (mut compiler, mut staging, proposal) = reserved_two_family_staging(40);
    let before = compiler.census();

    compiler
        .advance_staging(
            &mut staging,
            family_receipt(proposal, family::Focus, 2, 22, 32),
        )
        .unwrap();
    compiler
        .advance_staging(
            &mut staging,
            family_receipt(proposal, family::Portal, 1, 21, 31),
        )
        .unwrap();
    finish_owner_stages(
        &mut compiler,
        &mut staging,
        proposal,
        &[family::Portal, family::Focus],
        None,
    );
    let batch = compiler.finish_staging(staging).unwrap();

    assert_eq!(batch.identity(), proposal);
    assert_eq!(batch.fact_references()[0].diagnostic_value(), 21);
    assert_eq!(batch.fact_references()[1].diagnostic_value(), 22);
    assert_eq!(batch.mounted_work_references()[0].diagnostic_value(), 31);
    assert_eq!(batch.mounted_work_references()[1].diagnostic_value(), 32);
    assert_ne!(batch.digest(), 0);
    assert_eq!(batch.reveal_refinement(), None);
    assert_eq!(compiler.census().entries()[3].1, before.entries()[3].1 + 4);
}

#[test]
fn denied_witnesses_preserve_staging_and_census() {
    let (mut compiler, mut staging, proposal) = reserved_two_family_staging(50);

    for (receipt, denial) in [
        (
            family_receipt(
                super::super::UiServiceProposalIdentity::for_test(99),
                family::Portal,
                1,
                21,
                31,
            ),
            UiServiceProposalStagingDenial::ForeignProposal,
        ),
        (
            UiServiceProposalStageReceipt::recorded_stage_fixture(
                proposal,
                super::super::UiServiceProposalStage::AssembleSuccessor,
                UiServiceProposalStageIssuer::ExistingPreparation,
            ),
            UiServiceProposalStagingDenial::OutOfOrder {
                expected: super::super::UiServiceProposalStage::FamilyOwnedStaging,
                observed: super::super::UiServiceProposalStage::AssembleSuccessor,
            },
        ),
        (
            UiServiceProposalStageReceipt::recorded_family_fixture(
                proposal,
                family::Motion,
                scope(1),
                vec![fact(21, family::Motion, 1)],
                vec![work(31, family::Motion, 1)],
            ),
            UiServiceProposalStagingDenial::NonParticipatingFamily,
        ),
        (
            UiServiceProposalStageReceipt::recorded_family_fixture(
                proposal,
                family::Portal,
                scope(9),
                vec![fact(21, family::Portal, 9)],
                vec![work(31, family::Portal, 9)],
            ),
            UiServiceProposalStagingDenial::ScopeWidening,
        ),
        (
            UiServiceProposalStageReceipt::recorded_family_fixture(
                proposal,
                family::Portal,
                scope(1),
                vec![fact(21, family::Focus, 1)],
                vec![work(31, family::Portal, 1)],
            ),
            UiServiceProposalStagingDenial::ReferenceFamilyMismatch,
        ),
        (
            UiServiceProposalStageReceipt::recorded_family_fixture(
                proposal,
                family::Portal,
                scope(1),
                Vec::new(),
                vec![work(31, family::Portal, 1)],
            ),
            UiServiceProposalStagingDenial::ReferenceBudgetMismatch,
        ),
    ] {
        let before = compiler.census();
        assert_eq!(compiler.advance_staging(&mut staging, receipt), Err(denial));
        assert_eq!(compiler.census(), before);
    }

    compiler
        .advance_staging(
            &mut staging,
            family_receipt(proposal, family::Portal, 1, 21, 31),
        )
        .unwrap();
    let before_duplicate = compiler.census();
    assert_eq!(
        compiler.advance_staging(
            &mut staging,
            family_receipt(proposal, family::Portal, 1, 23, 33),
        ),
        Err(UiServiceProposalStagingDenial::DuplicateFamilyWitness)
    );
    assert_eq!(compiler.census(), before_duplicate);
}

#[test]
fn fixed_non_family_stages_reject_wrong_issuer_and_repeat_refinement() {
    let (mut compiler, mut staging, proposal) = reserved_one_family_staging(60);
    compiler
        .advance_staging(
            &mut staging,
            family_receipt(proposal, family::Portal, 1, 21, 31),
        )
        .unwrap();
    let before = compiler.census();
    assert_eq!(
        compiler.advance_staging(
            &mut staging,
            UiServiceProposalStageReceipt::recorded_stage_fixture(
                proposal,
                super::super::UiServiceProposalStage::AssembleSuccessor,
                UiServiceProposalStageIssuer::MotionOwner,
            ),
        ),
        Err(UiServiceProposalStagingDenial::WrongIssuer)
    );
    assert_eq!(compiler.census(), before);

    finish_owner_stages(
        &mut compiler,
        &mut staging,
        proposal,
        &[family::Portal],
        None,
    );
    assert_eq!(
        compiler.advance_staging(
            &mut staging,
            UiServiceProposalStageReceipt::recorded_stage_fixture(
                proposal,
                super::super::UiServiceProposalStage::ResolveFocusAndReveal,
                UiServiceProposalStageIssuer::FocusOwner {
                    reveal_refinement: None,
                },
            ),
        ),
        Err(UiServiceProposalStagingDenial::AlreadyComplete)
    );
}

#[test]
fn incomplete_finish_returns_the_live_transaction_for_exact_zero_teardown() {
    let (mut compiler, mut staging, proposal) = reserved_one_family_staging(61);
    compiler
        .advance_staging(
            &mut staging,
            family_receipt(proposal, family::Portal, 1, 21, 31),
        )
        .unwrap();
    let before = compiler.census();
    let (staging, denial) = compiler.finish_staging(staging).unwrap_err();
    assert_eq!(
        denial,
        UiServiceProposalStagingDenial::Incomplete {
            expected: super::super::UiServiceProposalStage::AssembleSuccessor,
        }
    );
    assert_eq!(compiler.census(), before);
    let mut teardown = compiler.cancel_staging(staging);
    let owner = super::super::UiRecordedServiceProposalOwnerPort::recorded_fixture(
        family::Portal,
        scope(1),
    );
    compiler
        .acknowledge_terminal_owner(
            &mut teardown,
            owner.terminal_outcome(
                proposal,
                super::super::UiServiceProposalTerminalReason::CancelledBeforePublication,
            ),
        )
        .unwrap();
    compiler.finish_teardown(teardown).unwrap();
    assert!(compiler.census().is_zero());
}

fn finish_owner_stages(
    compiler: &mut super::super::UiServiceProposalCompiler,
    staging: &mut super::UiServiceProposalStaging,
    proposal: super::super::UiServiceProposalIdentity,
    families: &[family],
    reveal_refinement: Option<super::super::super::UiServiceProposalOccupancyScopeIdentity>,
) {
    compiler
        .advance_staging(
            staging,
            UiServiceProposalStageReceipt::recorded_stage_fixture(
                proposal,
                super::super::UiServiceProposalStage::AssembleSuccessor,
                UiServiceProposalStageIssuer::ExistingPreparation,
            ),
        )
        .unwrap();
    if families.contains(&family::Focus) {
        compiler
            .advance_staging(
                staging,
                UiServiceProposalStageReceipt::recorded_stage_fixture(
                    proposal,
                    super::super::UiServiceProposalStage::ResolveFocusAndReveal,
                    UiServiceProposalStageIssuer::FocusOwner { reveal_refinement },
                ),
            )
            .unwrap();
    }
    if families.contains(&family::Motion) {
        compiler
            .advance_staging(
                staging,
                UiServiceProposalStageReceipt::recorded_stage_fixture(
                    proposal,
                    super::super::UiServiceProposalStage::DeriveMotion,
                    UiServiceProposalStageIssuer::MotionOwner,
                ),
            )
            .unwrap();
    }
}

pub(super) fn reserved_two_family_staging(
    identity: u64,
) -> (
    super::super::UiServiceProposalCompiler,
    super::UiServiceProposalStaging,
    super::super::UiServiceProposalIdentity,
) {
    reserved_staging(identity, &[family::Portal, family::Focus])
}

fn reserved_one_family_staging(
    identity: u64,
) -> (
    super::super::UiServiceProposalCompiler,
    super::UiServiceProposalStaging,
    super::super::UiServiceProposalIdentity,
) {
    reserved_staging(identity, &[family::Portal])
}

pub(super) fn reserved_staging(
    identity: u64,
    families: &[family],
) -> (
    super::super::UiServiceProposalCompiler,
    super::UiServiceProposalStaging,
    super::super::UiServiceProposalIdentity,
) {
    let mut compiler = super::super::UiServiceProposalCompiler::new();
    let coherence = super::super::super::fixture_service_request_coherence(identity);
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
    let demand = super::super::UiServiceProposalDemand::recorded_fixture(
        super::super::super::UiServiceFamilyParticipation::from_families(families).unwrap(),
        u8::try_from(families.len()).unwrap(),
        u16::try_from(families.len()).unwrap(),
        u16::try_from(families.len()).unwrap(),
    );
    let candidate = super::super::UiServiceProposalCandidate::for_test(
        identity,
        demand,
        coherence.clone(),
        proposals,
    );
    let mut support = crate::capability::UiRuntimeServiceSupport::none_installed();
    for family in families {
        support = support.with_installed(*family);
    }
    let preflighted = compiler.preflight(candidate, &coherence, support).unwrap();
    let reservation = match compiler.reserve(preflighted).unwrap() {
        super::super::UiServiceProposalReservationOutcome::Reserved(reservation) => reservation,
        super::super::UiServiceProposalReservationOutcome::Coalesced { .. } => unreachable!(),
    };
    let proposal = reservation.identity();
    let staging = compiler.begin_staging(reservation).unwrap();
    (compiler, staging, proposal)
}

#[allow(non_camel_case_types)]
pub(super) type family = crate::capability::UiRuntimeServiceFamily;

pub(super) fn family_receipt(
    proposal: super::super::UiServiceProposalIdentity,
    family: family,
    scope_value: u64,
    fact_identity: u64,
    work_identity: u64,
) -> UiServiceProposalStageReceipt {
    UiServiceProposalStageReceipt::recorded_family_fixture(
        proposal,
        family,
        scope(scope_value),
        vec![fact(fact_identity, family, scope_value)],
        vec![work(work_identity, family, scope_value)],
    )
}

pub(super) fn scope(value: u64) -> super::super::super::UiServiceProposalOccupancyScopeIdentity {
    super::super::super::UiServiceProposalOccupancyScopeIdentity::for_test(value)
}

fn fact(
    identity: u64,
    family: family,
    scope_value: u64,
) -> super::super::UiServiceProducedFactReference {
    super::super::UiServiceProducedFactReference::recorded_fixture(
        identity,
        family,
        scope(scope_value),
    )
}

fn work(
    identity: u64,
    family: family,
    scope_value: u64,
) -> super::super::UiServiceMountedWorkReference {
    super::super::UiServiceMountedWorkReference::recorded_fixture(
        identity,
        family,
        scope(scope_value),
    )
}

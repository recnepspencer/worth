use super::{
    UiReservedServiceProposal, UiServiceProposalCompiler, UiServiceProposalReservationOutcome,
    UiServiceProposalStage,
};

#[test]
fn compiler_begins_empty_and_exposes_fixed_semantic_stages() {
    let compiler = UiServiceProposalCompiler::new();
    assert!(compiler.census().is_zero());
    assert_eq!(UiServiceProposalStage::ORDER.len(), 7);
}

#[test]
fn reservation_and_before_effect_cancellation_are_census_atomic() {
    let mut compiler = UiServiceProposalCompiler::new();
    let coherence = super::super::fixture_service_request_coherence(11);
    let reserved = reserve(
        &mut compiler,
        &coherence,
        11,
        super::super::UiServiceProposalConflictPolicy::RejectOccupied,
    );
    assert_eq!(
        compiler.census().entries(),
        [
            ("proposals", 1),
            ("occupancy_leases", 1),
            ("cancellation_records", 1),
            ("stage_receipts", 0),
        ]
    );
    assert_eq!(compiler.live_occupancy_count(), 1);
    assert_eq!(compiler.live_cancellation_count(), 1);

    let receipt = compiler.cancel_before_effect(reserved).unwrap();
    assert_eq!(receipt.released_leases(), 1);
    assert!(compiler.census().is_zero());
    assert_eq!(compiler.live_occupancy_count(), 0);
    assert_eq!(compiler.live_cancellation_count(), 0);
}

#[test]
fn occupied_denial_changes_no_resource_and_supersession_is_aba_safe() {
    let mut compiler = UiServiceProposalCompiler::new();
    let coherence = super::super::fixture_service_request_coherence(12);
    let incumbent = reserve(
        &mut compiler,
        &coherence,
        12,
        super::super::UiServiceProposalConflictPolicy::RejectOccupied,
    );
    let before = compiler.census();
    let occupied = preflight(
        &mut compiler,
        &coherence,
        13,
        super::super::UiServiceProposalConflictPolicy::RejectOccupied,
    );
    assert!(matches!(
        compiler.reserve(occupied),
        Err(super::UiServiceProposalReservationDenial::Occupancy(
            super::super::UiServiceProposalOccupancyDenial::Occupied(_)
        ))
    ));
    assert_eq!(compiler.census(), before);

    let successor = reserve(
        &mut compiler,
        &coherence,
        14,
        super::super::UiServiceProposalConflictPolicy::SupersedeBeforeEffect,
    );
    assert_eq!(
        successor.displacement().unwrap().disposition(),
        super::super::UiServiceProposalConflictDisposition::Superseded
    );
    assert!(successor.leases()[0].slot_generation() > incumbent.leases()[0].slot_generation());
    assert_eq!(compiler.census(), before);
    assert!(compiler.cancel_before_effect(incumbent).is_err());
    assert_eq!(compiler.census(), before);
    compiler.cancel_before_effect(successor).unwrap();
    assert!(compiler.census().is_zero());
}

#[test]
fn exact_coalescing_reuses_only_an_equivalent_complete_occupancy_set() {
    let mut compiler = UiServiceProposalCompiler::new();
    let coherence = super::super::fixture_service_request_coherence(15);
    let incumbent = reserve(
        &mut compiler,
        &coherence,
        15,
        super::super::UiServiceProposalConflictPolicy::RejectOccupied,
    );
    let before = compiler.census();
    let coalescing = preflight(
        &mut compiler,
        &coherence,
        16,
        super::super::UiServiceProposalConflictPolicy::CoalesceExact,
    );
    assert!(matches!(
        compiler.reserve(coalescing),
        Ok(UiServiceProposalReservationOutcome::Coalesced { incumbent: found })
            if found == incumbent.identity()
    ));
    assert_eq!(compiler.census(), before);
    compiler.cancel_before_effect(incumbent).unwrap();
    assert!(compiler.census().is_zero());
}

pub(super) fn reserve(
    compiler: &mut UiServiceProposalCompiler,
    coherence: &super::super::UiServiceRequestCoherence,
    identity: u64,
    policy: super::super::UiServiceProposalConflictPolicy,
) -> UiReservedServiceProposal {
    let preflighted = preflight(compiler, coherence, identity, policy);
    match compiler.reserve(preflighted).unwrap() {
        UiServiceProposalReservationOutcome::Reserved(reserved) => reserved,
        UiServiceProposalReservationOutcome::Coalesced { .. } => {
            panic!("fixture expected a new reservation")
        }
    }
}

fn preflight(
    compiler: &mut UiServiceProposalCompiler,
    coherence: &super::super::UiServiceRequestCoherence,
    identity: u64,
    policy: super::super::UiServiceProposalConflictPolicy,
) -> super::UiPreflightedServiceProposal {
    let participation = super::super::fixture_service_family_participation(1);
    let family = super::UiServiceFamilyProposal::recorded_fixture(
        crate::capability::UiRuntimeServiceFamily::Portal,
        1,
        1,
        1,
        1,
    )
    .with_conflict_policy(policy);
    let candidate = super::UiServiceProposalCandidate::for_test(
        identity,
        super::UiServiceProposalDemand::recorded_fixture(participation, 1, 1, 1),
        coherence.clone(),
        vec![family],
    );
    compiler
        .preflight(
            candidate,
            coherence,
            crate::capability::UiRuntimeServiceSupport::none_installed()
                .with_installed(crate::capability::UiRuntimeServiceFamily::Portal),
        )
        .unwrap()
}

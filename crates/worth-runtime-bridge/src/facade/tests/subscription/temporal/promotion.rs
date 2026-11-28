use std::num::NonZeroUsize;

use worth_proof::{ConditionalSourceObservationOwner, TransitionOutcome};
use worth_signal::facade::{
    ClockAdvanceOrdinal, ClockAdvanceRequest, ClockDomain, ClockTick, SignalAspectLoweringOwner,
    SignalGraph, SignalRuntime, TemporalCondition, TemporalWakeId,
};

use crate::facade::tests::runtime;
use crate::facade::{
    BridgeRuntimePolicy, BridgeTemporalBasisDenial, BridgeTemporalSignalBasis,
    BridgeTemporalTruthViewBasis,
};
use crate::truth_identity_fixtures::{
    truth_branch_fixture, truth_commit_fixture, truth_snapshot_fixture,
};

#[test]
fn temporal_basis_requires_an_exact_wake_from_signal_promotion() {
    let owner = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let mut signal = SignalRuntime::build_for::<()>(graph);
    let basis = signal
        .observe_signal_branch_basis(signal.current_branch())
        .unwrap();
    signal.owner_component_services().unwrap();
    let source = ConditionalSourceObservationOwner::fresh();
    let service = signal
        .issue_conditional_execution_service(&basis, &owner, &source.authority())
        .unwrap();
    let maximum = NonZeroUsize::new(1).unwrap();
    let partition = service.admit_temporal_partition(maximum).unwrap();
    let tick = ClockTick::new(3);
    let domain = ClockDomain::MonotonicExecution;
    let scheduled = partition
        .schedule_temporal_wake(TemporalCondition::at_or_after(tick), tick)
        .unwrap();
    let (_, promotion) = partition
        .advance_clock_and_promote_due_temporal_wakes_ready_bounded(
            ClockAdvanceRequest::new(domain, tick),
            maximum,
        )
        .unwrap();

    let bridge = runtime(BridgeRuntimePolicy::default());
    let branch = truth_branch_fixture("temporal-promotion-branch");
    let truth = BridgeTemporalTruthViewBasis::authoritative(
        branch.clone(),
        truth_commit_fixture("temporal-promotion-commit"),
        truth_snapshot_fixture("temporal-promotion-snapshot"),
    );
    let clock =
        BridgeTemporalSignalBasis::new(branch, domain, tick, ClockAdvanceOrdinal::new(1), None);
    assert!(matches!(
        bridge.admit_promoted_temporal_basis(
            truth.clone(),
            clock.clone(),
            &promotion,
            TemporalWakeId::new(scheduled.id().get() + 1),
        ),
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::MissingWakeEvidence)
    ));
    let outcome = bridge.admit_promoted_temporal_basis(truth, clock, &promotion, scheduled.id());
    let TransitionOutcome::Success(admitted) = outcome else {
        panic!("the promoted wake should admit: {outcome:?}");
    };
    let evidence = admitted.wake_evidence().evidence();
    assert_eq!(evidence.wake_id(), scheduled.id());
    assert_eq!(evidence.wake_tick(), tick);
    assert_eq!(
        evidence.wake_ready_ordinal(),
        promotion.promotion().ready_wakes()[0].ready_ordinal()
    );
}

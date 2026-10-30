use super::super::locality_scale::{LocalityScaleTuple, RestorePosture, SparseFanoutAxis};
use super::{
    FinancialLocalityAction, FinancialLocalityDefinition, FinancialLocalityScenario, LocalityScope,
};

#[test]
fn sparse_generator_owns_every_exact_output_and_preserves_depth_sixteen_chain() {
    for axis in [
        SparseFanoutAxis::IndexDisjoint,
        SparseFanoutAxis::QueriedRejecting,
        SparseFanoutAxis::RejectedDescendants,
    ] {
        let definition = FinancialLocalityDefinition::generate(
            41,
            LocalityScaleTuple::SparseBookFanout {
                total_outputs: 64,
                axis,
            },
        );
        definition.validate_generator_invariants();
        assert_eq!(definition.seed(), 41);
        assert_eq!(
            definition.outputs()[15].subscriptions[0].upstream.ordinal(),
            14
        );
    }
}

#[test]
fn partition_generator_varies_regions_memberships_and_instruments_without_padding() {
    let definition = FinancialLocalityDefinition::generate(
        41,
        LocalityScaleTuple::PartitionedCurveUniverse {
            regions: 16,
            matching_memberships: 4,
            instruments_per_matching_region: 8,
        },
    );
    definition.validate_generator_invariants();
    assert_eq!(
        definition.scenario(),
        FinancialLocalityScenario::PartitionedCurveUniverse
    );
    assert_eq!(definition.outputs().len(), 1 + (1 + 8) + 3 + 2 * 15 + 4);
    assert_eq!(definition.mutation().scope.unwrap().region, 0);
    assert_eq!(
        definition.workload().observation_targets().len(),
        8 + 3 + 15 + 3
    );
}

#[test]
fn scheduled_partition_tuple_preserves_independent_r_m_and_i_axes() {
    let definition = FinancialLocalityDefinition::generate(
        41,
        LocalityScaleTuple::PartitionedCurveUniverse {
            regions: 1_024,
            matching_memberships: 256,
            instruments_per_matching_region: 32,
        },
    );

    definition.validate_generator_invariants();
    assert_eq!(definition.outputs().len(), 2 * 1_024 + 256 + 32 + 3);
    assert_eq!(
        definition.workload().observation_targets().len(),
        32 + 255 + 1_023 + 3
    );
}

#[test]
fn scheduled_sparse_rejection_contracts_never_wrap_into_the_queried_detail() {
    let definition = FinancialLocalityDefinition::generate(
        41,
        LocalityScaleTuple::SparseBookFanout {
            total_outputs: 100_000,
            axis: SparseFanoutAxis::QueriedRejecting,
        },
    );
    let queried = Some(LocalityScope::detail(0, 0));

    definition.validate_generator_invariants();
    assert!(definition.outputs()[16..].iter().all(|output| {
        output.subscriptions[0].edge_scope == queried
            && output.subscriptions[0].eligibility_scope != queried
    }));
}

#[test]
fn churn_and_restore_cases_freeze_every_lifecycle_action_in_order() {
    let churn = FinancialLocalityDefinition::generate(
        41,
        LocalityScaleTuple::PortfolioDependencyChurn {
            rounds: 8,
            canonical_seeds: 1,
        },
    );
    churn.validate_generator_invariants();
    let churn_actions = churn.action_traces()[0].actions();
    assert_eq!(churn_actions.len(), 64);
    for (round, actions) in churn_actions.as_chunks::<8>().0.iter().enumerate() {
        assert!(matches!(
            actions,
            [
                FinancialLocalityAction::CommitFactor(_),
                FinancialLocalityAction::StagePreRewireWork { round: staged_round, .. },
                FinancialLocalityAction::AcceptedOwnerMove { round: owner_round, .. },
                FinancialLocalityAction::RejectStaleWork { round: stale_round, .. },
                FinancialLocalityAction::AcceptedDependencyRemoval { round: removal_round, .. },
                FinancialLocalityAction::AcceptedDependencyRecreation { round: recreation_round, .. },
                FinancialLocalityAction::RejectedCycle { round: cycle_round, .. },
                FinancialLocalityAction::CommitFactor(_),
            ] if usize::from(*staged_round) == round
                && usize::from(*owner_round) == round
                && usize::from(*stale_round) == round
                && usize::from(*removal_round) == round
                && usize::from(*recreation_round) == round
                && usize::from(*cycle_round) == round
        ));
    }

    let restore = FinancialLocalityDefinition::generate(
        41,
        LocalityScaleTuple::BranchRestoreLocalityReplay {
            posture: RestorePosture::Narrow,
            total_outputs: 0,
            canonical_seeds: 1,
        },
    );
    restore.validate_generator_invariants();
    assert!(matches!(
        &restore.action_traces()[0].actions()[1..],
        [
            FinancialLocalityAction::StageSourceRecompute { .. },
            FinancialLocalityAction::CaptureBranch { branch_ordinal: 1 },
            FinancialLocalityAction::CaptureCheckpoint {
                checkpoint_ordinal: 1
            },
            FinancialLocalityAction::DestroyDerivedState {
                destruction_ordinal: 1
            },
            FinancialLocalityAction::ReadmitFreshRuntime { runtime_epoch: 2 },
            FinancialLocalityAction::ReplayCanonicalTrace { replay_ordinal: 1 },
            FinancialLocalityAction::DeterministicRerun { rerun_ordinal: 1 },
        ]
    ));
}

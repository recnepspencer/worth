use worth_store_formal_models::{
    map_checkpoint_selection, map_failed_wal_fence, map_recovery_completion, map_redo_execution,
    map_redo_generation_denial, DurabilityRecoveryAction,
};
use worth_store_physical_backend::{
    BackendDurabilityProfile, SimulatedStrictDurableProfile, WalAppendFailureObservation,
    WalAppendObservationScope, WalAppendReceipt, WalDurabilityBarrier, WalDurabilityObservation,
};
use worth_store_test_support::harness::{
    physical_residency::{canonical_physical_mutation_acknowledgment, PhysicalResidencyStoreWorld},
    recovery::{closeout, deterministic_checkpoint_plus_tail_source},
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange, WalSegmentGeneration, WalSegmentId};

use super::map_physical_mutation_acknowledgment;
use super::redo_fixture;

pub(in crate::courtroom::protocol_models) fn execute_ordinary_durability_recovery(
) -> Vec<DurabilityRecoveryAction> {
    execute_ordinary_durability_recovery_traces()
        .into_iter()
        .flatten()
        .collect()
}

pub(in crate::courtroom::protocol_models) fn execute_ordinary_durability_recovery_traces(
) -> Vec<Vec<DurabilityRecoveryAction>> {
    let completed_mutation = execute_canonical_physical_mutation();
    let data_durable_prefix = completed_mutation[..10].to_vec();
    let checkpoint = execute_checkpoint();
    let stable_setup = data_durable_prefix
        .iter()
        .copied()
        .chain(checkpoint.iter().copied())
        .collect::<Vec<_>>();
    let redo = execute_redo_traces();
    let completion = map_recovery_completion(&closeout::recovery_completion());
    let reopen = DurabilityRecoveryAction::Reopen;

    let applied_recovery = stable_setup
        .iter()
        .copied()
        .chain(redo[0].iter().copied())
        .chain(completion)
        .collect();
    let crash_reopen = stable_setup
        .iter()
        .copied()
        .chain([DurabilityRecoveryAction::Crash, reopen])
        .collect();
    let skipped_recovery = stable_setup
        .iter()
        .copied()
        .chain(redo[1].iter().copied())
        .collect();
    let rejected_generation = stable_setup
        .into_iter()
        .chain(redo[2].iter().copied())
        .collect();
    vec![
        completed_mutation,
        applied_recovery,
        crash_reopen,
        skipped_recovery,
        rejected_generation,
    ]
}

pub(in crate::courtroom::protocol_models) fn replay_acknowledgment_ordering_guard(
    seed: u64,
) -> Vec<DurabilityRecoveryAction> {
    let scope = WalAppendObservationScope::new(
        WalSegmentId::new(seed.max(1)).unwrap(),
        WalSegmentGeneration::new(1).unwrap(),
        WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11)).unwrap(),
        format!("failed-wal-fence-{seed}"),
        64,
    )
    .unwrap();
    let receipt = WalAppendReceipt::<SimulatedStrictDurableProfile>::from_certification_observation(
        scope,
        64,
        SimulatedStrictDurableProfile::REQUIRED_BARRIERS,
        Some(WalAppendFailureObservation::BarrierFailed(
            WalDurabilityBarrier::SimulatedDurableCommit,
        )),
    );
    let denial = WalDurabilityObservation::from_append_receipt(receipt).unwrap_err();
    let actions = map_failed_wal_fence(&denial).unwrap().to_vec();
    assert!(!actions.contains(&DurabilityRecoveryAction::PhysicalMutationAcknowledged));
    actions
}

fn execute_canonical_physical_mutation() -> Vec<DurabilityRecoveryAction> {
    let world = PhysicalResidencyStoreWorld::initialize("protocol-physical-ack")
        .expect("canonical physical mutation world");
    let acknowledgment = canonical_physical_mutation_acknowledgment(
        &world,
        [0x7a; 32],
        b"protocol-physical-mutation",
    );
    let actions = map_physical_mutation_acknowledgment(&acknowledgment).to_vec();
    let _closed = world.close();
    actions
}

fn execute_checkpoint() -> Vec<DurabilityRecoveryAction> {
    let source = deterministic_checkpoint_plus_tail_source();
    vec![map_checkpoint_selection(
        source.checkpoint().expect("fixture selects a checkpoint"),
    )]
}

fn execute_redo_traces() -> [Vec<DurabilityRecoveryAction>; 3] {
    let applied = redo_fixture::applied_plan();
    let skipped = redo_fixture::skipped_plan();
    [
        map_redo_execution(&applied).to_vec(),
        map_redo_execution(&skipped).to_vec(),
        vec![
            DurabilityRecoveryAction::RecoveryReplayRequired,
            execute_redo_generation_denial(),
        ],
    ]
}

fn execute_redo_generation_denial() -> DurabilityRecoveryAction {
    let denial = redo_fixture::generation_denial();
    map_redo_generation_denial(&denial).unwrap()
}

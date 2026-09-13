use worth_signal::facade::{SignalGraph, SignalRuntime};

use super::super::state::{BranchKey, ModelObservation, ModelWorld};
use super::super::transition::{ModelDenial, ModelResult, ModelSuccess};

pub(super) const ORACLE_SEED: u64 = 0x9e17_0112;

type Runtime = SignalRuntime<(), (), (), (), ()>;

pub(super) fn runtime() -> Runtime {
    SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

pub(super) fn model_observation(result: ModelResult, context: &str) -> ModelObservation {
    match result {
        ModelResult::Success(ModelSuccess::Observation(observation)) => observation,
        other => panic!(
            "seed {ORACLE_SEED:#x}: expected model observation after {context}, got {other:?}"
        ),
    }
}

pub(super) fn model_movement(result: ModelResult, context: &str) -> ModelObservation {
    match result {
        ModelResult::Success(ModelSuccess::Advance(observation))
        | ModelResult::Success(ModelSuccess::Restore(observation)) => observation,
        ModelResult::Success(ModelSuccess::Capture { observation, .. }) => observation,
        other => {
            panic!("seed {ORACLE_SEED:#x}: expected model movement after {context}, got {other:?}")
        }
    }
}

pub(super) fn model_fork(result: ModelResult, context: &str) -> ModelObservation {
    match result {
        ModelResult::Success(ModelSuccess::Fork(observation)) => observation,
        other => {
            panic!("seed {ORACLE_SEED:#x}: expected model fork after {context}, got {other:?}")
        }
    }
}

pub(super) fn model_denial(result: ModelResult, context: &str) -> ModelDenial {
    match result {
        ModelResult::Denied(denial) => denial,
        other => {
            panic!("seed {ORACLE_SEED:#x}: expected model denial after {context}, got {other:?}")
        }
    }
}

pub(super) fn assert_denial(expected: ModelDenial, actual: ModelDenial, context: &str) {
    assert_eq!(
        expected, actual,
        "seed {ORACLE_SEED:#x}: wrong denial reason after {context}"
    );
}

pub(super) fn model_lease(world: &ModelWorld, context: &str) -> u64 {
    world
        .leases
        .keys()
        .next_back()
        .copied()
        .unwrap_or_else(|| panic!("seed {ORACLE_SEED:#x}: model lost lease after {context}"))
}

pub(super) fn current_model(world: &ModelWorld, branch: BranchKey) -> ModelObservation {
    world
        .branch(branch)
        .unwrap_or_else(|| panic!("seed {ORACLE_SEED:#x}: model lost branch {branch}"))
        .observation
        .clone()
}

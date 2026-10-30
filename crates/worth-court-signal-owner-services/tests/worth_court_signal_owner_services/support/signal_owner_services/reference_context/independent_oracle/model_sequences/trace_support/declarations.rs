pub(crate) use {worth_signal::facade::SignalGraph, worth_signal::facade::SignalRuntime};

pub(crate) use {
    crate::signal_owner_services::reference_context::independent_oracle::state::BranchKey,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelObservation,
    crate::signal_owner_services::reference_context::independent_oracle::state::ModelWorld,
};
pub(crate) use {
    crate::signal_owner_services::reference_context::independent_oracle::transition::ModelDenial,
    crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult,
    crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess,
};

pub(crate) const ORACLE_SEED: u64 = 0x9e17_0112;

pub(crate) type Runtime = SignalRuntime<(), (), (), (), ()>;

pub(crate) fn runtime() -> Runtime {
    worth_signal::facade::SignalRuntime::builder(worth_signal::facade::SignalGraph::new())
        .with_kernel_defaults()
        .build()
}

pub(crate) fn model_observation(result: ModelResult, context: &str) -> ModelObservation {
    match result {
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Observation(observation)) => observation,
        other => panic!(
            "seed {ORACLE_SEED:#x}: expected model observation after {context}, got {other:?}"
        ),
    }
}

pub(crate) fn model_movement(result: ModelResult, context: &str) -> ModelObservation {
    match result {
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Advance(observation))
        | crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Restore(observation)) => observation,
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Capture { observation, .. }) => observation,
        other => {
            panic!("seed {ORACLE_SEED:#x}: expected model movement after {context}, got {other:?}")
        }
    }
}

pub(crate) fn model_fork(result: ModelResult, context: &str) -> ModelObservation {
    match result {
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Success(crate::signal_owner_services::reference_context::independent_oracle::transition::ModelSuccess::Fork(observation)) => observation,
        other => {
            panic!("seed {ORACLE_SEED:#x}: expected model fork after {context}, got {other:?}")
        }
    }
}

pub(crate) fn model_denial(result: ModelResult, context: &str) -> ModelDenial {
    match result {
        crate::signal_owner_services::reference_context::independent_oracle::transition::ModelResult::Denied(denial) => denial,
        other => {
            panic!("seed {ORACLE_SEED:#x}: expected model denial after {context}, got {other:?}")
        }
    }
}

pub(crate) fn assert_denial(expected: ModelDenial, actual: ModelDenial, context: &str) {
    assert_eq!(
        expected, actual,
        "seed {ORACLE_SEED:#x}: wrong denial reason after {context}"
    );
}

pub(crate) fn model_lease(world: &ModelWorld, context: &str) -> u64 {
    world
        .leases
        .keys()
        .next_back()
        .copied()
        .unwrap_or_else(|| panic!("seed {ORACLE_SEED:#x}: model lost lease after {context}"))
}

pub(crate) fn current_model(world: &ModelWorld, branch: BranchKey) -> ModelObservation {
    world
        .branch(branch)
        .unwrap_or_else(|| panic!("seed {ORACLE_SEED:#x}: model lost branch {branch}"))
        .observation
        .clone()
}

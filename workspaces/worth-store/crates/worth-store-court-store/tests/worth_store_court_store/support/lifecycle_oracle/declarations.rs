pub(crate) use {
    worth_store::physical_runtime::LifecycleGeneration,
    worth_store::physical_runtime::ObservationError,
    worth_store::physical_runtime::RuntimeIdentity,
    worth_store::physical_runtime::RuntimeObservation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LifecycleAction {
    Close,
    Abort,
    Panic,
    UnexpectedDrop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OracleLifecyclePhase {
    Admitted,
    Closed,
    Aborted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OracleObservationOutcome {
    Closed,
    Stale,
}

pub(crate) struct LifecycleTransition {
    pub(crate) source: OracleLifecyclePhase,
    pub(crate) action: LifecycleAction,
    pub(crate) terminal: OracleLifecyclePhase,
    pub(crate) observation: OracleObservationOutcome,
}

pub(crate) const LIFECYCLE_TRANSITIONS: [LifecycleTransition; 4] = [
    LifecycleTransition {
        source: crate::lifecycle_oracle::OracleLifecyclePhase::Admitted,
        action: crate::lifecycle_oracle::LifecycleAction::Close,
        terminal: crate::lifecycle_oracle::OracleLifecyclePhase::Closed,
        observation: crate::lifecycle_oracle::OracleObservationOutcome::Closed,
    },
    LifecycleTransition {
        source: crate::lifecycle_oracle::OracleLifecyclePhase::Admitted,
        action: crate::lifecycle_oracle::LifecycleAction::Abort,
        terminal: crate::lifecycle_oracle::OracleLifecyclePhase::Aborted,
        observation: crate::lifecycle_oracle::OracleObservationOutcome::Stale,
    },
    LifecycleTransition {
        source: crate::lifecycle_oracle::OracleLifecyclePhase::Admitted,
        action: crate::lifecycle_oracle::LifecycleAction::UnexpectedDrop,
        terminal: crate::lifecycle_oracle::OracleLifecyclePhase::Aborted,
        observation: crate::lifecycle_oracle::OracleObservationOutcome::Stale,
    },
    LifecycleTransition {
        source: crate::lifecycle_oracle::OracleLifecyclePhase::Admitted,
        action: crate::lifecycle_oracle::LifecycleAction::Panic,
        terminal: crate::lifecycle_oracle::OracleLifecyclePhase::Aborted,
        observation: crate::lifecycle_oracle::OracleObservationOutcome::Stale,
    },
];

pub(crate) fn assert_terminal_observation(
    action: LifecycleAction,
    outcome: Result<RuntimeObservation, ObservationError>,
    runtime_identity: RuntimeIdentity,
    admitted_generation: LifecycleGeneration,
) {
    let transition = LIFECYCLE_TRANSITIONS
        .iter()
        .find(|transition| {
            transition.source == crate::lifecycle_oracle::OracleLifecyclePhase::Admitted
                && transition.action == action
        })
        .expect("the independent C.3 lifecycle table must cover every exercised action");
    let expected_terminal_generation = admitted_generation.get() + 2;

    match (transition.terminal, transition.observation, outcome) {
        (
            crate::lifecycle_oracle::OracleLifecyclePhase::Closed,
            crate::lifecycle_oracle::OracleObservationOutcome::Closed,
            Err(worth_store::physical_runtime::ObservationError::Closed {
                runtime_identity: observed_identity,
                closed_generation,
            }),
        ) => {
            assert_eq!(observed_identity, runtime_identity);
            assert_eq!(closed_generation.get(), expected_terminal_generation);
        }
        (
            crate::lifecycle_oracle::OracleLifecyclePhase::Aborted,
            crate::lifecycle_oracle::OracleObservationOutcome::Stale,
            Err(worth_store::physical_runtime::ObservationError::Stale {
                runtime_identity: observed_identity,
                observed_generation,
                current_generation,
            }),
        ) => {
            assert_eq!(observed_identity, runtime_identity);
            assert_eq!(observed_generation, admitted_generation);
            assert_eq!(current_generation.get(), expected_terminal_generation);
        }
        (terminal, observation, outcome) => {
            panic!("expected {terminal:?}/{observation:?} after {action:?}, received {outcome:?}")
        }
    }
}

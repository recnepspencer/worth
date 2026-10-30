pub(crate) use worth_store::physical_runtime::ProcessRuntimeCounterSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PressureOutcome {
    pub(crate) admission_attempts: u64,
    pub(crate) admitted_incarnations: u64,
    pub(crate) admission_denials: u64,
    pub(crate) admission_cancellations: u64,
    pub(crate) admission_panics_before_return: u64,
    pub(crate) observation_acquisitions: u64,
    pub(crate) active_observations: u64,
    pub(crate) lifecycle_observations: u64,
    pub(crate) capability_observations: u64,
    pub(crate) explicit_closes: u64,
    pub(crate) explicit_aborts: u64,
    pub(crate) panic_terminations: u64,
    pub(crate) unexpected_drops: u64,
}

impl PressureOutcome {
    pub(crate) const EXPECTED: Self = Self {
        admission_attempts: 16,
        admitted_incarnations: 13,
        admission_denials: 2,
        admission_cancellations: 1,
        admission_panics_before_return: 0,
        observation_acquisitions: 11,
        active_observations: 0,
        lifecycle_observations: 23,
        capability_observations: 56,
        explicit_closes: 8,
        explicit_aborts: 3,
        panic_terminations: 1,
        unexpected_drops: 1,
    };

    pub(crate) fn between(
        before: ProcessRuntimeCounterSnapshot,
        after: ProcessRuntimeCounterSnapshot,
    ) -> Self {
        assert_eq!(after.physical_owner_count(), 0);
        assert_eq!(after.physical_operation_attempts(), 0);
        assert_eq!(after.publication_attempts(), 0);
        assert_eq!(after.media_operations(), 0);
        Self {
            admission_attempts: after.admission_attempts() - before.admission_attempts(),
            admitted_incarnations: after.admitted_incarnations() - before.admitted_incarnations(),
            admission_denials: after.admission_denials() - before.admission_denials(),
            admission_cancellations: after.admission_cancellations()
                - before.admission_cancellations(),
            admission_panics_before_return: after.admission_panics_before_return()
                - before.admission_panics_before_return(),
            observation_acquisitions: after.observation_acquisitions()
                - before.observation_acquisitions(),
            active_observations: after.active_observations() - before.active_observations(),
            lifecycle_observations: after.lifecycle_observations()
                - before.lifecycle_observations(),
            capability_observations: after.capability_observations()
                - before.capability_observations(),
            explicit_closes: after.explicit_closes() - before.explicit_closes(),
            explicit_aborts: after.explicit_aborts() - before.explicit_aborts(),
            panic_terminations: after.panic_terminations() - before.panic_terminations(),
            unexpected_drops: after.unexpected_drops() - before.unexpected_drops(),
        }
    }
}

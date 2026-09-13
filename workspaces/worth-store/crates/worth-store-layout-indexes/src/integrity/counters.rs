#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LayoutCorruptionCounterSnapshot {
    rebuild_classifications_inspected: u64,
    quarantine_observations_inspected: u64,
    recovery_admissions_inspected: u64,
    terminal_imports_inspected: u64,
    outcomes_issued: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutReadmissionCounterSnapshot {
    evidence_witnesses_inspected: u64,
    identity_bindings_checked: u64,
    replay_frontiers_checked: u64,
    foreground_witnesses_issued: u64,
}

impl LayoutReadmissionCounterSnapshot {
    pub(super) const fn new(
        identity_bindings_checked: u64,
        replay_frontiers_checked: u64,
        foreground_witnesses_issued: u64,
    ) -> Self {
        Self {
            evidence_witnesses_inspected: 1,
            identity_bindings_checked,
            replay_frontiers_checked,
            foreground_witnesses_issued,
        }
    }

    pub const fn evidence_witnesses_inspected(self) -> u64 {
        self.evidence_witnesses_inspected
    }
    pub const fn identity_bindings_checked(self) -> u64 {
        self.identity_bindings_checked
    }
    pub const fn replay_frontiers_checked(self) -> u64 {
        self.replay_frontiers_checked
    }
    pub const fn foreground_witnesses_issued(self) -> u64 {
        self.foreground_witnesses_issued
    }
}

impl LayoutCorruptionCounterSnapshot {
    pub(super) const fn rebuild_classification() -> Self {
        Self {
            rebuild_classifications_inspected: 1,
            outcomes_issued: 1,
            ..Self::empty()
        }
    }

    pub(super) const fn quarantine_observation() -> Self {
        Self {
            quarantine_observations_inspected: 1,
            outcomes_issued: 1,
            ..Self::empty()
        }
    }

    pub(super) const fn terminal_import() -> Self {
        Self {
            terminal_imports_inspected: 1,
            outcomes_issued: 1,
            ..Self::empty()
        }
    }

    pub(super) const fn with_observation_bound_readmission(mut self) -> Self {
        self.recovery_admissions_inspected += 1;
        self.outcomes_issued += 1;
        self
    }

    const fn empty() -> Self {
        Self {
            rebuild_classifications_inspected: 0,
            quarantine_observations_inspected: 0,
            recovery_admissions_inspected: 0,
            terminal_imports_inspected: 0,
            outcomes_issued: 0,
        }
    }

    pub const fn rebuild_classifications_inspected(self) -> u64 {
        self.rebuild_classifications_inspected
    }
    pub const fn quarantine_observations_inspected(self) -> u64 {
        self.quarantine_observations_inspected
    }
    pub const fn recovery_admissions_inspected(self) -> u64 {
        self.recovery_admissions_inspected
    }
    pub const fn terminal_imports_inspected(self) -> u64 {
        self.terminal_imports_inspected
    }
    pub const fn outcomes_issued(self) -> u64 {
        self.outcomes_issued
    }
}

use core::time::Duration;
use std::collections::BTreeMap;

use worth_execution::time::ControlledTime;

use crate::Seed;

/// One explicit clock action before the scenario runs the specified step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockFault {
    Advance(Duration),
    StepWallBack(Duration),
    Reboot,
}

impl ClockFault {
    pub(crate) fn apply(self, clock: &ControlledTime) {
        match self {
            Self::Advance(delta) => clock.advance(delta),
            Self::StepWallBack(delta) => clock.step_wall_back(delta),
            Self::Reboot => clock.reboot_epoch(),
        }
    }
}

/// The inputs needed to reproduce a scenario schedule and its clock faults.
pub struct Spec {
    pub(crate) seed: Seed,
    pub(crate) start_unix_secs: u64,
    pub(crate) steps: usize,
    clock_faults: BTreeMap<usize, Vec<ClockFault>>,
}

impl Spec {
    pub fn new(seed: Seed, start_unix_secs: u64, steps: usize) -> Self {
        Self {
            seed,
            start_unix_secs,
            steps,
            clock_faults: BTreeMap::new(),
        }
    }

    /// Schedules a fault at a zero-based step. Faults at the same step run in
    /// the order they were declared, without consuming scheduling draws.
    ///
    /// # Panics
    ///
    /// Panics if the step is outside the declared run.
    pub fn with_clock_fault(mut self, step: usize, fault: ClockFault) -> Self {
        assert!(step < self.steps, "clock fault step outside scenario");
        self.clock_faults.entry(step).or_default().push(fault);
        self
    }

    pub(crate) fn clock_faults(&self, step: usize) -> &[ClockFault] {
        self.clock_faults.get(&step).map_or(&[], Vec::as_slice)
    }
}

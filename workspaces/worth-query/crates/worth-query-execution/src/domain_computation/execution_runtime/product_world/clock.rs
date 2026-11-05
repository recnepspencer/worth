use std::time::Instant;

use worth_runtime_world::facade::{RuntimeWorldClockSource, RuntimeWorldInstant};

/// Query's monotonic clock for one installed Product World.
///
/// Construction is explicit so every public runtime root states which clock
/// defines its deadlines and recovery ages.
#[derive(Clone)]
pub struct WorthQueryProductWorldClock {
    origin: Instant,
    pub(in crate::domain_computation) admission_clock:
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
}

impl WorthQueryProductWorldClock {
    pub fn start(
        admission_clock: worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    ) -> Self {
        Self {
            origin: admission_clock.now(),
            admission_clock,
        }
    }

    pub(crate) fn deadline(
        &self,
        deadline: worth_execution::MonotonicDeadline,
    ) -> Option<RuntimeWorldInstant> {
        let deadline = deadline.instant_on(&self.admission_clock)?;
        Some(RuntimeWorldInstant::from_ticks(
            deadline
                .saturating_duration_since(self.origin)
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64,
        ))
    }
}

impl RuntimeWorldClockSource for WorthQueryProductWorldClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(
            self.admission_clock
                .now()
                .saturating_duration_since(self.origin)
                .as_nanos()
                .min(u128::from(u64::MAX)) as u64,
        )
    }
}

mod court_doors_for_test;

use std::fmt;
use std::time::{Duration, Instant};

use crate::failure_teardown::{
    teardown_native_bound_world, PulseExecutableWorldFailure, PulseExecutableWorldFailureReport,
};
use crate::native_platform::NativePlatformContract;

use super::{FinalRecovered, Published, PulseExecutableWorld};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseQuiescentObservation {
    observed_for: Duration,
    lifecycle_event_delta: usize,
    native_capture_count: u32,
    process_liveness_checks: u32,
    pixels_unchanged: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformPulseQuiescenceFailure {
    ProcessExited,
    CaptureAffinity,
    PixelsChanged,
}

impl PulseExecutableWorld<Published<FinalRecovered>> {
    pub(crate) fn observe_quiescent(
        self,
        interval: Duration,
    ) -> Result<(Self, PlatformPulseQuiescentObservation), PulseExecutableWorldFailureReport> {
        let Published { mut world, stage } = self.state;
        let result = (|| {
            if world
                .process
                .observed_exit()
                .map_err(PulseExecutableWorldFailure::Launch)?
                .is_some()
            {
                return Err(PulseExecutableWorldFailure::Quiescence(
                    PlatformPulseQuiescenceFailure::ProcessExited,
                ));
            }
            let before = world
                .platform
                .capture_client_area(&world.native_client)
                .map_err(PulseExecutableWorldFailure::Native)?;
            let lifecycle_before = world.lifecycle.measurement().accepted_events();
            let started = Instant::now();
            let lifecycle_after = world
                .lifecycle
                .observe_quiescent_until(started + interval)
                .map_err(PulseExecutableWorldFailure::Lifecycle)?
                .accepted_events();
            if world
                .process
                .observed_exit()
                .map_err(PulseExecutableWorldFailure::Launch)?
                .is_some()
            {
                return Err(PulseExecutableWorldFailure::Quiescence(
                    PlatformPulseQuiescenceFailure::ProcessExited,
                ));
            }
            let after = world
                .platform
                .capture_client_area(&world.native_client)
                .map_err(PulseExecutableWorldFailure::Native)?;
            if before.process_id() != after.process_id()
                || before.width() != after.width()
                || before.height() != after.height()
            {
                return Err(PulseExecutableWorldFailure::Quiescence(
                    PlatformPulseQuiescenceFailure::CaptureAffinity,
                ));
            }
            if before.rgba() != after.rgba() {
                return Err(PulseExecutableWorldFailure::Quiescence(
                    PlatformPulseQuiescenceFailure::PixelsChanged,
                ));
            }
            Ok(PlatformPulseQuiescentObservation {
                observed_for: started.elapsed(),
                lifecycle_event_delta: lifecycle_after.saturating_sub(lifecycle_before),
                native_capture_count: before.capture_count().saturating_add(after.capture_count()),
                process_liveness_checks: 2,
                pixels_unchanged: true,
            })
        })();
        match result {
            Ok(evidence) => Ok((
                Self {
                    state: Published { world, stage },
                },
                evidence,
            )),
            Err(primary) => Err(teardown_native_bound_world(
                primary,
                world.into_failure_resources(),
            )),
        }
    }
}

impl PlatformPulseQuiescentObservation {
    pub(crate) const fn observed_for(self) -> Duration {
        self.observed_for
    }

    pub(crate) const fn lifecycle_event_delta(self) -> usize {
        self.lifecycle_event_delta
    }

    pub(crate) const fn native_capture_count(self) -> u32 {
        self.native_capture_count
    }

    pub(crate) const fn process_liveness_checks(self) -> u32 {
        self.process_liveness_checks
    }

    pub(crate) const fn pixels_unchanged(self) -> bool {
        self.pixels_unchanged
    }
}

impl fmt::Display for PlatformPulseQuiescenceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

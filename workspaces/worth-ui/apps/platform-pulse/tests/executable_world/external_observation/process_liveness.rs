use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

use crate::product_process::{LivePlatformPulseProcess, PlatformPulseProcessLaunchFailure};

const REQUIRED_LIVENESS_HOLD: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StableProcessLivenessObservation {
    process_id: u32,
    held_for: Duration,
    liveness_checks: u32,
}

pub(crate) struct PendingStableProcessLivenessObservation {
    process_id: u32,
    started: Instant,
}

#[derive(Debug)]
pub(crate) enum StableProcessLivenessFailure {
    Poll(PlatformPulseProcessLaunchFailure),
    ExitedBeforeHold,
    ExitedDuringHold,
}

impl fmt::Display for StableProcessLivenessFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poll(failure) => write!(formatter, "liveness poll failed: {failure}"),
            Self::ExitedBeforeHold => {
                formatter.write_str("product exited before the liveness hold began")
            }
            Self::ExitedDuringHold => {
                formatter.write_str("product exited during the required liveness hold")
            }
        }
    }
}

pub(crate) fn begin_stable_process_liveness(
    process: &mut LivePlatformPulseProcess,
) -> Result<PendingStableProcessLivenessObservation, StableProcessLivenessFailure> {
    if process
        .observed_exit()
        .map_err(StableProcessLivenessFailure::Poll)?
        .is_some()
    {
        return Err(StableProcessLivenessFailure::ExitedBeforeHold);
    }
    Ok(PendingStableProcessLivenessObservation {
        process_id: process.id(),
        started: Instant::now(),
    })
}

impl PendingStableProcessLivenessObservation {
    pub(crate) fn finish(
        self,
        process: &mut LivePlatformPulseProcess,
    ) -> Result<StableProcessLivenessObservation, StableProcessLivenessFailure> {
        self.finish_with(thread::sleep, || {
            process
                .observed_exit()
                .map(|status| status.is_some())
                .map_err(StableProcessLivenessFailure::Poll)
        })
    }

    fn finish_with(
        self,
        sleep: impl FnOnce(Duration),
        poll_exited: impl FnOnce() -> Result<bool, StableProcessLivenessFailure>,
    ) -> Result<StableProcessLivenessObservation, StableProcessLivenessFailure> {
        sleep(remaining_hold(self.started.elapsed()));
        if poll_exited()? {
            return Err(StableProcessLivenessFailure::ExitedDuringHold);
        }
        Ok(StableProcessLivenessObservation {
            process_id: self.process_id,
            held_for: self.started.elapsed(),
            liveness_checks: 2,
        })
    }
}

fn remaining_hold(observed_for: Duration) -> Duration {
    REQUIRED_LIVENESS_HOLD.saturating_sub(observed_for)
}

impl StableProcessLivenessObservation {
    pub(crate) fn process_id(self) -> u32 {
        self.process_id
    }

    pub(crate) fn held_for(self) -> Duration {
        self.held_for
    }

    pub(crate) fn liveness_checks(self) -> u32 {
        self.liveness_checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorter_observation_waits_only_for_the_remaining_hold() {
        assert_eq!(
            remaining_hold(Duration::from_millis(175)),
            Duration::from_millis(325)
        );
    }

    #[test]
    fn longer_observation_adds_no_liveness_sleep() {
        assert_eq!(remaining_hold(Duration::from_millis(725)), Duration::ZERO);
    }

    #[test]
    fn child_exit_between_begin_and_finish_is_rejected() {
        let guard = PendingStableProcessLivenessObservation {
            process_id: 7,
            started: Instant::now() - REQUIRED_LIVENESS_HOLD,
        };
        let failure = guard
            .finish_with(
                |remaining| assert_eq!(remaining, Duration::ZERO),
                || Ok(true),
            )
            .expect_err("an exited child cannot complete the liveness hold");

        assert!(matches!(
            failure,
            StableProcessLivenessFailure::ExitedDuringHold
        ));
    }
}

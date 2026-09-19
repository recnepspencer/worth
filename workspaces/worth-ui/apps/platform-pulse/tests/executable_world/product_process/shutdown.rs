use std::fmt;
use std::process::ExitStatus;
use std::thread;
use std::time::{Duration, Instant};

use super::{LivePlatformPulseProcess, PlatformPulseProcessLaunchFailure};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SuccessfulPlatformPulseExit {
    status: ExitStatus,
    poll_count: u32,
}

#[derive(Debug)]
pub(crate) enum PlatformPulseProcessExitFailure {
    Poll(PlatformPulseProcessLaunchFailure),
    Deadline,
    Unsuccessful(ExitStatus),
}

impl fmt::Display for PlatformPulseProcessExitFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poll(failure) => write!(formatter, "poll failed: {failure}"),
            Self::Deadline => formatter.write_str("successful process exit deadline elapsed"),
            Self::Unsuccessful(status) => {
                write!(formatter, "product exited unsuccessfully: {status}")
            }
        }
    }
}

impl SuccessfulPlatformPulseExit {
    pub(crate) fn wait(
        process: &mut LivePlatformPulseProcess,
        deadline: Instant,
    ) -> Result<Self, PlatformPulseProcessExitFailure> {
        let mut poll_count = 0_u32;
        loop {
            poll_count = poll_count.saturating_add(1);
            if let Some(status) = process
                .observed_exit()
                .map_err(PlatformPulseProcessExitFailure::Poll)?
            {
                if status.success() {
                    return Ok(Self { status, poll_count });
                }
                return Err(PlatformPulseProcessExitFailure::Unsuccessful(status));
            }
            if Instant::now() >= deadline {
                return Err(PlatformPulseProcessExitFailure::Deadline);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub(crate) fn status(self) -> ExitStatus {
        self.status
    }

    pub(crate) fn poll_count(self) -> u32 {
        self.poll_count
    }
}

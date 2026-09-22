use std::fmt;

use super::{
    EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure,
    PlatformPulseProcessLaunchFailure,
};

impl fmt::Display for PlatformPulseProcessLaunchFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CargoExecutableNotAbsolute(path) => write!(
                formatter,
                "Cargo executable is not absolute: {}",
                path.display()
            ),
            Self::CargoExecutableMissing(path) => {
                write!(formatter, "Cargo executable is missing: {}", path.display())
            }
            Self::NativeDesktopLease(failure) => {
                write!(formatter, "exclusive native desktop lease: {failure}")
            }
            Self::Spawn(error) => write!(formatter, "spawn product process: {error}"),
            #[cfg(target_os = "windows")]
            Self::KillOnCloseJob(error) => write!(formatter, "contain product process: {error}"),
            Self::MissingStdout { teardown } => {
                write!(
                    formatter,
                    "product stdout was not piped; process teardown: "
                )?;
                format_emergency_exit_result(formatter, teardown)
            }
            Self::Poll(error) => write!(formatter, "poll product process: {error}"),
        }
    }
}

impl fmt::Display for EmergencyPlatformPulseExitFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Poll(error) => write!(formatter, "poll process during teardown: {error}"),
            Self::Terminate(error) => write!(formatter, "terminate failed process: {error}"),
            Self::Deadline => formatter.write_str("failed-process termination deadline elapsed"),
        }
    }
}

fn format_emergency_exit_result(
    formatter: &mut fmt::Formatter<'_>,
    result: &Result<EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure>,
) -> fmt::Result {
    match result {
        Ok(exit) => write!(
            formatter,
            "released(status={}, forced={}, polls={}, containment={})",
            exit.status,
            exit.forced_termination,
            exit.poll_count,
            exit.containment.name()
        ),
        Err(failure) => write!(formatter, "failed({failure})"),
    }
}

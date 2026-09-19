use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::external_observation::PlatformPulseLifecycleStream;

#[cfg(target_os = "windows")]
use super::kill_on_close_job::KillOnCloseJob;
#[cfg(target_os = "windows")]
use super::native_desktop_lease::{NativeDesktopCourtroomLease, NativeDesktopLease};
use super::output_capture::NativeProcessOutputCapture;

mod failure_display;

// The product-world journey ceiling is 45 seconds. An adjacent honest runner
// receives one full journey plus teardown headroom before contention is typed
// as an environment denial.
#[cfg(target_os = "windows")]
const NATIVE_DESKTOP_LEASE_DEADLINE: Duration = Duration::from_secs(60);

#[derive(Clone, Debug)]
pub(crate) struct CargoBuiltPlatformPulse {
    executable: PathBuf,
}

pub(crate) struct PlatformPulseProcessLaunch {
    pub(crate) process: LivePlatformPulseProcess,
    pub(crate) lifecycle: PlatformPulseLifecycleStream,
    pub(crate) launch_started: Instant,
}

pub(crate) struct NativePhase2ProcessLaunch {
    pub(crate) process: LivePlatformPulseProcess,
    pub(crate) stdout: NativeProcessOutputCapture,
}

pub(crate) struct LivePlatformPulseProcess {
    #[cfg(target_os = "windows")]
    _native_desktop_lease: NativeDesktopLease,
    #[cfg(target_os = "windows")]
    _native_desktop_courtroom: NativeDesktopCourtroomLease,
    child: Child,
    #[cfg(target_os = "windows")]
    _kill_on_close_job: KillOnCloseJob,
    fallback_termination_required: bool,
    exit_status: Option<ExitStatus>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct EmergencyPlatformPulseExit {
    status: ExitStatus,
    forced_termination: bool,
    poll_count: u32,
}

#[derive(Debug)]
pub(crate) enum PlatformPulseProcessLaunchFailure {
    CargoExecutableNotAbsolute(PathBuf),
    CargoExecutableMissing(PathBuf),
    NativeDesktopLease,
    Spawn(std::io::Error),
    #[cfg(target_os = "windows")]
    KillOnCloseJob(String),
    MissingStdout {
        teardown: Result<EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure>,
    },
    Poll(std::io::Error),
}

#[derive(Debug)]
pub(crate) enum EmergencyPlatformPulseExitFailure {
    Poll(std::io::Error),
    Terminate(std::io::Error),
    Deadline,
}

impl CargoBuiltPlatformPulse {
    pub(crate) fn exact() -> Result<Self, PlatformPulseProcessLaunchFailure> {
        let executable = PathBuf::from(env!("CARGO_BIN_EXE_worth-ui-platform-pulse"));
        if !executable.is_absolute() {
            return Err(PlatformPulseProcessLaunchFailure::CargoExecutableNotAbsolute(executable));
        }
        if !executable.is_file() {
            return Err(PlatformPulseProcessLaunchFailure::CargoExecutableMissing(
                executable,
            ));
        }
        Ok(Self { executable })
    }

    pub(crate) fn launch(
        self,
        source_root: &Path,
        query_source_root: &Path,
        intent_source_root: &Path,
    ) -> Result<PlatformPulseProcessLaunch, PlatformPulseProcessLaunchFailure> {
        #[cfg(target_os = "windows")]
        let native_desktop_courtroom = NativeDesktopCourtroomLease::acquire();
        #[cfg(target_os = "windows")]
        let desktop_wait_started = Instant::now();
        #[cfg(target_os = "windows")]
        let desktop_deadline = desktop_wait_started
            .checked_add(NATIVE_DESKTOP_LEASE_DEADLINE)
            .ok_or(PlatformPulseProcessLaunchFailure::NativeDesktopLease)?;
        #[cfg(target_os = "windows")]
        let native_desktop_lease = NativeDesktopLease::acquire(desktop_deadline)
            .map_err(|_| PlatformPulseProcessLaunchFailure::NativeDesktopLease)?;
        let launch_started = Instant::now();
        let native_close_evidence_path = source_root.join(super::NATIVE_CLOSE_EVIDENCE_FILE_NAME);
        let mut command = Command::new(&self.executable);
        let mut child = command
            .arg("--source-root")
            .arg(source_root)
            .arg("--query-source-root")
            .arg(query_source_root)
            .arg("--intent-source-root")
            .arg(intent_source_root)
            .env(
                super::NATIVE_CLOSE_EVIDENCE_PATH_ENVIRONMENT,
                native_close_evidence_path,
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(PlatformPulseProcessLaunchFailure::Spawn)?;
        #[cfg(target_os = "windows")]
        let kill_on_close_job = assign_kill_on_close_job(&mut child)?;
        let mut process = LivePlatformPulseProcess {
            #[cfg(target_os = "windows")]
            _native_desktop_lease: native_desktop_lease,
            #[cfg(target_os = "windows")]
            _native_desktop_courtroom: native_desktop_courtroom,
            child,
            #[cfg(target_os = "windows")]
            _kill_on_close_job: kill_on_close_job,
            fallback_termination_required: true,
            exit_status: None,
        };
        let Some(stdout) = process.child.stdout.take() else {
            let teardown = process.terminate_after_failure(Instant::now() + Duration::from_secs(5));
            return Err(PlatformPulseProcessLaunchFailure::MissingStdout { teardown });
        };
        Ok(PlatformPulseProcessLaunch {
            process,
            lifecycle: PlatformPulseLifecycleStream::read(stdout),
            launch_started,
        })
    }

    pub(crate) fn launch_native_phase2(
        self,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&["--worth-ui-native-phase2-world"])
    }

    pub(crate) fn launch_native_phase6(
        self,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&["--worth-ui-native-phase6-world"])
    }

    pub(crate) fn launch_native_phase7(
        self,
        points: &str,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        let argument = format!("--worth-ui-native-phase7-world={points}");
        self.launch_native(&[&argument])
    }

    pub(crate) fn launch_native_phase8(
        self,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&["--worth-ui-native-phase8-world"])
    }

    pub(crate) fn launch_native_phase3(
        self,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&["--worth-ui-native-phase3-world"])
    }

    pub(crate) fn launch_native_gate_d_pin_world(
        self,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&["--worth-ui-native-gate-d-pin-world"])
    }

    pub(crate) fn launch_native_phase_f_world(
        self,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&[
            "--worth-ui-native-phase-f-world",
            "--worth-ui-native-external-close",
        ])
    }

    pub(crate) fn launch_native_phase_f_partial_cancellation(
        self,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&["--worth-ui-native-phase-f-partial-cancellation-world"])
    }

    pub(crate) fn launch_native_phase_f_reconstruction(
        self,
        class: &str,
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        self.launch_native(&[&format!(
            "--worth-ui-native-phase-f-reconstruction-world={class}"
        )])
    }

    fn launch_native(
        self,
        arguments: &[&str],
    ) -> Result<NativePhase2ProcessLaunch, PlatformPulseProcessLaunchFailure> {
        #[cfg(target_os = "windows")]
        let native_desktop_courtroom = NativeDesktopCourtroomLease::acquire();
        #[cfg(target_os = "windows")]
        let deadline = Instant::now()
            .checked_add(NATIVE_DESKTOP_LEASE_DEADLINE)
            .ok_or(PlatformPulseProcessLaunchFailure::NativeDesktopLease)?;
        #[cfg(target_os = "windows")]
        let native_desktop_lease = NativeDesktopLease::acquire(deadline)
            .map_err(|_| PlatformPulseProcessLaunchFailure::NativeDesktopLease)?;
        let mut command = Command::new(&self.executable);
        let mut child = command
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(PlatformPulseProcessLaunchFailure::Spawn)?;
        #[cfg(target_os = "windows")]
        let kill_on_close_job = assign_kill_on_close_job(&mut child)?;
        let mut process = LivePlatformPulseProcess {
            #[cfg(target_os = "windows")]
            _native_desktop_lease: native_desktop_lease,
            #[cfg(target_os = "windows")]
            _native_desktop_courtroom: native_desktop_courtroom,
            child,
            #[cfg(target_os = "windows")]
            _kill_on_close_job: kill_on_close_job,
            fallback_termination_required: true,
            exit_status: None,
        };
        let Some(stdout) = process.child.stdout.take() else {
            let teardown = process.terminate_after_failure(Instant::now() + Duration::from_secs(5));
            return Err(PlatformPulseProcessLaunchFailure::MissingStdout { teardown });
        };
        Ok(NativePhase2ProcessLaunch {
            process,
            stdout: NativeProcessOutputCapture::start(stdout),
        })
    }
}

impl LivePlatformPulseProcess {
    pub(crate) fn id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn observed_exit(
        &mut self,
    ) -> Result<Option<ExitStatus>, PlatformPulseProcessLaunchFailure> {
        if let Some(status) = self.exit_status {
            return Ok(Some(status));
        }
        let status = self
            .child
            .try_wait()
            .map_err(PlatformPulseProcessLaunchFailure::Poll)?;
        if let Some(status) = status {
            self.exit_status = Some(status);
            self.fallback_termination_required = false;
        }
        Ok(status)
    }

    pub(crate) fn terminate_after_failure(
        &mut self,
        deadline: Instant,
    ) -> Result<EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure> {
        let initial_poll_count = 1_u32;
        if let Some(status) = self.poll_exit_after_failure()? {
            return Ok(emergency_exit(status, false, initial_poll_count));
        }
        if let Err(error) = self.child.kill() {
            let race_poll_count = initial_poll_count.saturating_add(1);
            if let Some(status) = self.poll_exit_after_failure()? {
                return Ok(emergency_exit(status, false, race_poll_count));
            }
            return Err(EmergencyPlatformPulseExitFailure::Terminate(error));
        }
        self.await_forced_exit(deadline, initial_poll_count)
    }

    fn await_forced_exit(
        &mut self,
        deadline: Instant,
        prior_poll_count: u32,
    ) -> Result<EmergencyPlatformPulseExit, EmergencyPlatformPulseExitFailure> {
        let mut poll_count = prior_poll_count;
        loop {
            poll_count = poll_count.saturating_add(1);
            if let Some(status) = self.poll_exit_after_failure()? {
                return Ok(emergency_exit(status, true, poll_count));
            }
            if Instant::now() >= deadline {
                return Err(EmergencyPlatformPulseExitFailure::Deadline);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn poll_exit_after_failure(
        &mut self,
    ) -> Result<Option<ExitStatus>, EmergencyPlatformPulseExitFailure> {
        self.observed_exit().map_err(|failure| match failure {
            PlatformPulseProcessLaunchFailure::Poll(error) => {
                EmergencyPlatformPulseExitFailure::Poll(error)
            }
            _ => unreachable!("observed_exit returns only Poll"),
        })
    }
}

impl EmergencyPlatformPulseExit {
    pub(crate) fn forced_termination(self) -> bool {
        self.forced_termination
    }
}

#[cfg(target_os = "windows")]
fn assign_kill_on_close_job(
    child: &mut Child,
) -> Result<KillOnCloseJob, PlatformPulseProcessLaunchFailure> {
    KillOnCloseJob::assign(child).map_err(|error| {
        let _ = child.kill();
        let _ = child.wait();
        PlatformPulseProcessLaunchFailure::KillOnCloseJob(error)
    })
}

fn emergency_exit(
    status: ExitStatus,
    forced_termination: bool,
    poll_count: u32,
) -> EmergencyPlatformPulseExit {
    EmergencyPlatformPulseExit {
        status,
        forced_termination,
        poll_count,
    }
}

impl Drop for LivePlatformPulseProcess {
    fn drop(&mut self) {
        if self.fallback_termination_required {
            if let Err(failure) =
                self.terminate_after_failure(Instant::now() + Duration::from_secs(2))
            {
                eprintln!(
                    "fallback Platform Pulse teardown deferred to kill-on-close containment: {failure}"
                );
            }
        }
    }
}

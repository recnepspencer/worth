use crate::facade::{WorthUiApp, WorthUiNativeApplicationShell};
use worth_ui_host_native::{
    UiNativeEventLoopClient, UiNativeEventLoopClientClose, UiNativeEventLoopClientFailure,
    UiNativeEventLoopDirective, UiNativeObservationReadinessGrant, UiNativeReadinessGrant,
    WorthUiNativeEventLoop,
};

#[path = "application_driver/application_runtime.rs"]
mod application_runtime;
#[path = "application_driver/cleanup.rs"]
mod cleanup;
#[path = "application_driver/host_callbacks.rs"]
mod host_callbacks;
#[path = "application_driver/motion_readiness.rs"]
mod motion_readiness;
mod observation_time;
#[path = "application_driver/physical_recovery_tracker.rs"]
mod physical_recovery_tracker;
#[cfg(test)]
pub(crate) mod pointer_refresh_tests;
#[path = "application_driver/program_progress.rs"]
mod program_progress;
#[path = "application_driver/program_reconstruction.rs"]
mod program_reconstruction;
#[path = "application_driver/runtime_qualification.rs"]
mod runtime_qualification;
#[path = "application_driver/shutdown_observation.rs"]
pub(crate) mod shutdown_observation;
#[cfg(test)]
#[path = "application_driver/tests.rs"]
mod tests;
mod visual_snapshot;
use cleanup::UiNativeApplicationDriverCleanup;
#[cfg(test)]
use cleanup::UiNativeApplicationDriverCleanupCompletion;
use motion_readiness::UiNativeMotionReadinessLane;
use program_progress::UiNativeApplicationProgramProgress;
use shutdown_observation::UiNativeDriverShutdownEvidence;

pub(crate) struct UiNativeApplicationDriver {
    observation_clock: Option<worth_ui_host_native::UiNativeObservationClock>,
    application: Option<WorthUiApp>,
    native_surface_declaration: Option<Box<str>>,
    shell: Option<WorthUiNativeApplicationShell>,
    last_ready_generation: u64,
    last_observation_ready_generation: u64,
    observation_ingress_counts: [u64; 5],
    scale_factor_milli: Option<u32>,
    consumed_application_cleanup_complete: bool,
    pending_cleanup: Option<UiNativeApplicationDriverCleanup>,
    progress: UiNativeApplicationProgramProgress,
    application_runtime: Option<Box<dyn super::UiNativeApplicationRuntime>>,
    application_runtime_ports: Option<Box<[super::UiNativeApplicationReadinessPort]>>,
    motion_support_installed: bool,
    motion_readiness: Option<UiNativeMotionReadinessLane>,
    last_motion_readiness_generation: u64,
    application_runtime_active: bool,
    pending_application_runtime_close: Option<super::UiNativeApplicationRuntimeCloseIncomplete>,
}

impl UiNativeApplicationDriver {
    pub(crate) fn new(
        application: WorthUiApp,
        program: crate::facade::entry::UiNativeApplicationProgram,
        runtime_qualification: Option<
            super::runtime_qualification::UiNativeRuntimeQualificationPlan,
        >,
        application_runtime: Option<Box<dyn super::UiNativeApplicationRuntime>>,
        native_surface_declaration: Option<Box<str>>,
    ) -> Self {
        let motion_support_installed = application.motion_support_installed();
        Self {
            observation_clock: None,
            application: Some(application),
            native_surface_declaration,
            shell: None,
            last_ready_generation: 0,
            last_observation_ready_generation: 0,
            observation_ingress_counts: [0; 5],
            scale_factor_milli: None,
            consumed_application_cleanup_complete: false,
            pending_cleanup: None,
            progress: UiNativeApplicationProgramProgress::new(program, runtime_qualification),
            application_runtime,
            application_runtime_ports: None,
            motion_support_installed,
            motion_readiness: None,
            last_motion_readiness_generation: 0,
            application_runtime_active: false,
            pending_application_runtime_close: None,
        }
    }

    #[cfg(test)]
    fn from_launched_shell_for_test(shell: WorthUiNativeApplicationShell) -> Self {
        Self {
            observation_clock: None,
            application: None,
            native_surface_declaration: None,
            shell: Some(shell),
            last_ready_generation: 0,
            last_observation_ready_generation: 0,
            observation_ingress_counts: [0; 5],
            scale_factor_milli: Some(1_000),
            consumed_application_cleanup_complete: false,
            pending_cleanup: None,
            progress: UiNativeApplicationProgramProgress::new(
                crate::facade::entry::UiNativeApplicationProgram::single_frame(),
                None,
            ),
            application_runtime: None,
            application_runtime_ports: None,
            motion_support_installed: false,
            motion_readiness: None,
            last_motion_readiness_generation: 0,
            application_runtime_active: false,
            pending_application_runtime_close: None,
        }
    }

    pub(crate) fn run(
        self,
        event_loop: WorthUiNativeEventLoop,
    ) -> Result<
        worth_ui_host_native::UiNativeEventLoopRunReport,
        worth_ui_host_native::UiNativeEventLoopStopReport,
    > {
        event_loop.run(self)
    }

    fn next_directive(&self) -> UiNativeEventLoopDirective {
        if self.progress.should_close() {
            UiNativeEventLoopDirective::Close
        } else {
            UiNativeEventLoopDirective::Continue
        }
    }
}

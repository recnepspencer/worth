use worth_ui::facade::app::UiMountedFrameOutcome;
use worth_ui_native_platform::UiNativeSurfaceSuccession;

use super::PlatformPulseApplicationRuntime;
use crate::native_application::frame_execution_diagnostic;
use crate::native_application::terminal_error::PlatformPulseTerminalError;

#[path = "frame_timing.rs"]
mod frame_timing;

/// A presentation still owed its host outcome. `reconstruction` is the
/// frame's purpose, which a retry after rejection re-enters.
pub(super) enum PlatformPulsePendingFramePresentation {
    InFlight {
        presentation: worth_ui::facade::app::UiMountedPresentationInFlight,
        reconstruction: bool,
    },
    PhysicalRecovery {
        frame: worth_ui::facade::app::UiMountedIndeterminateFrame,
        reconstruction: bool,
    },
}

/// A frame the host rejected before effects while it waits for readiness it
/// schedules itself. The next presentation re-enters preparation with the
/// rejected frame's purpose instead of ending the runtime.
#[derive(Clone, Copy)]
pub(super) struct PlatformPulsePresentationRetry {
    reconstruction: bool,
}

impl PlatformPulseApplicationRuntime {
    pub(super) fn present(&mut self) {
        // A host-rejected frame retries only at the readiness the host
        // schedules, so a product turn before it cannot spend the retry.
        if self
            .shell
            .as_ref()
            .is_some_and(|shell| shell.native_presentation_retry_pending())
        {
            return;
        }
        self.present_for_surface(None);
    }

    /// Present after a host surface succession, if any. A successor target
    /// the host kept the retained presentation on only needs a frame; a
    /// reconstruction the host owes is presented as one directly.
    pub(super) fn present_for_surface(&mut self, surface: Option<UiNativeSurfaceSuccession>) {
        if self.external_close_requested
            || self.pending_frame_presentation.is_some()
            || self.pending_managed_rebind.is_some()
        {
            return;
        }
        let Some(shell) = self.shell.as_mut() else {
            return;
        };
        if shell.native_motion_sample_presentation_pending() {
            return;
        }
        let first_frame = self.initial_source.is_some();
        let viewport_successor = shell.native_viewport_presentation_pending();
        let retry = self.presentation_retry;
        let reconstruction_required = surface == Some(UiNativeSurfaceSuccession::Reconstruct)
            || retry.is_some_and(|retry| retry.reconstruction);
        if !first_frame
            && surface.is_none()
            && !reconstruction_required
            && retry.is_none()
            && !shell.native_presentation_retry_pending()
            && !viewport_successor
            && !shell.native_pointer_presentation_pending()
            && !shell.native_application_presentation_pending()
            && !shell.native_presentation_reconstruction_pending()
        {
            return;
        }
        if viewport_successor {
            match super::layout::publish_native_layout(shell) {
                Ok(true) => {}
                Ok(false) => return,
                Err(detail) => {
                    self.fail(PlatformPulseTerminalError::FrameExecution(detail), Ok(()));
                    return;
                }
            }
        }
        let Some((now, deadline)) = self.sample_frame_time() else {
            return;
        };
        self.presentation_retry = None;
        let shell = self
            .shell
            .as_mut()
            .expect("frame preparation retains the runtime shell");
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let reconstruction = reconstruction_required && !first_frame;
        let outcome = if reconstruction {
            match shell.reconstruct_native_surface_successor(deadline, now) {
                Ok(outcome) => outcome,
                Err(denial) => {
                    self.fail(
                        PlatformPulseTerminalError::FrameExecution(format!(
                            "native-surface-successor:{denial:?}"
                        )),
                        Ok(()),
                    );
                    return;
                }
            }
        } else {
            match shell.present_frame(deadline, now) {
                Ok(outcome) => outcome,
                Err(denial) => {
                    let detail = frame_execution_diagnostic::stop_label(&denial);
                    let observation = self.publisher.frame_execution_failure(&denial);
                    drop(denial);
                    self.fail(
                        PlatformPulseTerminalError::FrameExecution(detail),
                        observation,
                    );
                    return;
                }
            }
        };
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let outcome = match shell.resume_frame_presentation(outcome, deadline, now) {
            Ok(outcome) => outcome,
            Err(denial) => {
                self.fail(
                    PlatformPulseTerminalError::FrameExecution(format!(
                        "host-required-reconstruction-unavailable:{denial:?}"
                    )),
                    Ok(()),
                );
                return;
            }
        };
        self.settle_frame_outcome(outcome, reconstruction);
    }

    fn settle_frame_outcome(&mut self, outcome: UiMountedFrameOutcome, reconstruction: bool) {
        match outcome {
            UiMountedFrameOutcome::Published(publication)
            | UiMountedFrameOutcome::Reconciled(publication) => {
                if let Some(source) = self.initial_source.take() {
                    if let Err(error) = self.publish_first_frame(&source, &publication) {
                        self.fail(
                            PlatformPulseTerminalError::ObservationPublication,
                            Err(error),
                        );
                        return;
                    }
                    if let Err(denial) = self
                        .visual_identity
                        .arm_after_first_frame(std::time::Instant::now())
                    {
                        self.fail_visual_identity(denial);
                    }
                } else {
                    if let Err(error) = self.publisher.mounted_content_published(&publication) {
                        self.fail(
                            PlatformPulseTerminalError::ObservationPublication,
                            Err(error),
                        );
                        return;
                    }
                    let Some(shell) = self.shell.as_mut() else {
                        self.fail(
                            PlatformPulseTerminalError::FrameExecution(
                                "published-frame-lost-runtime-shell".to_owned(),
                            ),
                            Ok(()),
                        );
                        return;
                    };
                    let refresh = self.visual_identity.refresh_after_presentation_replacement(
                        shell,
                        self.presentation_tick,
                        std::time::Instant::now(),
                    );
                    if let Err(denial) = refresh {
                        self.fail_visual_identity(denial);
                    }
                }
            }
            UiMountedFrameOutcome::Unchanged(_) if self.initial_source.is_none() => {}
            UiMountedFrameOutcome::Unchanged(_) => {
                self.fail(PlatformPulseTerminalError::UnexpectedInitialFrame, Ok(()));
            }
            UiMountedFrameOutcome::InFlight(presentation) => {
                self.pending_frame_presentation =
                    Some(PlatformPulsePendingFramePresentation::InFlight {
                        presentation,
                        reconstruction,
                    });
            }
            UiMountedFrameOutcome::PresentationIndeterminate(frame)
                if frame.report().awaits_physical_recovery() =>
            {
                self.pending_frame_presentation =
                    Some(PlatformPulsePendingFramePresentation::PhysicalRecovery {
                        frame,
                        reconstruction,
                    });
            }
            outcome
                if worth_ui::facade::app::WorthUiNativeApplicationShell::
                    frame_presentation_awaits_host_readiness(&outcome) =>
            {
                self.presentation_retry = Some(PlatformPulsePresentationRetry { reconstruction });
            }
            outcome => {
                let observation = self.publisher.frame_outcome_failure(&outcome);
                self.fail(
                    PlatformPulseTerminalError::FrameExecution(
                        frame_execution_diagnostic::outcome_label(&outcome),
                    ),
                    observation,
                );
            }
        }
    }

    pub(super) fn progress_pending_frame_presentation(
        &mut self,
        progress: &worth_ui_native_platform::UiNativeApplicationPhysicalProgress,
    ) -> bool {
        if self.pending_frame_presentation.is_none() {
            return false;
        }
        let Some((now, deadline)) = self.sample_frame_time() else {
            return true;
        };
        let Some(pending) = self.pending_frame_presentation.take() else {
            return false;
        };
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let (outcome, reconstruction) = match pending {
            PlatformPulsePendingFramePresentation::InFlight {
                presentation,
                reconstruction,
            } => {
                let outcome = self
                    .shell
                    .as_mut()
                    .expect("pending presentation retains the runtime shell")
                    .complete_frame_presentation(presentation, now);
                (outcome, reconstruction)
            }
            PlatformPulsePendingFramePresentation::PhysicalRecovery {
                frame,
                reconstruction,
            } => {
                let Some(outcome) =
                    self.recover_physical_frame(frame, progress, reconstruction, (now, deadline))
                else {
                    return true;
                };
                (outcome, reconstruction)
            }
        };
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let recovered = self
            .shell
            .as_mut()
            .expect("presentation completion retains the runtime shell")
            .resume_frame_presentation(outcome, deadline, now);
        match recovered {
            Ok(UiMountedFrameOutcome::PresentationIndeterminate(frame))
                if frame.report().awaits_physical_recovery() =>
            {
                if let Some(outcome) =
                    self.recover_physical_frame(frame, progress, reconstruction, (now, deadline))
                {
                    self.settle_frame_outcome(outcome, reconstruction);
                }
            }
            Ok(outcome) => self.settle_frame_outcome(outcome, reconstruction),
            Err(denial) => self.fail(
                PlatformPulseTerminalError::FrameExecution(format!(
                    "host-required-reconstruction-unavailable:{denial:?}"
                )),
                Ok(()),
            ),
        }
        true
    }

    fn recover_physical_frame(
        &mut self,
        frame: worth_ui::facade::app::UiMountedIndeterminateFrame,
        progress: &worth_ui_native_platform::UiNativeApplicationPhysicalProgress,
        reconstruction: bool,
        (now, deadline): (u64, u64),
    ) -> Option<UiMountedFrameOutcome> {
        let recovery = self
            .shell
            .as_mut()
            .expect("physical recovery retains the runtime shell")
            .progress_indeterminate_presentation_recovery(frame, progress, deadline, now);
        match recovery {
            worth_ui::facade::app::WorthUiNativePhysicalPresentationRecovery::Awaiting(frame) => {
                self.pending_frame_presentation =
                    Some(PlatformPulsePendingFramePresentation::PhysicalRecovery {
                        frame,
                        reconstruction,
                    });
                None
            }
            worth_ui::facade::app::WorthUiNativePhysicalPresentationRecovery::Blocked {
                frame,
                ..
            } => {
                self.pending_frame_presentation =
                    Some(PlatformPulsePendingFramePresentation::PhysicalRecovery {
                        frame,
                        reconstruction,
                    });
                None
            }
            worth_ui::facade::app::WorthUiNativePhysicalPresentationRecovery::Recovered(
                outcome,
            ) => Some(outcome),
        }
    }
}

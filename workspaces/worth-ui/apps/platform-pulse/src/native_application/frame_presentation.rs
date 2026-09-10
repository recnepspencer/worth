use worth_ui::facade::app::UiMountedFrameOutcome;

use super::PlatformPulseApplicationRuntime;
use crate::native_application::frame_execution_diagnostic;
use crate::native_application::terminal_error::PlatformPulseTerminalError;

pub(super) enum PlatformPulsePendingFramePresentation {
    InFlight {
        presentation: worth_ui::facade::app::UiMountedPresentationInFlight,
        viewport_successor: bool,
    },
    PhysicalRecovery {
        frame: worth_ui::facade::app::UiMountedIndeterminateFrame,
        viewport_successor: bool,
    },
}

impl PlatformPulseApplicationRuntime {
    pub(super) fn present(&mut self) {
        if self.pending_frame_presentation.is_some() || self.pending_managed_rebind.is_some() {
            return;
        }
        let Some(shell) = self.shell.as_mut() else {
            return;
        };
        let first_frame = self.initial_source.is_some();
        let viewport_successor = shell.native_viewport_presentation_pending();
        if !first_frame && !viewport_successor && !shell.native_pointer_presentation_pending() {
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
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let deadline = self.presentation_tick.saturating_add(1);
        let outcome = match shell.present_frame(deadline, self.presentation_tick) {
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
        };
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let outcome = match shell.recover_reconstruction_required_presentation(
            outcome,
            self.presentation_tick.saturating_add(1),
            self.presentation_tick,
        ) {
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
        self.settle_frame_outcome(outcome, viewport_successor);
    }

    fn settle_frame_outcome(&mut self, outcome: UiMountedFrameOutcome, viewport_successor: bool) {
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
                } else if viewport_successor {
                    let Some(shell) = self.shell.as_mut() else {
                        self.fail(
                            PlatformPulseTerminalError::FrameExecution(
                                "published-frame-lost-runtime-shell".to_owned(),
                            ),
                            Ok(()),
                        );
                        return;
                    };
                    let refresh = self.visual_identity.refresh_after_viewport_replacement(
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
                        viewport_successor,
                    });
            }
            UiMountedFrameOutcome::PresentationIndeterminate(frame)
                if frame.report().awaits_physical_recovery() =>
            {
                self.pending_frame_presentation =
                    Some(PlatformPulsePendingFramePresentation::PhysicalRecovery {
                        frame,
                        viewport_successor,
                    });
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
        let Some(pending) = self.pending_frame_presentation.take() else {
            return false;
        };
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let (outcome, viewport_successor) = match pending {
            PlatformPulsePendingFramePresentation::InFlight {
                presentation,
                viewport_successor,
            } => {
                let outcome = self
                    .shell
                    .as_mut()
                    .expect("pending presentation retains the runtime shell")
                    .complete_frame_presentation(presentation, self.presentation_tick);
                (outcome, viewport_successor)
            }
            PlatformPulsePendingFramePresentation::PhysicalRecovery {
                frame,
                viewport_successor,
            } => {
                let Some(outcome) =
                    self.recover_physical_frame(frame, progress, viewport_successor)
                else {
                    return true;
                };
                (outcome, viewport_successor)
            }
        };
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let recovered = self
            .shell
            .as_mut()
            .expect("presentation completion retains the runtime shell")
            .recover_reconstruction_required_presentation(
                outcome,
                self.presentation_tick.saturating_add(1),
                self.presentation_tick,
            );
        match recovered {
            Ok(UiMountedFrameOutcome::PresentationIndeterminate(frame))
                if frame.report().awaits_physical_recovery() =>
            {
                if let Some(outcome) =
                    self.recover_physical_frame(frame, progress, viewport_successor)
                {
                    self.settle_frame_outcome(outcome, viewport_successor);
                }
            }
            Ok(outcome) => self.settle_frame_outcome(outcome, viewport_successor),
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
        viewport_successor: bool,
    ) -> Option<UiMountedFrameOutcome> {
        let recovery = self
            .shell
            .as_mut()
            .expect("physical recovery retains the runtime shell")
            .progress_indeterminate_presentation_recovery(
                frame,
                progress,
                self.presentation_tick.saturating_add(1),
                self.presentation_tick,
            );
        match recovery {
            worth_ui::facade::app::WorthUiNativePhysicalPresentationRecovery::Awaiting(frame) => {
                self.pending_frame_presentation =
                    Some(PlatformPulsePendingFramePresentation::PhysicalRecovery {
                        frame,
                        viewport_successor,
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
                        viewport_successor,
                    });
                None
            }
            worth_ui::facade::app::WorthUiNativePhysicalPresentationRecovery::Recovered(
                outcome,
            ) => Some(outcome),
        }
    }
}

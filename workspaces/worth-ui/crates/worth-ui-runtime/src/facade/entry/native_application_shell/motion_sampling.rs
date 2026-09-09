use super::WorthUiNativeApplicationShell;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiNativeMotionTickDisposition {
    Inactive,
    Active,
    AwaitingPhysicalCompletion,
}

impl UiNativeMotionTickDisposition {
    pub(crate) const fn schedules_next_frame(self) -> bool {
        matches!(self, Self::Active)
    }
}

impl WorthUiNativeApplicationShell {
    pub(crate) fn admit_native_motion_tick(
        &mut self,
        tick: u64,
        reduced_motion: worth_ui_host_native::UiNativeReducedMotionPosture,
    ) -> Result<UiNativeMotionTickDisposition, ()> {
        use super::super::active_application_session::UiPortalExitTerminalProgress;

        self.reduced_motion_posture = map_reduced_motion_inspection(reduced_motion);

        match self.session.progress_portal_exit_terminal(tick) {
            UiPortalExitTerminalProgress::Retry => {
                return Ok(UiNativeMotionTickDisposition::Active);
            }
            UiPortalExitTerminalProgress::AwaitingPhysical => {
                return Ok(UiNativeMotionTickDisposition::AwaitingPhysicalCompletion);
            }
            UiPortalExitTerminalProgress::Published | UiPortalExitTerminalProgress::Idle => {}
        }
        self.session
            .mounted
            .set_reduced_motion_posture(map_reduced_motion(reduced_motion));
        if !self.session.mounted.has_active_motion_samples() {
            return Ok(UiNativeMotionTickDisposition::Inactive);
        }
        if self.session.mounted.motion_sample_presentation_pending() {
            return Ok(UiNativeMotionTickDisposition::AwaitingPhysicalCompletion);
        }
        let presentation = self.current_motion_presentation().ok_or(())?;
        let prepared = self
            .session
            .prepare_motion_tick(tick, presentation)
            .map_err(|_| ())?;
        self.session
            .present_prepared_motion_tick(prepared, presentation);
        Ok(self.native_motion_tick_disposition())
    }

    pub(crate) fn native_motion_sampling_active(&self) -> bool {
        self.session.mounted.has_active_motion_samples()
            || self.session.portal_exit_terminal_work_pending()
    }

    pub(crate) fn native_motion_sample_presentation_pending(&self) -> bool {
        self.session.mounted.motion_sample_presentation_pending()
            || self.session.portal_exit_terminal_awaits_physical()
    }

    pub(crate) fn owns_pending_native_motion_physical(
        &self,
        class: worth_ui_host_native::UiNativePhysicalProgressClass,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> bool {
        presentation.is_some_and(|presentation| {
            self.session
                .mounted
                .pending_motion_sample_matches(presentation)
        }) || self
            .session
            .pending_portal_exit_terminal_matches_native_physical(class, presentation)
    }

    pub(crate) fn complete_pending_native_motion_physical(
        &mut self,
        class: worth_ui_host_native::UiNativePhysicalProgressClass,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> UiNativeMotionTickDisposition {
        use super::super::active_application_session::{
            UiPortalExitTerminalPending, UiPortalExitTerminalProgress,
        };

        if presentation.is_some_and(|presentation| {
            self.session
                .mounted
                .pending_motion_sample_matches(presentation)
        }) {
            self.session.complete_motion_sample_presentation();
            return self.native_motion_tick_disposition();
        }
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let progress = match self.session.take_portal_exit_terminal_pending() {
            Some(pending @ UiPortalExitTerminalPending::InFlight { .. }) => {
                self.session.retain_portal_exit_terminal_pending(pending);
                self.session
                    .complete_portal_exit_terminal_physical(self.managed_rebind_completion_tick)
            }
            Some(UiPortalExitTerminalPending::Indeterminate { track, recovery }) => {
                let session_identity = recovery.session_identity();
                let (frame, proposal) = recovery.into_parts();
                match self.progress_indeterminate_presentation_recovery_with_correlation(
                    frame,
                    presentation,
                    u64::MAX,
                    self.managed_rebind_completion_tick,
                ) {
                    super::WorthUiNativePhysicalPresentationRecovery::Awaiting(frame)
                    | super::WorthUiNativePhysicalPresentationRecovery::Blocked { frame, .. } => {
                        self.session.retain_portal_exit_terminal_pending(
                            UiPortalExitTerminalPending::Indeterminate {
                                track,
                                recovery: super::super::portal_dismissal::DetachedUiPortalDismissalIndeterminate::from_parts(
                                    session_identity,
                                    frame,
                                    proposal,
                                ),
                            },
                        );
                        UiPortalExitTerminalProgress::AwaitingPhysical
                    }
                    super::WorthUiNativePhysicalPresentationRecovery::Recovered(outcome) => self
                        .session
                        .settle_portal_exit_reconstruction(track, proposal, outcome),
                }
            }
            Some(UiPortalExitTerminalPending::Reconstruction {
                track,
                proposal,
                in_flight,
            }) => {
                let progress_class = match class {
                    worth_ui_host_native::UiNativePhysicalProgressClass::Presentation => {
                        worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface
                    }
                    worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas => {
                        worth_ui_host_contract::UiHostPresentationProgressClass::TextAtlas
                    }
                    worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery => {
                        unreachable!("reconstruction exact-match rejects recovery-only progress")
                    }
                };
                assert!(in_flight.awaits_progress_class(progress_class));
                let outcome = self
                    .session
                    .complete_mounted_presentation(in_flight, self.managed_rebind_completion_tick);
                self.session
                    .settle_portal_exit_reconstruction(track, proposal, outcome)
            }
            Some(pending @ UiPortalExitTerminalPending::Retry(_)) => {
                self.session.retain_portal_exit_terminal_pending(pending);
                UiPortalExitTerminalProgress::Retry
            }
            None => UiPortalExitTerminalProgress::Idle,
        };
        match progress {
            UiPortalExitTerminalProgress::AwaitingPhysical => {
                return UiNativeMotionTickDisposition::AwaitingPhysicalCompletion;
            }
            UiPortalExitTerminalProgress::Retry => return UiNativeMotionTickDisposition::Active,
            UiPortalExitTerminalProgress::Published | UiPortalExitTerminalProgress::Idle => {}
        }
        self.native_motion_tick_disposition()
    }

    fn current_motion_presentation(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.session
            .mounted
            .current_publication()?
            .presentation_for_surface(self.surface)
    }

    fn native_motion_tick_disposition(&self) -> UiNativeMotionTickDisposition {
        if self.native_motion_sample_presentation_pending() {
            UiNativeMotionTickDisposition::AwaitingPhysicalCompletion
        } else if self.native_motion_sampling_active() {
            UiNativeMotionTickDisposition::Active
        } else {
            UiNativeMotionTickDisposition::Inactive
        }
    }
}

const fn map_reduced_motion(
    posture: worth_ui_host_native::UiNativeReducedMotionPosture,
) -> crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture {
    match posture {
        worth_ui_host_native::UiNativeReducedMotionPosture::Reduce => {
            crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture::Reduce
        }
        worth_ui_host_native::UiNativeReducedMotionPosture::NoPreference
        | worth_ui_host_native::UiNativeReducedMotionPosture::Unavailable => {
            crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture::NoPreference
        }
    }
}

const fn map_reduced_motion_inspection(
    posture: worth_ui_host_native::UiNativeReducedMotionPosture,
) -> super::WorthUiNativeReducedMotionPosture {
    match posture {
        worth_ui_host_native::UiNativeReducedMotionPosture::NoPreference => {
            super::WorthUiNativeReducedMotionPosture::NoPreference
        }
        worth_ui_host_native::UiNativeReducedMotionPosture::Reduce => {
            super::WorthUiNativeReducedMotionPosture::Reduce
        }
        worth_ui_host_native::UiNativeReducedMotionPosture::Unavailable => {
            super::WorthUiNativeReducedMotionPosture::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::map_reduced_motion;
    use crate::mounting::presentation::motion_sampling::UiPresentationReducedMotionPosture;

    #[test]
    fn unavailable_system_posture_preserves_the_sampler_default() {
        assert_eq!(
            map_reduced_motion(worth_ui_host_native::UiNativeReducedMotionPosture::Unavailable),
            UiPresentationReducedMotionPosture::NoPreference
        );
        assert_eq!(
            map_reduced_motion(worth_ui_host_native::UiNativeReducedMotionPosture::Reduce),
            UiPresentationReducedMotionPosture::Reduce
        );
    }
}

use super::{
    WorthUiNativeManagedPortalDismissalOutcome, WorthUiNativeManagedRebindProgress,
    WorthUiNativePendingManagedRebind,
};

impl crate::facade::entry::WorthUiNativeApplicationShell {
    pub(in crate::facade::entry::native_managed_rebind) fn progress_indeterminate_portal_dismissal(
        &mut self,
        pending: crate::facade::entry::portal_dismissal::DetachedUiPortalDismissalIndeterminate,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::super::WorthUiNativeManagedRebindDenial>
    {
        if pending.session_identity() != self.session.session_identity() {
            return Err(super::super::WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        let session = pending.session_identity();
        let (frame, proposal) = pending.into_parts();
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let recovery = self.progress_indeterminate_presentation_recovery(
            frame,
            progress,
            u64::MAX,
            self.managed_rebind_completion_tick,
        );
        match recovery {
            crate::facade::entry::native_application_shell::WorthUiNativePhysicalPresentationRecovery::Awaiting(frame) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(
                        crate::facade::entry::portal_dismissal::DetachedUiPortalDismissalIndeterminate::from_parts(
                            session, frame, proposal,
                        ),
                    ),
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            crate::facade::entry::native_application_shell::WorthUiNativePhysicalPresentationRecovery::Blocked {
                frame,
                denial,
            } => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(
                        crate::facade::entry::portal_dismissal::DetachedUiPortalDismissalIndeterminate::from_parts(
                            session, frame, proposal,
                        ),
                    ),
                );
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(denial))
            }
            crate::facade::entry::native_application_shell::WorthUiNativePhysicalPresentationRecovery::Recovered(outcome) => {
                self.finish_portal_dismissal_recovery(proposal, outcome)
            }
        }
    }

    pub(in crate::facade::entry::native_managed_rebind) fn progress_portal_dismissal_reconstruction(
        &mut self,
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::super::WorthUiNativeManagedRebindDenial>
    {
        if !matches_reconstruction_progress(&in_flight, progress) {
            self.pending_managed_rebind = Some(
                WorthUiNativePendingManagedRebind::PortalDismissalReconstruction {
                    proposal,
                    in_flight,
                },
            );
            return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
        }
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let outcome = self
            .session
            .complete_mounted_presentation(in_flight, self.managed_rebind_completion_tick);
        self.finish_portal_dismissal_recovery(proposal, outcome)
    }

    pub(in crate::facade::entry::native_managed_rebind) fn progress_deferred_portal_dismissal_reconstruction(
        &mut self,
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::super::WorthUiNativeManagedRebindDenial>
    {
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        match self.reconstruct_current_presentation(u64::MAX, self.managed_rebind_completion_tick) {
            Ok(outcome) => self.finish_portal_dismissal_recovery(proposal, outcome),
            Err(()) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::PortalDismissalReconstructionDeferred {
                        proposal,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(
                    crate::facade::entry::native_application_shell::WorthUiNativePresentationRecoveryDenial::CurrentPresentationUnavailable,
                ))
            }
        }
    }

    fn finish_portal_dismissal_recovery(
        &mut self,
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
        outcome: crate::mounting::UiMountedFrameOutcome,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::super::WorthUiNativeManagedRebindDenial>
    {
        match outcome {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::PortalDismissalReconstruction {
                        proposal,
                        in_flight,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
                self.session
                    .application
                    .settle_indeterminate_portal_service_proposal_to_predecessor(
                        proposal,
                        self.session
                            .focus
                            .as_mut()
                            .expect("retained proposal owns Focus"),
                        self.session
                            .motion
                            .as_mut()
                            .expect("retained proposal owns Motion"),
                    );
                Ok(self.replay_retained_portal_dismissal_after_recovery())
            }
            _ => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::PortalDismissalReconstructionDeferred {
                        proposal,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(
                    crate::facade::entry::native_application_shell::WorthUiNativePresentationRecoveryDenial::CurrentPresentationUnavailable,
                ))
            }
        }
    }

    fn replay_retained_portal_dismissal_after_recovery(
        &mut self,
    ) -> WorthUiNativeManagedRebindProgress {
        let Some(retained) = self.retained_portal_dismissal.take() else {
            return WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(
                super::super::WorthUiNativePredecessorRecovery::PortalDismissal,
            );
        };
        match self.begin_managed_portal_dismissal(retained, self.managed_rebind_completion_tick) {
            WorthUiNativeManagedPortalDismissalOutcome::Ignored
            | WorthUiNativeManagedPortalDismissalOutcome::Stopped(
                super::WorthUiNativePortalDismissalStop::InteractionCancelled,
            ) => WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(
                super::super::WorthUiNativePredecessorRecovery::PortalDismissal,
            ),
            WorthUiNativeManagedPortalDismissalOutcome::Retained => {
                unreachable!("recovery replay runs without another managed intent consequence")
            }
            WorthUiNativeManagedPortalDismissalOutcome::Published(receipt) => {
                WorthUiNativeManagedRebindProgress::PortalDismissed(receipt)
            }
            WorthUiNativeManagedPortalDismissalOutcome::Pending => {
                WorthUiNativeManagedRebindProgress::AwaitingProgress
            }
            WorthUiNativeManagedPortalDismissalOutcome::Stopped(stop) => {
                WorthUiNativeManagedRebindProgress::Stopped(
                    super::super::WorthUiNativeManagedRebindStop::PortalDismissal(stop),
                )
            }
        }
    }
}

fn matches_reconstruction_progress(
    in_flight: &crate::mounting::UiMountedPresentationInFlight,
    progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
) -> bool {
    let class = match progress.class() {
        worth_ui_host_native::UiNativePhysicalProgressClass::Presentation => {
            worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface
        }
        worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas => {
            worth_ui_host_contract::UiHostPresentationProgressClass::TextAtlas
        }
        worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery => return false,
    };
    in_flight.awaits_progress_class(class)
        && progress.presentation().is_none_or(|presentation| {
            presentation.attempt() == in_flight.attempt()
                && in_flight
                    .pending_bindings()
                    .any(|binding| binding == presentation.binding())
        })
}

use super::{
    intent_posture, reconstruction_matches_progress, reconstruction_settled,
    WorthUiNativeApplicationShell, WorthUiNativeManagedRebindDenial,
    WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindStop,
    WorthUiNativePendingManagedRebind,
};

type Pending = crate::facade::entry::native_intent_posture::DetachedNativeIntentPosturePending;
type BeginOutcome =
    crate::facade::entry::native_intent_posture::WorthUiNativeManagedIntentPosturePublicationOutcome;
type BeginDenial =
    crate::facade::entry::native_intent_posture::WorthUiNativeManagedIntentPosturePublicationDenial;

impl WorthUiNativeApplicationShell {
    pub(in crate::facade::entry) fn begin_intent_posture_predecessor_reconstruction(
        &mut self,
        retry: Pending,
    ) -> Result<BeginOutcome, BeginDenial> {
        if retry.session_identity() != self.session.session_identity() {
            return Err(BeginDenial::ManagedRebindSessionMismatch);
        }
        let recovery = self
            .reconstruct_current_presentation(u64::MAX, self.managed_rebind_completion_tick)
            .map_err(|()| BeginDenial::PredecessorReconstruction)?;
        match recovery {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstruction {
                        retry,
                        in_flight,
                    },
                );
                Ok(BeginOutcome::Pending)
            }
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected)
                if text_atlas_deferred(rejected.rejections()) =>
            {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::
                        IntentPosturePredecessorReconstructionDeferred(retry),
                );
                Ok(BeginOutcome::Pending)
            }
            crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentPosturePredecessorIndeterminate {
                        retry,
                        frame,
                    },
                );
                Ok(BeginOutcome::Pending)
            }
            outcome if reconstruction_settled(&outcome) => {
                self.managed_rebind_completion_tick =
                    self.managed_rebind_completion_tick.saturating_add(1);
                let outcome =
                    retry.complete(&mut self.session, self.managed_rebind_completion_tick);
                let normalized = intent_posture::normalize_managed_intent_posture(outcome);
                Ok(self.finish_reconstructed_intent_posture_begin(normalized))
            }
            _ => Ok(BeginOutcome::Stopped(
                WorthUiNativeManagedRebindStop::PredecessorReconstructionFailed,
            )),
        }
    }

    pub(super) fn progress_intent_posture_predecessor_reconstruction(
        &mut self,
        retry: Pending,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        if retry.session_identity() != self.session.session_identity() {
            return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        if !reconstruction_matches_progress(&in_flight, progress) {
            self.pending_managed_rebind = Some(
                WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstruction {
                    retry,
                    in_flight,
                },
            );
            return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
        }
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let recovery = self
            .session
            .complete_mounted_presentation(in_flight, self.managed_rebind_completion_tick);
        match recovery {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstruction {
                        retry,
                        in_flight,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            outcome if reconstruction_settled(&outcome) => {
                let outcome =
                    retry.complete(&mut self.session, self.managed_rebind_completion_tick);
                Ok(intent_posture::finish(
                    &mut self.pending_managed_rebind,
                    outcome,
                ))
            }
            _ => Ok(WorthUiNativeManagedRebindProgress::Stopped(
                WorthUiNativeManagedRebindStop::PredecessorReconstructionFailed,
            )),
        }
    }

    pub(super) fn progress_deferred_intent_posture_predecessor_reconstruction(
        &mut self,
        retry: Pending,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        if retry.session_identity() != self.session.session_identity() {
            return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        if progress.class() != worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas {
            self.pending_managed_rebind = Some(
                WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstructionDeferred(
                    retry,
                ),
            );
            return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
        }
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let recovery = self
            .reconstruct_current_presentation(u64::MAX, self.managed_rebind_completion_tick)
            .map_err(|()| WorthUiNativeManagedRebindDenial::PredecessorReconstruction)?;
        match recovery {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstruction {
                        retry,
                        in_flight,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected)
                if text_atlas_deferred(rejected.rejections()) =>
            {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::
                        IntentPosturePredecessorReconstructionDeferred(retry),
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            outcome if reconstruction_settled(&outcome) => {
                let outcome =
                    retry.complete(&mut self.session, self.managed_rebind_completion_tick);
                Ok(intent_posture::finish(
                    &mut self.pending_managed_rebind,
                    outcome,
                ))
            }
            _ => Ok(WorthUiNativeManagedRebindProgress::Stopped(
                WorthUiNativeManagedRebindStop::PredecessorReconstructionFailed,
            )),
        }
    }

    pub(super) fn progress_indeterminate_intent_posture_predecessor_reconstruction(
        &mut self,
        retry: Pending,
        frame: crate::mounting::UiMountedIndeterminateFrame,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        if retry.session_identity() != self.session.session_identity() {
            return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let recovery = self.progress_indeterminate_presentation_recovery(
            frame,
            progress,
            u64::MAX,
            self.managed_rebind_completion_tick,
        );
        let recovery = match recovery {
            crate::facade::entry::native_application_shell::
                WorthUiNativePhysicalPresentationRecovery::Awaiting(frame) =>
            {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentPosturePredecessorIndeterminate {
                        retry,
                        frame,
                    },
                );
                return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
            }
            crate::facade::entry::native_application_shell::
                WorthUiNativePhysicalPresentationRecovery::Blocked { frame, denial } =>
            {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentPosturePredecessorIndeterminate {
                        retry,
                        frame,
                    },
                );
                return Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(denial));
            }
            crate::facade::entry::native_application_shell::
                WorthUiNativePhysicalPresentationRecovery::Recovered(outcome) => outcome,
        };
        match recovery {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstruction {
                        retry,
                        in_flight,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected)
                if text_atlas_deferred(rejected.rejections()) =>
            {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::
                        IntentPosturePredecessorReconstructionDeferred(retry),
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            outcome if reconstruction_settled(&outcome) => {
                let outcome =
                    retry.complete(&mut self.session, self.managed_rebind_completion_tick);
                Ok(intent_posture::finish(
                    &mut self.pending_managed_rebind,
                    outcome,
                ))
            }
            _ => Ok(WorthUiNativeManagedRebindProgress::Stopped(
                WorthUiNativeManagedRebindStop::PredecessorReconstructionFailed,
            )),
        }
    }

    fn finish_reconstructed_intent_posture_begin(
        &mut self,
        normalized: intent_posture::ManagedIntentPostureNormalization,
    ) -> BeginOutcome {
        match normalized {
            intent_posture::ManagedIntentPostureNormalization::Published(receipt) => {
                BeginOutcome::Published(receipt)
            }
            intent_posture::ManagedIntentPostureNormalization::Pending(pending) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::IntentPosture(pending));
                BeginOutcome::Pending
            }
            intent_posture::ManagedIntentPostureNormalization::ReconstructionRequired(_) => {
                BeginOutcome::Stopped(
                    WorthUiNativeManagedRebindStop::PredecessorReconstructionFailed,
                )
            }
            intent_posture::ManagedIntentPostureNormalization::Indeterminate {
                recovery,
                frame,
            } => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame });
                BeginOutcome::Pending
            }
            intent_posture::ManagedIntentPostureNormalization::Stopped(stop) => {
                BeginOutcome::Stopped(stop)
            }
        }
    }
}

fn text_atlas_deferred(
    rejections: &[crate::mounting::UiMountedSurfacePresentationRejection],
) -> bool {
    !rejections.is_empty()
        && rejections.iter().all(|rejection| {
            rejection.denial()
                == worth_ui_host_contract::UiHostSurfacePresentationDenial::
                    TextAtlasPresentationDeferred
        })
}

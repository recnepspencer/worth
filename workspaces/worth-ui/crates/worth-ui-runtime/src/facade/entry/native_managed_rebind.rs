use super::WorthUiNativeApplicationShell;

#[path = "native_managed_rebind/indeterminate.rs"]
mod indeterminate;
#[path = "native_managed_rebind/intent_consequence.rs"]
mod intent_consequence;
#[path = "native_managed_rebind/intent_posture.rs"]
mod intent_posture;
#[path = "native_managed_rebind/intent_posture_reconstruction.rs"]
mod intent_posture_reconstruction;
#[path = "native_managed_rebind/model.rs"]
mod model;
#[path = "native_managed_rebind/normalization.rs"]
mod normalization;
#[path = "native_managed_rebind/portal_dismissal.rs"]
mod portal_dismissal;
#[path = "native_managed_rebind/predecessor_reconstruction.rs"]
mod predecessor_reconstruction;
#[path = "native_managed_rebind/reconstruction.rs"]
mod reconstruction;
#[path = "native_managed_rebind/shutdown.rs"]
mod shutdown;
pub(super) use intent_consequence::{
    normalize_managed_intent_consequence, ManagedIntentConsequenceNormalization,
};
pub(super) use intent_posture::{
    normalize_managed_intent_posture, ManagedIntentPostureNormalization,
};
pub(super) use model::WorthUiNativePendingManagedRebind;
pub use model::{
    WorthUiNativeManagedRebindDenial, WorthUiNativeManagedRebindProgress,
    WorthUiNativeManagedRebindStop, WorthUiNativePredecessorRecovery,
};
pub(super) use normalization::{
    finish_normalized_managed_rebind, normalize_managed_outcome, retain_normalized_managed_rebind,
    ManagedRebindNormalization,
};
pub use portal_dismissal::{
    WorthUiNativeManagedPortalDismissalOutcome, WorthUiNativePortalDismissalStop,
};
use predecessor_reconstruction::{reconstruction_matches_progress, reconstruction_settled};
pub(super) use reconstruction::{
    detach_required_surface_reconstruction, RequiredSurfaceReconstruction,
};

impl WorthUiNativeApplicationShell {
    pub fn retry_managed_rebind(
        &mut self,
        now_tick: u64,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        let Some(pending) = self.pending_managed_rebind.take() else {
            return Ok(WorthUiNativeManagedRebindProgress::Unrelated);
        };
        let WorthUiNativePendingManagedRebind::Retry {
            retry,
            requires_reconstruction,
        } = pending
        else {
            self.pending_managed_rebind = Some(pending);
            return Ok(WorthUiNativeManagedRebindProgress::Unrelated);
        };
        let progress = self.progress_managed_retry(retry, requires_reconstruction, now_tick)?;
        Ok(self.finalize_managed_rebind_progress(progress))
    }

    pub fn progress_managed_rebind(
        &mut self,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        let progress = self.progress_managed_rebind_internal(progress)?;
        Ok(self.finalize_managed_rebind_progress(progress))
    }

    fn finalize_managed_rebind_progress(
        &mut self,
        progress: WorthUiNativeManagedRebindProgress,
    ) -> WorthUiNativeManagedRebindProgress {
        if let WorthUiNativeManagedRebindProgress::Published(receipt) = &progress {
            self.settle_native_rebind_reconciliation(receipt);
        }
        progress
    }

    fn progress_managed_rebind_internal(
        &mut self,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        let Some(pending) = self.pending_managed_rebind.take() else {
            return Ok(WorthUiNativeManagedRebindProgress::Unrelated);
        };
        match pending {
            WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame } => {
                self.progress_indeterminate_rebind(recovery, frame, progress)
            }
            WorthUiNativePendingManagedRebind::RecoveryReconstruction {
                recovery,
                in_flight,
            } => self.progress_rebind_recovery_reconstruction(recovery, in_flight, progress),
            WorthUiNativePendingManagedRebind::RecoveryReconstructionDeferred(recovery) => {
                self.progress_deferred_rebind_recovery(recovery)
            }
            WorthUiNativePendingManagedRebind::Completion(pending) => {
                if pending.session_identity() != self.session.session_identity() {
                    return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
                }
                if !pending.matches_native_progress(progress) {
                    self.pending_managed_rebind =
                        Some(WorthUiNativePendingManagedRebind::Completion(pending));
                    return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
                }
                self.managed_rebind_completion_tick =
                    self.managed_rebind_completion_tick.saturating_add(1);
                let outcome =
                    pending.complete(&mut self.session, self.managed_rebind_completion_tick);
                let outcome = normalization::retry_progressed_text_atlas_deferral(
                    outcome,
                    self.managed_rebind_completion_tick,
                );
                let retry = match detach_required_surface_reconstruction(outcome) {
                    RequiredSurfaceReconstruction::NotRequired(outcome) => {
                        return Ok(finish_normalized_managed_rebind(
                            &mut self.pending_managed_rebind,
                            outcome,
                        ));
                    }
                    RequiredSurfaceReconstruction::Required(retry) => retry,
                };
                self.begin_rebind_reconstruction(retry, self.managed_rebind_completion_tick)
            }
            WorthUiNativePendingManagedRebind::Retry {
                retry,
                requires_reconstruction,
            } => {
                self.managed_rebind_completion_tick =
                    self.managed_rebind_completion_tick.saturating_add(1);
                self.progress_managed_retry(
                    retry,
                    requires_reconstruction,
                    self.managed_rebind_completion_tick,
                )
            }
            WorthUiNativePendingManagedRebind::IntentPosture(pending) => {
                if pending.session_identity() != self.session.session_identity() {
                    return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
                }
                if !pending.matches_native_progress(progress) {
                    self.pending_managed_rebind =
                        Some(WorthUiNativePendingManagedRebind::IntentPosture(pending));
                    return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
                }
                self.managed_rebind_completion_tick =
                    self.managed_rebind_completion_tick.saturating_add(1);
                let outcome =
                    pending.complete(&mut self.session, self.managed_rebind_completion_tick);
                let outcome = intent_posture::retry_progressed_text_atlas_deferral(
                    outcome,
                    self.managed_rebind_completion_tick,
                );
                Ok(intent_posture::finish(
                    &mut self.pending_managed_rebind,
                    outcome,
                ))
            }
            WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstruction {
                retry,
                in_flight,
            } => {
                self.progress_intent_posture_predecessor_reconstruction(retry, in_flight, progress)
            }
            WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstructionDeferred(
                retry,
            ) => self.progress_deferred_intent_posture_predecessor_reconstruction(retry, progress),
            WorthUiNativePendingManagedRebind::IntentPosturePredecessorIndeterminate {
                retry,
                frame,
            } => self.progress_indeterminate_intent_posture_predecessor_reconstruction(
                retry, frame, progress,
            ),
            WorthUiNativePendingManagedRebind::IntentConsequence(pending) => {
                if pending.session_identity() != self.session.session_identity() {
                    return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
                }
                if !pending.matches_native_progress(progress) {
                    self.pending_managed_rebind = Some(
                        WorthUiNativePendingManagedRebind::IntentConsequence(pending),
                    );
                    return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
                }
                self.managed_rebind_completion_tick =
                    self.managed_rebind_completion_tick.saturating_add(1);
                let outcome =
                    pending.complete(&mut self.session, self.managed_rebind_completion_tick);
                Ok(intent_consequence::finish(
                    &mut self.pending_managed_rebind,
                    outcome,
                ))
            }
            WorthUiNativePendingManagedRebind::IntentConsequenceIndeterminate(pending) => {
                self.progress_indeterminate_intent_consequence(pending, progress)
            }
            WorthUiNativePendingManagedRebind::IntentConsequenceReconstruction {
                portal,
                resources,
                in_flight,
            } => self
                .progress_intent_consequence_reconstruction(portal, resources, in_flight, progress),
            WorthUiNativePendingManagedRebind::IntentConsequenceReconstructionDeferred {
                portal,
                resources,
            } => self.progress_deferred_intent_consequence_reconstruction(portal, resources),
            WorthUiNativePendingManagedRebind::PortalDismissal(pending) => {
                if pending.session_identity() != self.session.session_identity() {
                    return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
                }
                if !pending.matches_native_progress(progress) {
                    self.pending_managed_rebind =
                        Some(WorthUiNativePendingManagedRebind::PortalDismissal(pending));
                    return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
                }
                self.managed_rebind_completion_tick =
                    self.managed_rebind_completion_tick.saturating_add(1);
                let outcome =
                    pending.complete(&mut self.session, self.managed_rebind_completion_tick);
                let progress = portal_dismissal::finish(&mut self.pending_managed_rebind, outcome);
                if matches!(
                    progress,
                    WorthUiNativeManagedRebindProgress::PortalDismissed(_)
                ) {
                    self.retained_portal_dismissal = None;
                }
                Ok(progress)
            }
            WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(pending) => {
                self.progress_indeterminate_portal_dismissal(pending, progress)
            }
            WorthUiNativePendingManagedRebind::PortalDismissalReconstruction {
                proposal,
                in_flight,
            } => self.progress_portal_dismissal_reconstruction(proposal, in_flight, progress),
            WorthUiNativePendingManagedRebind::PortalDismissalReconstructionDeferred {
                proposal,
            } => self.progress_deferred_portal_dismissal_reconstruction(proposal),
            WorthUiNativePendingManagedRebind::PredecessorReconstruction { retry, in_flight } => {
                if retry.session_identity() != self.session.session_identity() {
                    return Err(WorthUiNativeManagedRebindDenial::SessionMismatch);
                }
                if !reconstruction_matches_progress(&in_flight, progress) {
                    self.pending_managed_rebind = Some(
                        WorthUiNativePendingManagedRebind::PredecessorReconstruction {
                            retry,
                            in_flight,
                        },
                    );
                    return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
                }
                self.managed_rebind_completion_tick =
                    self.managed_rebind_completion_tick.saturating_add(1);
                let recovery = self
                    .session
                    .complete_mounted_presentation(in_flight, self.managed_rebind_completion_tick);
                match recovery {
                    crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                        self.pending_managed_rebind = Some(
                            WorthUiNativePendingManagedRebind::PredecessorReconstruction {
                                retry,
                                in_flight,
                            },
                        );
                        Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
                    }
                    outcome if reconstruction_settled(&outcome) => {
                        let outcome = retry
                            .rebase_content_and_retry(
                                &mut self.session,
                                self.managed_rebind_completion_tick,
                            )
                            .map_err(WorthUiNativeManagedRebindDenial::Preparation)?;
                        Ok(finish_normalized_managed_rebind(
                            &mut self.pending_managed_rebind,
                            outcome,
                        ))
                    }
                    _ => Ok(WorthUiNativeManagedRebindProgress::Stopped(
                        WorthUiNativeManagedRebindStop::PredecessorReconstructionFailed,
                    )),
                }
            }
        }
    }

    fn progress_managed_retry(
        &mut self,
        retry: crate::runtime::rebind::UiDetachedRebindRetry,
        requires_reconstruction: bool,
        now_tick: u64,
    ) -> Result<WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindDenial> {
        if requires_reconstruction {
            return self.begin_rebind_reconstruction(retry, now_tick);
        }
        let outcome = retry
            .rebase_content_and_retry(&mut self.session, now_tick)
            .map_err(WorthUiNativeManagedRebindDenial::Preparation)?;
        Ok(finish_normalized_managed_rebind(
            &mut self.pending_managed_rebind,
            outcome,
        ))
    }
}

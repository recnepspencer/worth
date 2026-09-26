use super::{
    WorthUiNativeManagedRebindProgress, WorthUiNativeManagedRebindStop,
    WorthUiNativePendingManagedRebind, WorthUiNativePredecessorRecovery,
};

pub(in crate::facade::entry) enum ManagedIntentConsequenceNormalization {
    NoConsequences(crate::runtime::intent_execution::UiIntentConsequenceCompletionReceipt),
    Published(
        crate::facade::entry::intent_consequence_publication::UiIntentConsequencePublicationReceipt,
    ),
    Pending(
        crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceInFlight,
    ),
    Indeterminate(
        crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceIndeterminate,
    ),
    Stopped(WorthUiNativeManagedRebindStop),
}

pub(in crate::facade::entry) fn normalize_managed_intent_consequence(
    outcome: crate::facade::entry::UiIntentConsequencePublicationOutcome<'_>,
) -> ManagedIntentConsequenceNormalization {
    use crate::facade::entry::UiIntentConsequencePublicationOutcome as Outcome;
    match outcome {
        Outcome::NoConsequences(receipt) => {
            ManagedIntentConsequenceNormalization::NoConsequences(receipt)
        }
        Outcome::Published(receipt) => ManagedIntentConsequenceNormalization::Published(receipt),
        Outcome::InFlight(completion) => {
            ManagedIntentConsequenceNormalization::Pending(completion.detach_for_native())
        }
        Outcome::Stopped(stop) => {
            let (reason, recovery) = stop.into_parts();
            drop(recovery);
            ManagedIntentConsequenceNormalization::Stopped(
                WorthUiNativeManagedRebindStop::IntentConsequence(reason),
            )
        }
        Outcome::Indeterminate(recovery) => {
            ManagedIntentConsequenceNormalization::Indeterminate(recovery.detach_for_native())
        }
        Outcome::InternalDefect(defect) => ManagedIntentConsequenceNormalization::Stopped(
            WorthUiNativeManagedRebindStop::InternalDefect(defect.kind()),
        ),
    }
}

pub(super) fn finish(
    pending: &mut Option<WorthUiNativePendingManagedRebind>,
    outcome: crate::facade::entry::UiIntentConsequencePublicationOutcome<'_>,
) -> WorthUiNativeManagedRebindProgress {
    match normalize_managed_intent_consequence(outcome) {
        ManagedIntentConsequenceNormalization::Published(receipt) => {
            WorthUiNativeManagedRebindProgress::IntentConsequencePublished(receipt)
        }
        ManagedIntentConsequenceNormalization::Pending(completion) => {
            *pending = Some(WorthUiNativePendingManagedRebind::IntentConsequence(
                completion,
            ));
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        }
        ManagedIntentConsequenceNormalization::Indeterminate(recovery) => {
            *pending =
                Some(WorthUiNativePendingManagedRebind::IntentConsequenceIndeterminate(recovery));
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        }
        ManagedIntentConsequenceNormalization::Stopped(stop) => {
            WorthUiNativeManagedRebindProgress::Stopped(stop)
        }
        ManagedIntentConsequenceNormalization::NoConsequences(_) => {
            unreachable!("admitted consequence publication cannot become consequence-free")
        }
    }
}

impl super::super::WorthUiNativeApplicationShell {
    pub(super) fn progress_indeterminate_intent_consequence(
        &mut self,
        pending: crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceIndeterminate,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::WorthUiNativeManagedRebindDenial> {
        if pending.session_identity() != self.session.session_identity() {
            return Err(super::WorthUiNativeManagedRebindDenial::SessionMismatch);
        }
        let (session, frame, portal, resources) = pending.into_parts();
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let recovery = self.progress_indeterminate_presentation_recovery(
            frame,
            progress,
            u64::MAX,
            self.managed_rebind_completion_tick,
        );
        match recovery {
            super::super::native_application_shell::WorthUiNativePhysicalPresentationRecovery::Awaiting(frame) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentConsequenceIndeterminate(
                        crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceIndeterminate::from_parts(
                            session, frame, portal, resources,
                        ),
                    ),
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            super::super::native_application_shell::WorthUiNativePhysicalPresentationRecovery::Blocked {
                frame,
                denial,
            } => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentConsequenceIndeterminate(
                        crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceIndeterminate::from_parts(
                            session, frame, portal, resources,
                        ),
                    ),
                );
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(denial))
            }
            super::super::native_application_shell::WorthUiNativePhysicalPresentationRecovery::Recovered(outcome) => {
                self.finish_intent_consequence_recovery(portal, resources, outcome)
            }
        }
    }

    pub(super) fn progress_intent_consequence_reconstruction(
        &mut self,
        portal: Option<crate::runtime::session::UiIndeterminatePortalProposalTransaction>,
        resources: crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceRecoveryResources,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::WorthUiNativeManagedRebindDenial> {
        if !super::reconstruction_matches_progress(&in_flight, progress) {
            self.pending_managed_rebind = Some(
                WorthUiNativePendingManagedRebind::IntentConsequenceReconstruction {
                    portal,
                    resources,
                    in_flight,
                },
            );
            return Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress);
        }
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        let outcome = self
            .session
            .complete_mounted_presentation(in_flight, self.managed_rebind_completion_tick);
        self.finish_intent_consequence_recovery(portal, resources, outcome)
    }

    pub(super) fn progress_deferred_intent_consequence_reconstruction(
        &mut self,
        portal: Option<crate::runtime::session::UiIndeterminatePortalProposalTransaction>,
        resources: crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceRecoveryResources,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::WorthUiNativeManagedRebindDenial> {
        self.managed_rebind_completion_tick = self.managed_rebind_completion_tick.saturating_add(1);
        match self.reconstruct_current_presentation(u64::MAX, self.managed_rebind_completion_tick) {
            Ok(outcome) => self.finish_intent_consequence_recovery(portal, resources, outcome),
            Err(()) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentConsequenceReconstructionDeferred {
                        portal,
                        resources,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(
                    super::super::native_application_shell::WorthUiNativePresentationRecoveryDenial::CurrentPresentationUnavailable,
                ))
            }
        }
    }

    fn finish_intent_consequence_recovery(
        &mut self,
        portal: Option<crate::runtime::session::UiIndeterminatePortalProposalTransaction>,
        resources: crate::facade::entry::intent_consequence_publication::DetachedUiIntentConsequenceRecoveryResources,
        outcome: crate::mounting::UiMountedFrameOutcome,
    ) -> Result<WorthUiNativeManagedRebindProgress, super::WorthUiNativeManagedRebindDenial> {
        match outcome {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentConsequenceReconstruction {
                        portal,
                        resources,
                        in_flight,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::AwaitingProgress)
            }
            crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
                if let Some(portal) = portal {
                    self.session
                        .application
                        .settle_indeterminate_portal_service_proposal_to_predecessor(
                            portal,
                            self.session
                                .focus
                                .as_mut()
                                .expect("indeterminate proposal retains Focus installation"),
                            self.session
                                .motion
                                .as_mut()
                                .expect("indeterminate proposal retains Motion installation"),
                        );
                }
                resources.settle_predecessor(&mut self.session);
                Ok(WorthUiNativeManagedRebindProgress::RecoveredToPredecessor(
                    WorthUiNativePredecessorRecovery::IntentConsequence,
                ))
            }
            crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(_)
            | crate::mounting::UiMountedFrameOutcome::Superseded(_)
            | crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
                self.pending_managed_rebind = Some(
                    WorthUiNativePendingManagedRebind::IntentConsequenceReconstructionDeferred {
                        portal,
                        resources,
                    },
                );
                Ok(WorthUiNativeManagedRebindProgress::RecoveryBlocked(
                    super::super::native_application_shell::WorthUiNativePresentationRecoveryDenial::CurrentPresentationUnavailable,
                ))
            }
        }
    }
}

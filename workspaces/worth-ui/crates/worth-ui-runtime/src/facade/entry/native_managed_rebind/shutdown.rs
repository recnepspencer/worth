use super::{WorthUiNativeApplicationShell, WorthUiNativePendingManagedRebind};

impl WorthUiNativeApplicationShell {
    pub(in crate::facade::entry) fn cancel_managed_rebind_for_shutdown(&mut self) {
        self.retained_portal_dismissal = None;
        let Some(pending) = self.pending_managed_rebind.take() else {
            return;
        };
        match pending {
            WorthUiNativePendingManagedRebind::Indeterminate { recovery, frame } => {
                drop((recovery, frame))
            }
            WorthUiNativePendingManagedRebind::RecoveryReconstruction {
                recovery,
                in_flight,
            } => {
                drop(self.session.cancel_mounted_presentation(in_flight));
                drop(recovery);
            }
            WorthUiNativePendingManagedRebind::RecoveryReconstructionDeferred(recovery) => {
                drop(recovery)
            }
            WorthUiNativePendingManagedRebind::Completion(pending) => {
                drop(pending.cancel(&mut self.session));
            }
            WorthUiNativePendingManagedRebind::Retry { retry, .. } => drop(retry),
            WorthUiNativePendingManagedRebind::IntentPosture(pending) => {
                pending.cancel(&mut self.session);
            }
            WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstruction {
                retry,
                in_flight,
            } => {
                drop(self.session.cancel_mounted_presentation(in_flight));
                retry.cancel(&mut self.session);
            }
            WorthUiNativePendingManagedRebind::IntentPosturePredecessorReconstructionDeferred(
                retry,
            ) => retry.cancel(&mut self.session),
            WorthUiNativePendingManagedRebind::IntentPosturePredecessorIndeterminate {
                retry,
                frame,
            } => {
                drop(frame);
                retry.cancel(&mut self.session);
            }
            WorthUiNativePendingManagedRebind::IntentConsequence(pending) => {
                drop(pending.cancel(&mut self.session));
            }
            WorthUiNativePendingManagedRebind::IntentConsequenceIndeterminate(pending) => {
                let (_, frame, portal, resources) = pending.into_parts();
                drop(frame);
                if let Some(portal) = portal {
                    self.session
                        .application
                        .abandon_indeterminate_portal_service_proposal_for_shutdown(
                            portal,
                            self.session
                                .focus
                                .as_mut()
                                .expect("retained proposal owns Focus"),
                            self.session
                                .motion
                                .as_mut()
                                .expect("retained proposal owns Motion"),
                        );
                }
                resources.abandon_for_shutdown(&mut self.session);
            }
            WorthUiNativePendingManagedRebind::IntentConsequenceReconstruction {
                portal,
                resources,
                in_flight,
            } => {
                drop(self.session.cancel_mounted_presentation(in_flight));
                if let Some(portal) = portal {
                    self.session
                        .application
                        .abandon_indeterminate_portal_service_proposal_for_shutdown(
                            portal,
                            self.session
                                .focus
                                .as_mut()
                                .expect("retained proposal owns Focus"),
                            self.session
                                .motion
                                .as_mut()
                                .expect("retained proposal owns Motion"),
                        );
                }
                resources.abandon_for_shutdown(&mut self.session);
            }
            WorthUiNativePendingManagedRebind::IntentConsequenceReconstructionDeferred {
                portal,
                resources,
            } => {
                if let Some(portal) = portal {
                    self.session
                        .application
                        .abandon_indeterminate_portal_service_proposal_for_shutdown(
                            portal,
                            self.session
                                .focus
                                .as_mut()
                                .expect("retained proposal owns Focus"),
                            self.session
                                .motion
                                .as_mut()
                                .expect("retained proposal owns Motion"),
                        );
                }
                resources.abandon_for_shutdown(&mut self.session);
            }
            WorthUiNativePendingManagedRebind::PortalDismissal(pending) => {
                drop(pending.cancel(&mut self.session));
            }
            WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(pending) => {
                let (frame, proposal) = pending.into_parts();
                drop(frame);
                self.session
                    .application
                    .abandon_indeterminate_portal_service_proposal_for_shutdown(
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
            }
            WorthUiNativePendingManagedRebind::PortalDismissalReconstruction {
                proposal,
                in_flight,
            } => {
                drop(self.session.cancel_mounted_presentation(in_flight));
                self.session
                    .application
                    .abandon_indeterminate_portal_service_proposal_for_shutdown(
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
            }
            WorthUiNativePendingManagedRebind::PortalDismissalReconstructionDeferred {
                proposal,
            } => self
                .session
                .application
                .abandon_indeterminate_portal_service_proposal_for_shutdown(
                    proposal,
                    self.session
                        .focus
                        .as_mut()
                        .expect("retained proposal owns Focus"),
                    self.session
                        .motion
                        .as_mut()
                        .expect("retained proposal owns Motion"),
                ),
            WorthUiNativePendingManagedRebind::PredecessorReconstruction { retry, in_flight } => {
                drop(self.session.cancel_mounted_presentation(in_flight));
                drop(retry);
            }
        }
    }
}

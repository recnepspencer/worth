use crate::native::physical_work_signal::UiNativePhysicalSignalSettlement;

use super::{
    ready_token::ReadyPresentation, UiNativeHostState, UiNativePresentationPhysicalProgress,
};

impl UiNativeHostState {
    pub(super) fn settle_presentation_signal(
        &mut self,
        identity: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
        ready: ReadyPresentation,
        signal_settlement: UiNativePhysicalSignalSettlement,
    ) -> UiNativePresentationPhysicalProgress {
        match signal_settlement {
            UiNativePhysicalSignalSettlement::Completed => {
                if ready.resolving_recovery {
                    self.complete_presentation_recovery(ready)
                } else {
                    self.complete_presentation(identity, ready, signal_settlement)
                }
            }
            UiNativePhysicalSignalSettlement::Superseded => self.supersede_presentation(ready),
            UiNativePhysicalSignalSettlement::Indeterminate => {
                self.schedule_presentation_recovery(identity, ready)
            }
            UiNativePhysicalSignalSettlement::Rejected => self.reject_presentation(ready),
            UiNativePhysicalSignalSettlement::Pending | UiNativePhysicalSignalSettlement::Stale => {
                self.retain_pending_presentation(ready);
                UiNativePresentationPhysicalProgress::Pending
            }
        }
    }

    fn complete_presentation_recovery(
        &mut self,
        mut ready: ReadyPresentation,
    ) -> UiNativePresentationPhysicalProgress {
        ready.pending.release_external(&mut self.resources);
        self.retain_if_completion_pending(ready);
        UiNativePresentationPhysicalProgress::RecoveryCompleted
    }

    fn complete_presentation(
        &mut self,
        identity: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
        mut ready: ReadyPresentation,
        signal_settlement: UiNativePhysicalSignalSettlement,
    ) -> UiNativePresentationPhysicalProgress {
        let duplicate_observation =
            self.observe_certified_duplicate(identity, &mut ready.pending, signal_settlement);
        if ready.pending.has_settlement() {
            match ready.pending.take_presented_observation() {
                Some(observation) => ready.pending.mark_presented(observation),
                None => {
                    if let Some(settlement) = ready.pending.take_settlement() {
                        settlement.abandon(
                            self,
                            ready.pending.physical_basis(),
                            ready.pending.completion_identity(),
                        );
                    }

                    ready.pending.mark_indeterminate();
                }
            }
        }
        ready.pending.release_external(&mut self.resources);
        self.retain_if_completion_pending(ready);
        UiNativePresentationPhysicalProgress::Completed {
            duplicate_observation,
        }
    }

    fn supersede_presentation(
        &mut self,
        mut ready: ReadyPresentation,
    ) -> UiNativePresentationPhysicalProgress {
        if let Some(observation) = ready.pending.take_presented_observation() {
            self.resolve_superseded_settlement(&mut ready.pending);
            ready.pending.mark_superseded(observation);
        } else {
            if let Some(settlement) = ready.pending.take_settlement() {
                settlement.abandon(
                    self,
                    ready.pending.physical_basis(),
                    ready.pending.completion_identity(),
                );
            }

            ready.pending.mark_indeterminate();
        }
        ready.pending.release_external(&mut self.resources);
        self.retain_if_completion_pending(ready);
        UiNativePresentationPhysicalProgress::Superseded
    }

    fn resolve_superseded_settlement(
        &mut self,
        predecessor: &mut crate::native::UiNativePendingPresentation,
    ) {
        let Some(mut settlement) = predecessor.take_settlement() else {
            return;
        };
        let basis = predecessor.physical_basis();
        let predecessor_attempt = basis.attempt().diagnostic_value();
        for successor in &mut self.pending_presentations {
            let successor_basis = successor.physical_basis();
            if successor_basis.binding() != basis.binding()
                || successor_basis.attempt().diagnostic_value() <= predecessor_attempt
            {
                continue;
            }
            match successor
                .inherit_predecessor_settlement(settlement, predecessor.prepared_cursor())
            {
                Ok(()) => {
                    predecessor.replace_settlement(
                        crate::native::presentation::UiNativePendingSurfaceSettlement::SupersededDeltaResolved,
                    );
                    return;
                }
                Err(returned) => settlement = *returned,
            }
        }
        let committed_successor = self
            .presentation_epochs
            .get(&basis.binding().diagnostic_value())
            .is_some_and(|epoch| epoch.diagnostic_value() > predecessor_attempt);
        if committed_successor {
            settlement.commit_superseded_predecessor();
            crate::native::presentation::appearance::cursor::accept_cursor(
                self,
                basis.attempt(),
                predecessor.prepared_cursor(),
            );
        } else {
            settlement.rollback_superseded_predecessor(self, basis);
        }
        predecessor.replace_settlement(
            crate::native::presentation::UiNativePendingSurfaceSettlement::SupersededDeltaResolved,
        );
    }

    fn schedule_presentation_recovery(
        &mut self,
        identity: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
        mut ready: ReadyPresentation,
    ) -> UiNativePresentationPhysicalProgress {
        #[cfg(feature = "certification-support")]
        if self
            .qualification
            .presentation_poll_override(identity)
            .is_some()
        {
            self.qualification
                .commit_presentation_poll_override(identity);
        }
        if let Some(settlement) = ready.pending.take_settlement() {
            settlement.abandon(
                self,
                ready.pending.physical_basis(),
                ready.pending.completion_identity(),
            );
        }
        #[cfg(feature = "certification-support")]
        if let Some(class) = self.qualification.take_derived_state_loss() {
            let binding = ready.pending.physical_basis().binding().diagnostic_value();
            self.apply_qualified_derived_state_loss(binding, class);
        }

        ready.pending.mark_indeterminate();
        let Ok(recovery) = self
            .physical_signal
            .transition_presentation_to_recovery(identity)
        else {
            self.retain_pending_presentation(ready);
            return UiNativePresentationPhysicalProgress::NoProgress;
        };
        if !ready.pending.refresh_physical_token(recovery) {
            return UiNativePresentationPhysicalProgress::NoProgress;
        }
        self.retain_pending_presentation(ready);
        UiNativePresentationPhysicalProgress::IndeterminateRecoveryScheduled
    }

    fn reject_presentation(
        &mut self,
        mut ready: ReadyPresentation,
    ) -> UiNativePresentationPhysicalProgress {
        if let Some(settlement) = ready.pending.take_settlement() {
            settlement.abandon(
                self,
                ready.pending.physical_basis(),
                ready.pending.completion_identity(),
            );
        }

        ready.pending.mark_indeterminate();
        ready.pending.release_external(&mut self.resources);
        self.retain_if_completion_pending(ready);
        UiNativePresentationPhysicalProgress::Rejected
    }

    fn retain_if_completion_pending(&mut self, ready: ReadyPresentation) {
        if ready.pending.completion_identity().is_some() {
            self.retain_pending_presentation(ready);
        }
    }

    fn retain_pending_presentation(&mut self, ready: ReadyPresentation) {
        self.pending_presentations
            .insert(ready.index, ready.pending);
    }

    fn observe_certified_duplicate(
        &mut self,
        identity: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
        pending: &mut crate::native::UiNativePendingPresentation,
        signal_settlement: UiNativePhysicalSignalSettlement,
    ) -> bool {
        #[cfg(feature = "certification-support")]
        {
            if signal_settlement == UiNativePhysicalSignalSettlement::Completed {
                let Some(duplicate) = pending.take_duplicate_completed_observation() else {
                    return false;
                };
                assert_eq!(
                    self.physical_signal.reconcile(duplicate),
                    UiNativePhysicalSignalSettlement::Stale,
                    "the physical Signal must reject a repeated owner observation"
                );
                self.qualification
                    .commit_duplicate_completed_observation(identity);
                return true;
            }
        }
        #[cfg(not(feature = "certification-support"))]
        let _ = (identity, pending, signal_settlement);
        false
    }
}

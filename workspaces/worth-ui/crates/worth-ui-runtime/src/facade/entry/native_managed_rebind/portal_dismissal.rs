use super::{WorthUiNativeManagedRebindProgress, WorthUiNativePendingManagedRebind};

#[path = "portal_dismissal_recovery.rs"]
mod recovery;

#[derive(Clone, Copy)]
pub(in crate::facade::entry) struct UiRetainedPortalDismissalRequest {
    cause: crate::facade::interaction::UiDismissInteractionCause,
    sequence: worth_ui_host_contract::UiHostObservationSequence,
    time_basis: worth_ui_host_contract::UiHostObservationTimeBasis,
}

impl UiRetainedPortalDismissalRequest {
    fn retain(interaction: crate::facade::interaction::UiDismissInteraction) -> Self {
        Self {
            cause: interaction.cause(),
            sequence: interaction.sequence(),
            time_basis: interaction.time_basis(),
        }
    }

    fn rebase(
        self,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> crate::facade::interaction::UiDismissInteraction {
        match self.cause {
            crate::facade::interaction::UiDismissInteractionCause::Escape => {
                crate::facade::interaction::UiDismissInteraction::escape(
                    presentation,
                    self.sequence,
                    self.time_basis,
                )
            }
            crate::facade::interaction::UiDismissInteractionCause::OutsidePress(position) => {
                crate::facade::interaction::UiDismissInteraction::outside_press(
                    presentation,
                    self.sequence,
                    self.time_basis,
                    position,
                )
            }
        }
    }
}

pub enum WorthUiNativeManagedPortalDismissalOutcome {
    Ignored,
    Retained,
    Published(super::super::portal_dismissal::UiPortalDismissalPublicationReceipt),
    Pending,
    Stopped(WorthUiNativePortalDismissalStop),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiNativePortalDismissalStop {
    /// Input names a superseded presentation. No proposal or host work began.
    StalePresentation,
    Busy,
    IdentityExhausted,
    Transition,
    Proposal,
    Preparation,
    HostRejectedBeforeEffects,
    MountedRetention,
    MountedPresentation,
    Superseded,
    Indeterminate,
}

enum NormalizedPortalDismissal {
    Ignored,
    Published(super::super::portal_dismissal::UiPortalDismissalPublicationReceipt),
    Pending(super::super::portal_dismissal::DetachedUiPortalDismissalInFlight),
    Indeterminate(super::super::portal_dismissal::DetachedUiPortalDismissalIndeterminate),
    Stopped(WorthUiNativePortalDismissalStop),
}

impl super::super::WorthUiNativeApplicationShell {
    pub(in crate::facade::entry) fn begin_managed_anchor_loss_dismissal(
        &mut self,
        portal: crate::runtime::portal::UiPortalIdentity,
        now_tick: u64,
    ) -> WorthUiNativeManagedPortalDismissalOutcome {
        if self.pending_managed_rebind.is_some() {
            return WorthUiNativeManagedPortalDismissalOutcome::Stopped(
                WorthUiNativePortalDismissalStop::Busy,
            );
        }
        match normalize(
            self.session
                .publish_anchor_loss_portal_dismissal(portal, now_tick),
        ) {
            NormalizedPortalDismissal::Ignored => {
                WorthUiNativeManagedPortalDismissalOutcome::Ignored
            }
            NormalizedPortalDismissal::Published(receipt) => {
                WorthUiNativeManagedPortalDismissalOutcome::Published(receipt)
            }
            NormalizedPortalDismissal::Pending(pending) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::PortalDismissal(pending));
                WorthUiNativeManagedPortalDismissalOutcome::Pending
            }
            NormalizedPortalDismissal::Indeterminate(pending) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(pending));
                WorthUiNativeManagedPortalDismissalOutcome::Pending
            }
            NormalizedPortalDismissal::Stopped(stop) => {
                WorthUiNativeManagedPortalDismissalOutcome::Stopped(stop)
            }
        }
    }

    pub fn begin_managed_portal_dismissal(
        &mut self,
        interaction: crate::facade::interaction::UiDismissInteraction,
        now_tick: u64,
    ) -> WorthUiNativeManagedPortalDismissalOutcome {
        if matches!(
            &self.pending_managed_rebind,
            Some(
                WorthUiNativePendingManagedRebind::PortalDismissal(_)
                    | WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(_)
                    | WorthUiNativePendingManagedRebind::PortalDismissalReconstruction { .. }
                    | WorthUiNativePendingManagedRebind::PortalDismissalReconstructionDeferred { .. }
            )
        ) {
            self.retained_portal_dismissal =
                Some(UiRetainedPortalDismissalRequest::retain(interaction));
            return WorthUiNativeManagedPortalDismissalOutcome::Pending;
        }
        if self
            .pending_managed_rebind
            .as_ref()
            .is_some_and(WorthUiNativePendingManagedRebind::carries_portal_intent_consequence)
        {
            self.retained_portal_dismissal =
                Some(UiRetainedPortalDismissalRequest::retain(interaction));
            return WorthUiNativeManagedPortalDismissalOutcome::Retained;
        }
        if self.pending_managed_rebind.is_some() {
            return WorthUiNativeManagedPortalDismissalOutcome::Stopped(
                WorthUiNativePortalDismissalStop::Busy,
            );
        }
        self.retained_portal_dismissal = None;
        // Escape queued during a publication names the visible predecessor.
        // Reuse the Portal-owned continuation only across a mounted-admitted
        // direct successor. Outside presses keep their original geometry basis.
        let interaction = if matches!(
            interaction.cause(),
            crate::facade::interaction::UiDismissInteractionCause::Escape
        ) {
            self.session
                .portal
                .as_ref()
                .and_then(crate::runtime::portal::UiPortalRuntimeState::topmost_presentation)
                .and_then(|current| {
                    self.session
                        .mounted
                        .direct_successor_presentation(interaction.presentation(), current)
                })
                .map(|current| {
                    UiRetainedPortalDismissalRequest::retain(interaction).rebase(current)
                })
                .unwrap_or(interaction)
        } else {
            interaction
        };
        let outcome = self.session.publish_portal_dismissal(interaction, now_tick);
        match normalize(outcome) {
            NormalizedPortalDismissal::Ignored => {
                WorthUiNativeManagedPortalDismissalOutcome::Ignored
            }
            NormalizedPortalDismissal::Published(receipt) => {
                WorthUiNativeManagedPortalDismissalOutcome::Published(receipt)
            }
            NormalizedPortalDismissal::Pending(pending) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::PortalDismissal(pending));
                WorthUiNativeManagedPortalDismissalOutcome::Pending
            }
            NormalizedPortalDismissal::Indeterminate(pending) => {
                self.pending_managed_rebind =
                    Some(WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(pending));
                WorthUiNativeManagedPortalDismissalOutcome::Pending
            }
            NormalizedPortalDismissal::Stopped(stop) => {
                WorthUiNativeManagedPortalDismissalOutcome::Stopped(stop)
            }
        }
    }

    pub fn continue_retained_portal_dismissal_after_managed_intent(
        &mut self,
        now_tick: u64,
    ) -> WorthUiNativeManagedPortalDismissalOutcome {
        if self.pending_managed_rebind.is_some() {
            return WorthUiNativeManagedPortalDismissalOutcome::Stopped(
                WorthUiNativePortalDismissalStop::Busy,
            );
        }
        let Some(retained) = self.retained_portal_dismissal.take() else {
            return WorthUiNativeManagedPortalDismissalOutcome::Ignored;
        };
        let Some(presentation) = self
            .session
            .portal
            .as_ref()
            .and_then(crate::runtime::portal::UiPortalRuntimeState::topmost_presentation)
        else {
            return WorthUiNativeManagedPortalDismissalOutcome::Ignored;
        };
        self.begin_managed_portal_dismissal(retained.rebase(presentation), now_tick)
    }
}

pub(super) fn finish(
    pending_slot: &mut Option<WorthUiNativePendingManagedRebind>,
    outcome: super::super::portal_dismissal::UiPortalDismissalPublicationOutcome<'_>,
) -> WorthUiNativeManagedRebindProgress {
    match normalize(outcome) {
        NormalizedPortalDismissal::Published(receipt) => {
            WorthUiNativeManagedRebindProgress::PortalDismissed(receipt)
        }
        NormalizedPortalDismissal::Pending(pending) => {
            *pending_slot = Some(WorthUiNativePendingManagedRebind::PortalDismissal(pending));
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        }
        NormalizedPortalDismissal::Indeterminate(pending) => {
            *pending_slot =
                Some(WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(pending));
            WorthUiNativeManagedRebindProgress::AwaitingProgress
        }
        NormalizedPortalDismissal::Stopped(stop) => WorthUiNativeManagedRebindProgress::Stopped(
            super::WorthUiNativeManagedRebindStop::PortalDismissal(stop),
        ),
        NormalizedPortalDismissal::Ignored => {
            unreachable!("an admitted in-flight dismissal cannot become ignored")
        }
    }
}

fn normalize(
    outcome: super::super::portal_dismissal::UiPortalDismissalPublicationOutcome<'_>,
) -> NormalizedPortalDismissal {
    use super::super::portal_dismissal::UiPortalDismissalPublicationOutcome as Outcome;
    match outcome {
        Outcome::IgnoredNoMatchingPortal | Outcome::IgnoredInsideTopmostPortal => {
            NormalizedPortalDismissal::Ignored
        }
        Outcome::Published(receipt) => NormalizedPortalDismissal::Published(receipt),
        Outcome::InFlight(completion) => {
            NormalizedPortalDismissal::Pending(completion.detach_for_native())
        }
        Outcome::Indeterminate(recovery) => {
            NormalizedPortalDismissal::Indeterminate(recovery.detach_for_native())
        }
        Outcome::Stopped(stop) => NormalizedPortalDismissal::Stopped(map_stop(stop)),
    }
}

fn map_stop(
    stop: super::super::portal_dismissal::UiPortalDismissalPublicationStop,
) -> WorthUiNativePortalDismissalStop {
    use super::super::portal_dismissal::UiPortalDismissalPublicationStop as Stop;
    match stop {
        Stop::IdentityExhausted => WorthUiNativePortalDismissalStop::IdentityExhausted,
        Stop::StalePresentation => WorthUiNativePortalDismissalStop::StalePresentation,
        Stop::Transition => WorthUiNativePortalDismissalStop::Transition,
        Stop::Proposal => WorthUiNativePortalDismissalStop::Proposal,
        Stop::Preparation => WorthUiNativePortalDismissalStop::Preparation,
        Stop::HostRejectedBeforeEffects => {
            WorthUiNativePortalDismissalStop::HostRejectedBeforeEffects
        }
        Stop::MountedRetention => WorthUiNativePortalDismissalStop::MountedRetention,
        Stop::MountedPresentation => WorthUiNativePortalDismissalStop::MountedPresentation,
        Stop::Superseded => WorthUiNativePortalDismissalStop::Superseded,
    }
}

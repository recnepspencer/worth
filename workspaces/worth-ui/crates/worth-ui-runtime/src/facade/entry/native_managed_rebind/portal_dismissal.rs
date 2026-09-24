use super::{WorthUiNativeManagedRebindProgress, WorthUiNativePendingManagedRebind};

#[path = "portal_dismissal_recovery.rs"]
mod recovery;

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
    /// The admitted Portal target or application lifetime changed before publication.
    InteractionCancelled,
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
        interaction: super::super::WorthUiAdmittedPortalDismissal,
        now_tick: u64,
    ) -> WorthUiNativeManagedPortalDismissalOutcome {
        // The bounded slot never overwrites a previously admitted operation.
        if self.retained_portal_dismissal.is_some() {
            return WorthUiNativeManagedPortalDismissalOutcome::Stopped(
                WorthUiNativePortalDismissalStop::InteractionCancelled,
            );
        }
        if matches!(
            &self.pending_managed_rebind,
            Some(
                WorthUiNativePendingManagedRebind::PortalDismissal(_)
                    | WorthUiNativePendingManagedRebind::PortalDismissalIndeterminate(_)
                    | WorthUiNativePendingManagedRebind::PortalDismissalReconstruction { .. }
                    | WorthUiNativePendingManagedRebind::PortalDismissalReconstructionDeferred { .. }
            )
        ) {
            self.retained_portal_dismissal = Some(interaction);
            return WorthUiNativeManagedPortalDismissalOutcome::Pending;
        }
        if self
            .pending_managed_rebind
            .as_ref()
            .is_some_and(WorthUiNativePendingManagedRebind::carries_portal_intent_consequence)
        {
            self.retained_portal_dismissal = Some(interaction);
            return WorthUiNativeManagedPortalDismissalOutcome::Retained;
        }
        if self.pending_managed_rebind.is_some() {
            return WorthUiNativeManagedPortalDismissalOutcome::Stopped(
                WorthUiNativePortalDismissalStop::Busy,
            );
        }
        self.retained_portal_dismissal = None;
        let outcome = self
            .session
            .publish_admitted_portal_dismissal(interaction, now_tick);
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
        self.begin_managed_portal_dismissal(retained, now_tick)
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
        Stop::InteractionCancelled => WorthUiNativePortalDismissalStop::InteractionCancelled,
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

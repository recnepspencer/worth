#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPortalRuntimeCertificationSnapshot {
    active_portals: usize,
    open_portals: usize,
    visible_portals: usize,
    closing_portals: usize,
    indeterminate_portals: usize,
    portal_exit_retentions: usize,
    pending_track_coordinated: bool,
    committed_requests: u64,
    committed_idempotent_requests: u64,
    revision: u64,
}

pub trait WorthUiPortalRuntimeCertificationExt {
    fn inspect_portal_runtime_for_certification(&self) -> UiPortalRuntimeCertificationSnapshot;
    fn publish_nested_portal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalNestedCertificationOutcome;
    fn publish_root_portal_dismissal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalDismissalCertificationOutcome;
    fn publish_escape_portal_dismissal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalDismissalCertificationOutcome;
    fn publish_outside_portal_dismissal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalDismissalCertificationOutcome;
    fn progress_portal_exit_terminal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalExitTerminalCertificationOutcome;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPortalNestedCertificationOutcome {
    Published,
    NotPublished,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPortalExitTerminalCertificationOutcome {
    Idle,
    Published,
    Retry,
    AwaitingPhysical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPortalDismissalCertificationOutcome {
    Ignored,
    Published,
    InFlight,
    Indeterminate,
    Stopped(UiPortalDismissalCertificationStop),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiPortalDismissalCertificationStop {
    IdentityExhausted,
    Transition,
    Proposal,
    Preparation,
    HostRejectedBeforeEffects,
    MountedRetention,
    MountedPresentation,
    Superseded,
}

impl WorthUiPortalRuntimeCertificationExt for crate::facade::WorthUiActiveApplicationSession {
    fn inspect_portal_runtime_for_certification(&self) -> UiPortalRuntimeCertificationSnapshot {
        crate::facade::WorthUiActiveApplicationSession::inspect_portal_runtime_for_certification(
            self,
        )
    }

    fn publish_nested_portal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalNestedCertificationOutcome {
        crate::facade::entry::WorthUiActiveApplicationSession::publish_nested_portal_for_certification(
            self, now_tick,
        )
    }

    fn publish_root_portal_dismissal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalDismissalCertificationOutcome {
        map_dismissal_outcome(
            crate::facade::entry::WorthUiActiveApplicationSession::publish_root_portal_dismissal_for_certification(
                self, now_tick,
            ),
        )
    }

    fn publish_escape_portal_dismissal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalDismissalCertificationOutcome {
        let Some(presentation) = self.topmost_portal_presentation_for_certification() else {
            return UiPortalDismissalCertificationOutcome::Ignored;
        };
        let dismissal = crate::runtime::interaction::UiDismissInteraction::escape(
            presentation,
            worth_ui_host_contract::UiHostObservationSequence::new(now_tick),
            worth_ui_host_contract::UiHostObservationTimeBasis::PresentationRelativeTick(now_tick),
        );
        map_dismissal_outcome(self.publish_portal_dismissal(dismissal, now_tick))
    }

    fn publish_outside_portal_dismissal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalDismissalCertificationOutcome {
        let Some(presentation) = self.topmost_portal_presentation_for_certification() else {
            return UiPortalDismissalCertificationOutcome::Ignored;
        };
        let dismissal = crate::runtime::interaction::UiDismissInteraction::outside_press(
            presentation,
            worth_ui_host_contract::UiHostObservationSequence::new(now_tick),
            worth_ui_host_contract::UiHostObservationTimeBasis::PresentationRelativeTick(now_tick),
            worth_ui_host_contract::UiHostSurfacePosition::viewport_logical(0, 0),
        );
        map_dismissal_outcome(self.publish_portal_dismissal(dismissal, now_tick))
    }

    fn progress_portal_exit_terminal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> UiPortalExitTerminalCertificationOutcome {
        crate::facade::WorthUiActiveApplicationSession::progress_portal_exit_terminal_for_certification(
            self, now_tick,
        )
    }
}

fn map_dismissal_outcome(
    outcome: crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome<'_>,
) -> UiPortalDismissalCertificationOutcome {
    use crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationOutcome as Outcome;
    match outcome {
        Outcome::IgnoredNoMatchingPortal | Outcome::IgnoredInsideTopmostPortal => {
            UiPortalDismissalCertificationOutcome::Ignored
        }
        Outcome::Published(_) => UiPortalDismissalCertificationOutcome::Published,
        Outcome::InFlight(_) => UiPortalDismissalCertificationOutcome::InFlight,
        Outcome::Indeterminate(_) => UiPortalDismissalCertificationOutcome::Indeterminate,
        Outcome::Stopped(stop) => UiPortalDismissalCertificationOutcome::Stopped(map_stop(stop)),
    }
}

fn map_stop(
    stop: crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationStop,
) -> UiPortalDismissalCertificationStop {
    use crate::facade::entry::portal_dismissal::UiPortalDismissalPublicationStop as Stop;
    match stop {
        Stop::IdentityExhausted => UiPortalDismissalCertificationStop::IdentityExhausted,
        Stop::Transition => UiPortalDismissalCertificationStop::Transition,
        Stop::Proposal => UiPortalDismissalCertificationStop::Proposal,
        Stop::Preparation => UiPortalDismissalCertificationStop::Preparation,
        Stop::HostRejectedBeforeEffects => {
            UiPortalDismissalCertificationStop::HostRejectedBeforeEffects
        }
        Stop::MountedRetention => UiPortalDismissalCertificationStop::MountedRetention,
        Stop::MountedPresentation => UiPortalDismissalCertificationStop::MountedPresentation,
        Stop::Superseded => UiPortalDismissalCertificationStop::Superseded,
    }
}

impl UiPortalRuntimeCertificationSnapshot {
    pub(crate) const fn uninstalled() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, true, 0, 0, 0)
    }

    pub(crate) const fn new(
        active_portals: usize,
        open_portals: usize,
        visible_portals: usize,
        closing_portals: usize,
        indeterminate_portals: usize,
        portal_exit_retentions: usize,
        pending_track_coordinated: bool,
        committed_requests: u64,
        committed_idempotent_requests: u64,
        revision: u64,
    ) -> Self {
        Self {
            active_portals,
            open_portals,
            visible_portals,
            closing_portals,
            indeterminate_portals,
            portal_exit_retentions,
            pending_track_coordinated,
            committed_requests,
            committed_idempotent_requests,
            revision,
        }
    }

    pub const fn active_portals(self) -> usize {
        self.active_portals
    }

    pub const fn open_portals(self) -> usize {
        self.open_portals
    }

    pub const fn visible_portals(self) -> usize {
        self.visible_portals
    }

    pub const fn closing_portals(self) -> usize {
        self.closing_portals
    }

    pub const fn indeterminate_portals(self) -> usize {
        self.indeterminate_portals
    }

    pub const fn portal_exit_retentions(self) -> usize {
        self.portal_exit_retentions
    }

    pub const fn pending_track_coordinated(self) -> bool {
        self.pending_track_coordinated
    }

    pub const fn committed_requests(self) -> u64 {
        self.committed_requests
    }

    pub const fn committed_idempotent_requests(self) -> u64 {
        self.committed_idempotent_requests
    }

    pub const fn revision(self) -> u64 {
        self.revision
    }
}

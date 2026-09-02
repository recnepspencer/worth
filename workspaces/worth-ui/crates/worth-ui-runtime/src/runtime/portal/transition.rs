#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalServiceTransitionDenial {
    RevisionExhausted,
    StackOrdinalExhausted,
    LiveRowCapacityExceeded { limit: u16 },
    PortalNotLive,
    PortalSurfaceMismatch,
    ReplacementNotTopmost,
    StackOrdinalConflict,
    StalePlan,
    Placement(super::UiPortalPlacementDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalExitTerminalDenial {
    RetentionMismatch,
    Transition(UiPortalServiceTransitionDenial),
}

#[must_use = "a prepared portal transition changes no truth until it is committed"]
pub(crate) struct UiPreparedPortalServiceTransition {
    request: super::UiPortalServiceRequest,
    expected_revision: u64,
    committed_revision: u64,
    staged_posture: super::UiPortalLifecyclePosture,
    disposition: super::UiPortalServiceDisposition,
    placement: Option<super::UiPreparedPortalPlacement>,
    stack_ordinal: Option<super::UiPortalStackOrdinal>,
    closed_descendants: Box<[super::UiPortalIdentity]>,
}

impl UiPreparedPortalServiceTransition {
    pub(super) const fn new(
        request: super::UiPortalServiceRequest,
        expected_revision: u64,
        committed_revision: u64,
        staged_posture: super::UiPortalLifecyclePosture,
        disposition: super::UiPortalServiceDisposition,
        placement: Option<super::UiPreparedPortalPlacement>,
        stack_ordinal: Option<super::UiPortalStackOrdinal>,
        closed_descendants: Box<[super::UiPortalIdentity]>,
    ) -> Self {
        Self {
            request,
            expected_revision,
            committed_revision,
            staged_posture,
            disposition,
            placement,
            stack_ordinal,
            closed_descendants,
        }
    }

    pub(crate) const fn request(&self) -> super::UiPortalServiceRequest {
        self.request
    }

    pub(crate) const fn portal(&self) -> super::UiPortalIdentity {
        self.request.portal()
    }

    pub(crate) const fn opens_portal(&self) -> bool {
        matches!(
            self.request.operation(),
            super::request::UiPortalServiceOperation::Open
        )
    }

    pub(crate) const fn closes_portal(&self) -> bool {
        matches!(
            self.request.operation(),
            super::request::UiPortalServiceOperation::Close(_)
        )
    }

    pub(crate) const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    pub(super) const fn committed_revision(&self) -> u64 {
        self.committed_revision
    }

    pub(crate) const fn staged_posture(&self) -> super::UiPortalLifecyclePosture {
        self.staged_posture
    }

    pub(super) const fn disposition(&self) -> super::UiPortalServiceDisposition {
        self.disposition
    }

    pub(crate) const fn placement(&self) -> Option<super::UiPreparedPortalPlacement> {
        self.placement
    }

    pub(crate) const fn stack_ordinal(&self) -> Option<super::UiPortalStackOrdinal> {
        self.stack_ordinal
    }

    pub(crate) const fn successor_revision(&self) -> u64 {
        self.committed_revision
    }

    pub(crate) const fn is_idempotent(&self) -> bool {
        matches!(
            self.disposition,
            super::UiPortalServiceDisposition::Idempotent
        )
    }

    pub(super) fn closes(&self, portal: super::UiPortalIdentity) -> bool {
        self.portal() == portal || self.closed_descendants.contains(&portal)
    }

    pub(crate) fn closed_descendants(&self) -> &[super::UiPortalIdentity] {
        &self.closed_descendants
    }
}

use std::sync::Arc;

/// Move-only continuation from one completed product effect into its declared
/// consequence batch. It carries no provider or effect-invocation authority.
#[must_use]
pub struct UiIntentConsequenceHandle {
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    lease: Arc<UiIntentConsequenceLease>,
}

pub(crate) struct UiIntentConsequenceLease;

/// Move-only authority to retry only the consequence handoff of an already
/// completed effect.
#[must_use]
pub struct UiIntentConsequenceRecovery {
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
    lease: Arc<UiIntentConsequenceLease>,
}

#[derive(Debug)]
pub enum UiIntentConsequenceStopReason {
    StaleOrForeign,
    ApplicationGenerationChanged,
    TargetChanged(crate::runtime::interaction::UiInteractionTargetingDenial),
    ProductRouteChanged,
    MultipleQueryConsequences,
    UndeclaredQueryConsequence {
        observed: worth_ui_query_binding::WorthUiQueryViewIdentity,
    },
    MissingDeclaredQueryConsequence {
        expected: worth_ui_query_binding::WorthUiQueryViewIdentity,
    },
    QueryConsequenceIdentityMismatch {
        expected: worth_ui_query_binding::WorthUiQueryViewIdentity,
        observed: worth_ui_query_binding::WorthUiQueryViewIdentity,
    },
    ConsequenceFactCapacityExceeded {
        limit: usize,
        observed: usize,
    },
    ObservationTurn(crate::runtime::observation::UiObservationTurnDenial),
    ObservationAdmission(crate::runtime::observation::UiObservationAdmissionDenial),
    QueryHandoff(worth_ui_query_binding::WorthUiCollectionChangeHandoffRetryDenial),
    QueryAdmission(worth_ui_query_binding::WorthUiCollectionChangeAdmissionDenial),
    RebindAdmission(crate::runtime::rebind::UiRebindReservationDenial),
    MountedRetention(crate::mounting::UiMountedFrameRetentionDenial),
    MountedPresentation(crate::mounting::UiMountedPresentationAdmissionDenial),
    HostRejectedBeforeEffects {
        rejection_count: usize,
    },
    AffectedScope(Box<crate::runtime::rebind::UiAffectedScopeDenial>),
    IdentityLifecycle(Box<crate::runtime::rebind::UiIdentityLifecycleDenial>),
    Planning(Box<crate::runtime::rebind::UiRebindPlanningDenial>),
    Preparation(Box<crate::runtime::rebind::UiRebindPreparationDenial>),
    IntentPostureIdentityExhausted,
    RuntimeServiceRequiresMountedPosture,
    RuntimeServiceCommandRouteMissing,
    RuntimeServiceOwnerUnavailable(UiRuntimeServiceFamilyStopReason),
    RuntimeServiceTransitionExhausted,
    RuntimeServicePortalPlacement(UiIntentPortalPlacementStopReason),
    RuntimeServicePortalBinding(UiIntentPortalBindingStopReason),
    RuntimeServiceProposal(UiRuntimeServiceProposalStop),
}

/// Public, stable service-family identity for consequence diagnostics. Runtime
/// registry identity remains private to the capability owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRuntimeServiceFamilyStopReason {
    Portal,
    Focus,
    Motion,
    CommandRouting,
    Scroll,
    Selection,
}

/// Public classification of a failed runtime-service proposal. Internal proof
/// and authority types do not leak through the product-facing stop surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRuntimeServiceProposalStopReason {
    RequestBasis,
    Demand,
    Preflight,
    Reservation,
    Staging,
    Publication,
    Focus,
    Scroll,
    MissingScrollOwner,
    SelectionMapping,
    Selection,
    MotionRequest,
    Motion,
    MountedFrameMismatch,
    RevealRefinementMismatch,
    Coalesced,
}

/// Product-facing proposal stop with a stable category and retained internal
/// diagnostic detail. The detail is observational only and carries no runtime
/// authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiRuntimeServiceProposalStop {
    reason: UiRuntimeServiceProposalStopReason,
    detail: Box<str>,
}

impl UiRuntimeServiceProposalStop {
    pub const fn reason(&self) -> UiRuntimeServiceProposalStopReason {
        self.reason
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl From<crate::capability::UiRuntimeServiceFamily> for UiRuntimeServiceFamilyStopReason {
    fn from(family: crate::capability::UiRuntimeServiceFamily) -> Self {
        match family {
            crate::capability::UiRuntimeServiceFamily::Portal => Self::Portal,
            crate::capability::UiRuntimeServiceFamily::Focus => Self::Focus,
            crate::capability::UiRuntimeServiceFamily::Motion => Self::Motion,
            crate::capability::UiRuntimeServiceFamily::CommandRouting => Self::CommandRouting,
            crate::capability::UiRuntimeServiceFamily::Scroll => Self::Scroll,
            crate::capability::UiRuntimeServiceFamily::Selection => Self::Selection,
        }
    }
}

impl From<crate::runtime::session::UiPortalProposalPreparationDenial>
    for UiRuntimeServiceProposalStop
{
    fn from(denial: crate::runtime::session::UiPortalProposalPreparationDenial) -> Self {
        use crate::runtime::session::UiPortalProposalPreparationDenial as Denial;
        let (reason, detail) = match denial {
            Denial::RequestBasis(detail) => (
                UiRuntimeServiceProposalStopReason::RequestBasis,
                format!("{detail:?}"),
            ),
            Denial::Demand(detail) => (
                UiRuntimeServiceProposalStopReason::Demand,
                format!("{detail:?}"),
            ),
            Denial::Preflight(detail) => (
                UiRuntimeServiceProposalStopReason::Preflight,
                format!("{detail:?}"),
            ),
            Denial::Reservation(detail) => (
                UiRuntimeServiceProposalStopReason::Reservation,
                format!("{detail:?}"),
            ),
            Denial::Staging(detail) => (
                UiRuntimeServiceProposalStopReason::Staging,
                format!("{detail:?}"),
            ),
            Denial::Publication(detail) => (
                UiRuntimeServiceProposalStopReason::Publication,
                format!("{detail:?}"),
            ),
            Denial::Focus(detail) => (
                UiRuntimeServiceProposalStopReason::Focus,
                format!("{detail:?}"),
            ),
            Denial::Scroll(detail) => (
                UiRuntimeServiceProposalStopReason::Scroll,
                format!("{detail:?}"),
            ),
            Denial::MissingScrollOwner => (
                UiRuntimeServiceProposalStopReason::MissingScrollOwner,
                "missing Scroll owner".into(),
            ),
            Denial::SelectionMapping(detail) => (
                UiRuntimeServiceProposalStopReason::SelectionMapping,
                format!("{detail:?}"),
            ),
            Denial::Selection(detail) => (
                UiRuntimeServiceProposalStopReason::Selection,
                format!("{detail:?}"),
            ),
            Denial::MotionRequest(detail) => (
                UiRuntimeServiceProposalStopReason::MotionRequest,
                format!("{detail:?}"),
            ),
            Denial::Motion(detail) => (
                UiRuntimeServiceProposalStopReason::Motion,
                format!("{detail:?}"),
            ),
            Denial::MountedFrameMismatch => (
                UiRuntimeServiceProposalStopReason::MountedFrameMismatch,
                "mounted frame mismatch".into(),
            ),
            Denial::RevealRefinementMismatch => (
                UiRuntimeServiceProposalStopReason::RevealRefinementMismatch,
                "reveal refinement mismatch".into(),
            ),
            Denial::Coalesced(detail) => (
                UiRuntimeServiceProposalStopReason::Coalesced,
                format!("coalesced with {detail:?}"),
            ),
        };
        Self {
            reason,
            detail: detail.into_boxed_str(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentPortalPlacementStopReason {
    MissingPresentedAnchor,
    MissingPresentedViewport,
    IncompatibleCoordinateSpace,
    EmptyAnchor,
    InsufficientViewport,
    UnknownPortalParent,
    PortalLayerDepthExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiIntentPortalBindingStopReason {
    ForeignGeneration,
    UnknownPortalDeclaration,
    ForeignSurfaceDeclaration,
    PortalSurfaceUndeclared,
    DeclaredSurfaceUnbound,
    DeclaredSurfaceAlreadyBound,
    RuntimeSurfaceConflict,
    PortalSurfaceMismatch,
    PortalDeclarationConflict,
    PortalAlreadyLiveWithoutBinding,
    RetiredBinding,
    TransitionMismatch,
    SurfaceBindingCapacityExceeded,
    OwnerConflict,
    MountedIdentity,
}

#[must_use]
pub struct UiIntentConsequenceStop {
    reason: UiIntentConsequenceStopReason,
    recovery: UiIntentConsequenceRecovery,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiIntentConsequenceCompletionReceipt {
    attempt: super::UiIntentExecutionAttemptIdentity,
    idempotency: super::UiIntentExecutionIdempotencyIdentity,
}

impl UiIntentConsequenceHandle {
    pub(crate) fn new(
        attempt: super::UiIntentExecutionAttemptIdentity,
        idempotency: super::UiIntentExecutionIdempotencyIdentity,
    ) -> (Self, Arc<UiIntentConsequenceLease>) {
        let lease = Arc::new(UiIntentConsequenceLease);
        (
            Self {
                attempt,
                idempotency,
                lease: Arc::clone(&lease),
            },
            lease,
        )
    }

    pub const fn attempt(&self) -> super::UiIntentExecutionAttemptIdentity {
        self.attempt
    }

    pub const fn idempotency(&self) -> super::UiIntentExecutionIdempotencyIdentity {
        self.idempotency
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        super::UiIntentExecutionAttemptIdentity,
        super::UiIntentExecutionIdempotencyIdentity,
        Arc<UiIntentConsequenceLease>,
    ) {
        (self.attempt, self.idempotency, self.lease)
    }

    pub(crate) const fn lease(&self) -> &Arc<UiIntentConsequenceLease> {
        &self.lease
    }
}

impl UiIntentConsequenceRecovery {
    pub const fn attempt(&self) -> super::UiIntentExecutionAttemptIdentity {
        self.attempt
    }

    pub const fn idempotency(&self) -> super::UiIntentExecutionIdempotencyIdentity {
        self.idempotency
    }

    pub(crate) fn from_parts(
        attempt: super::UiIntentExecutionAttemptIdentity,
        idempotency: super::UiIntentExecutionIdempotencyIdentity,
        lease: Arc<UiIntentConsequenceLease>,
    ) -> Self {
        Self {
            attempt,
            idempotency,
            lease,
        }
    }

    pub(crate) fn into_handle(self) -> UiIntentConsequenceHandle {
        UiIntentConsequenceHandle {
            attempt: self.attempt,
            idempotency: self.idempotency,
            lease: self.lease,
        }
    }
}

impl UiIntentConsequenceStop {
    pub(crate) const fn new(
        reason: UiIntentConsequenceStopReason,
        recovery: UiIntentConsequenceRecovery,
    ) -> Self {
        Self { reason, recovery }
    }

    pub const fn reason(&self) -> &UiIntentConsequenceStopReason {
        &self.reason
    }

    pub fn into_recovery(self) -> UiIntentConsequenceRecovery {
        self.recovery
    }

    pub(crate) fn into_parts(self) -> (UiIntentConsequenceStopReason, UiIntentConsequenceRecovery) {
        (self.reason, self.recovery)
    }
}

impl UiIntentConsequenceCompletionReceipt {
    pub(crate) const fn new(
        attempt: super::UiIntentExecutionAttemptIdentity,
        idempotency: super::UiIntentExecutionIdempotencyIdentity,
    ) -> Self {
        Self {
            attempt,
            idempotency,
        }
    }

    pub const fn attempt(self) -> super::UiIntentExecutionAttemptIdentity {
        self.attempt
    }

    pub const fn idempotency(self) -> super::UiIntentExecutionIdempotencyIdentity {
        self.idempotency
    }
}

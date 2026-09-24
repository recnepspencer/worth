use worth_ui_host_contract::{
    UiHostObservationFamily, UiHostObservationPresentationBasis, UiSemanticSurfaceIdentity,
    UiSurfaceBindingGeneration,
};

use super::UiDraftSessionIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiLocalInputStopReason {
    NoLocalRecipient,
    MissingInputRecipientAffinity,
    InputRecipientAffinityChanged,
    TextProfileGenerationChanged {
        expected: worth_ui_host_contract::UiTextProfileGeneration,
        observed: Option<worth_ui_host_contract::UiTextProfileGeneration>,
    },
    ForeignBinding {
        expected: UiSurfaceBindingGeneration,
        observed: UiSurfaceBindingGeneration,
    },
    ApplicationGenerationChanged,
    TargetNoLongerCurrent(crate::runtime::interaction::UiInteractionTargetingDenial),
    InputRevisionDiscontinuity {
        previous: u64,
        observed: u64,
    },
    DraftByteBudgetExceeded {
        limit: usize,
        attempted: usize,
    },
    RecipientFamilyMismatch {
        required: super::UiLocalInputRecipientFamily,
        active: super::UiLocalInputRecipientFamily,
    },
    CompositionActive,
    RecipientReplaced,
    ExplicitCancel,
    FocusLost,
    ObservationInvalid,
    ObservationLoss(UiHostObservationFamily),
    SurfaceRebound,
    MountedInstanceRemoved,
    ApplicationRebound,
    Shutdown,
}

#[derive(Debug, Eq, PartialEq)]
pub struct UiLocalInputStop {
    recipient: UiLocalInputStopRecipient,
    presentation: UiHostObservationPresentationBasis,
    reason: UiLocalInputStopReason,
}

/// Which recipient a stop reached and what it settled. Only a session stop
/// names a draft session, and only a report with no recipient names no
/// surface.
#[derive(Debug, Eq, PartialEq)]
enum UiLocalInputStopRecipient {
    /// The session settled its draft along with its recipient.
    SettledSession {
        session: UiDraftSessionIdentity,
        surface: UiSemanticSurfaceIdentity,
    },
    /// The recipient settled; its session stays suspended for a later owner.
    SuspendedSession {
        session: UiDraftSessionIdentity,
        surface: UiSemanticSurfaceIdentity,
    },
    /// Neither the session nor its recipient settled.
    UnsettledSession {
        session: UiDraftSessionIdentity,
        surface: UiSemanticSurfaceIdentity,
    },
    /// A sessionless recipient settled.
    SettledRecipient { surface: UiSemanticSurfaceIdentity },
    /// A stop reported with no recipient to settle.
    Unreported,
}

impl UiLocalInputStop {
    pub(super) const fn for_settled_session(
        session: UiDraftSessionIdentity,
        surface: UiSemanticSurfaceIdentity,
        presentation: UiHostObservationPresentationBasis,
        reason: UiLocalInputStopReason,
    ) -> Self {
        Self {
            recipient: UiLocalInputStopRecipient::SettledSession { session, surface },
            presentation,
            reason,
        }
    }

    pub(super) const fn for_settled_recipient(
        surface: UiSemanticSurfaceIdentity,
        presentation: UiHostObservationPresentationBasis,
        reason: UiLocalInputStopReason,
    ) -> Self {
        Self {
            recipient: UiLocalInputStopRecipient::SettledRecipient { surface },
            presentation,
            reason,
        }
    }

    pub(super) const fn for_suspended_session(
        session: UiDraftSessionIdentity,
        surface: UiSemanticSurfaceIdentity,
        presentation: UiHostObservationPresentationBasis,
        reason: UiLocalInputStopReason,
    ) -> Self {
        Self {
            recipient: UiLocalInputStopRecipient::SuspendedSession { session, surface },
            presentation,
            reason,
        }
    }

    pub(super) const fn for_unsettled_report(
        presentation: UiHostObservationPresentationBasis,
        reason: UiLocalInputStopReason,
    ) -> Self {
        Self {
            recipient: UiLocalInputStopRecipient::Unreported,
            presentation,
            reason,
        }
    }

    pub(super) const fn for_unsettled_session(
        session: UiDraftSessionIdentity,
        surface: UiSemanticSurfaceIdentity,
        presentation: UiHostObservationPresentationBasis,
        reason: UiLocalInputStopReason,
    ) -> Self {
        Self {
            recipient: UiLocalInputStopRecipient::UnsettledSession { session, surface },
            presentation,
            reason,
        }
    }

    pub const fn session(&self) -> Option<UiDraftSessionIdentity> {
        match self.recipient {
            UiLocalInputStopRecipient::SettledSession { session, .. }
            | UiLocalInputStopRecipient::SuspendedSession { session, .. }
            | UiLocalInputStopRecipient::UnsettledSession { session, .. } => Some(session),
            UiLocalInputStopRecipient::SettledRecipient { .. }
            | UiLocalInputStopRecipient::Unreported => None,
        }
    }

    pub const fn surface(&self) -> Option<UiSemanticSurfaceIdentity> {
        match self.recipient {
            UiLocalInputStopRecipient::SettledSession { surface, .. }
            | UiLocalInputStopRecipient::SuspendedSession { surface, .. }
            | UiLocalInputStopRecipient::UnsettledSession { surface, .. }
            | UiLocalInputStopRecipient::SettledRecipient { surface } => Some(surface),
            UiLocalInputStopRecipient::Unreported => None,
        }
    }

    pub const fn presentation(&self) -> UiHostObservationPresentationBasis {
        self.presentation
    }

    pub const fn settled_session(&self) -> bool {
        matches!(
            self.recipient,
            UiLocalInputStopRecipient::SettledSession { .. }
        )
    }

    pub const fn settled_recipient(&self) -> bool {
        matches!(
            self.recipient,
            UiLocalInputStopRecipient::SettledSession { .. }
                | UiLocalInputStopRecipient::SuspendedSession { .. }
                | UiLocalInputStopRecipient::SettledRecipient { .. }
        )
    }

    pub const fn reason(&self) -> UiLocalInputStopReason {
        self.reason
    }
}

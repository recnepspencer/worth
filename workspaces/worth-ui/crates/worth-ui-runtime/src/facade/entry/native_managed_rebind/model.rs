#[derive(Debug)]
pub enum WorthUiNativeManagedRebindDenial {
    SessionMismatch,
    Reconstruction(super::super::WorthUiNativePresentationRecoveryDenial),
    PredecessorReconstruction,
    Preparation(crate::runtime::rebind::UiRebindPreparationDenial),
}

#[derive(Debug)]
pub enum WorthUiNativeManagedRebindStop {
    Duplicate,
    ObservedNoChange,
    RejectedBeforeEffects {
        phase: crate::runtime::rebind::UiRebindStoppedPhase,
        cause: crate::runtime::rebind::UiRebindDenialCause,
        host_denials: Box<[worth_ui_host_contract::UiHostSurfacePresentationDenial]>,
    },
    CancelledBeforeEffects(crate::runtime::rebind::UiRebindCancellationReceipt),
    TimedOutBeforeEffects(crate::runtime::rebind::UiRebindTimeoutReceipt),
    SupersededBeforeEffects(crate::runtime::rebind::UiRebindSupersededReceipt),
    Indeterminate,
    PredecessorReconstructionFailed,
    IntentPosture(super::super::native_intent_posture::WorthUiNativeIntentPosturePublicationStop),
    IntentConsequence(crate::runtime::intent_execution::UiIntentConsequenceStopReason),
    PortalDismissal(super::super::WorthUiNativePortalDismissalStop),
    InternalDefect(crate::runtime::rebind::UiRebindInternalDefectKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiNativePredecessorRecovery {
    IntentConsequence,
    PortalDismissal,
}

pub enum WorthUiNativeManagedRebindProgress {
    Unrelated,
    AwaitingProgress,
    RecoveryBlocked(
        super::super::native_application_shell::WorthUiNativePresentationRecoveryDenial,
    ),
    RecoveredToPredecessor(WorthUiNativePredecessorRecovery),
    Published(crate::runtime::rebind::UiRebindReceipt),
    RebindRecovered(crate::runtime::rebind::UiRebindRecoveryReceipt),
    IntentConsequencePublished(
        crate::facade::entry::intent_consequence_publication::UiIntentConsequencePublicationReceipt,
    ),
    PortalDismissed(super::super::portal_dismissal::UiPortalDismissalPublicationReceipt),
    Stopped(WorthUiNativeManagedRebindStop),
}

pub(in crate::facade::entry) enum WorthUiNativePendingManagedRebind {
    Indeterminate {
        recovery: crate::runtime::rebind::UiDetachedRebindRecovery,
        frame: crate::mounting::UiMountedIndeterminateFrame,
    },
    RecoveryReconstruction {
        recovery: crate::runtime::rebind::UiDetachedRebindRecovery,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    },
    RecoveryReconstructionDeferred(crate::runtime::rebind::UiDetachedRebindRecovery),
    Completion(crate::runtime::rebind::UiDetachedRebindCompletion),
    Retry {
        retry: crate::runtime::rebind::UiDetachedRebindRetry,
        requires_reconstruction: bool,
    },
    IntentPosture(super::super::native_intent_posture::DetachedNativeIntentPosturePending),
    IntentPosturePredecessorReconstruction {
        retry: super::super::native_intent_posture::DetachedNativeIntentPosturePending,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    },
    IntentPosturePredecessorReconstructionDeferred(
        super::super::native_intent_posture::DetachedNativeIntentPosturePending,
    ),
    IntentPosturePredecessorIndeterminate {
        retry: super::super::native_intent_posture::DetachedNativeIntentPosturePending,
        frame: crate::mounting::UiMountedIndeterminateFrame,
    },
    IntentConsequence(
        super::super::intent_consequence_publication::DetachedUiIntentConsequenceInFlight,
    ),
    IntentConsequenceIndeterminate(
        super::super::intent_consequence_publication::DetachedUiIntentConsequenceIndeterminate,
    ),
    IntentConsequenceReconstruction {
        portal: Option<crate::runtime::session::UiIndeterminatePortalProposalTransaction>,
        resources: super::super::intent_consequence_publication::DetachedUiIntentConsequenceRecoveryResources,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    },
    IntentConsequenceReconstructionDeferred {
        portal: Option<crate::runtime::session::UiIndeterminatePortalProposalTransaction>,
        resources: super::super::intent_consequence_publication::DetachedUiIntentConsequenceRecoveryResources,
    },
    PortalDismissal(super::super::portal_dismissal::DetachedUiPortalDismissalInFlight),
    PortalDismissalIndeterminate(
        super::super::portal_dismissal::DetachedUiPortalDismissalIndeterminate,
    ),
    PortalDismissalReconstruction {
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    },
    PortalDismissalReconstructionDeferred {
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
    },
    PredecessorReconstruction {
        retry: crate::runtime::rebind::UiDetachedRebindRetry,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    },
}

impl WorthUiNativePendingManagedRebind {
    pub(in crate::facade::entry) fn carries_portal_intent_consequence(&self) -> bool {
        match self {
            Self::IntentConsequence(pending) => pending.carries_portal_transition(),
            Self::IntentConsequenceIndeterminate(pending) => pending.carries_portal_transition(),
            Self::IntentConsequenceReconstruction { portal, .. }
            | Self::IntentConsequenceReconstructionDeferred { portal, .. } => portal.is_some(),
            _ => false,
        }
    }
}

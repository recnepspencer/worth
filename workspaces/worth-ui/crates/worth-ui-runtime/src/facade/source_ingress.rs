//! Filesystem/source ingress and inseparable candidate-composition outcomes.

pub use crate::runtime::{
    UiSourceCompilationDenialReceipt, UiSourceRebindAttemptBasis, UiSourceRebindAttemptDenial,
    UiSourceRebindAttemptDenialReceipt, UiSourceRebindAttemptFailure, UiSourceRebindAttemptOutcome,
    WorthUiAuthoredBackdropDeclaration, WorthUiAuthoredOverlayMaterial,
    WorthUiAuthoredPortalAnchorBinding, WorthUiAuthoredProjectionRequirement,
    WorthUiAuthoredServiceDeclaration, WorthUiCandidateComposition,
    WorthUiCandidateCompositionBasis, WorthUiCandidateOrderingReceipt,
    WorthUiFilesystemSourceAcquisitionDenial, WorthUiFilesystemSourceProvider,
    WorthUiFilesystemSourceWatcher, WorthUiFilesystemWatcherBackend,
    WorthUiFilesystemWatcherDenial, WorthUiFilesystemWatcherReadiness,
    WorthUiFilesystemWatcherShutdownReceipt, WorthUiProjectionContentEdge, WorthUiReloadDebounce,
    WorthUiSemanticHandoffEvidence, WorthUiSemanticHandoffPreparationDenial,
    WorthUiSemanticHandoffPreparationStop, WorthUiServiceDeclarationAdmissionCause,
    WorthUiSettledSourceSnapshot, WorthUiSourceEventIngress, WorthUiSourceEventIngressSession,
    WorthUiSourceIngressCounters, WorthUiSourceIngressDenial, WorthUiSourceIngressDenialReason,
    WorthUiSourcePackageRevision, WorthUiSourceProvider, WorthUiSourceProviderKind,
    WorthUiWatchedCandidateSubmission, WorthUiWatchedCandidateSubmissionDenial,
    WorthUiWatcherEvent,
};

/// Application-author access to runtime-owned source event ingress.
///
/// Importing this trait from the source audience is required to open the
/// ingress capability on an active application session.
pub trait WorthUiSourceIngressExt {
    fn source_event_ingress(&self, provider: WorthUiSourceProvider) -> WorthUiSourceEventIngress;
}

impl WorthUiSourceIngressExt for crate::facade::WorthUiActiveApplicationSession {
    fn source_event_ingress(&self, provider: WorthUiSourceProvider) -> WorthUiSourceEventIngress {
        crate::facade::WorthUiActiveApplicationSession::source_event_ingress(self, provider)
    }
}

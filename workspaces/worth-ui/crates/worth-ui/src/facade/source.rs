//! Runtime-owned source transport, settlement, and candidate-ingress surfaces.
//!
//! Authored inputs, spans, and compiler diagnostics are owned by `worth-ui-dsl`
//! and are intentionally not forwarded through this runtime audience.

pub use worth_ui_runtime::facade::source_ingress::{
    UiSourceCompilationDenialReceipt, UiSourceRebindAttemptBasis, UiSourceRebindAttemptDenial,
    UiSourceRebindAttemptDenialReceipt, UiSourceRebindAttemptFailure, UiSourceRebindAttemptOutcome,
    WorthUiCandidateComposition, WorthUiCandidateCompositionBasis, WorthUiCandidateOrderingReceipt,
    WorthUiFilesystemSourceAcquisitionDenial, WorthUiFilesystemSourceProvider,
    WorthUiFilesystemSourceWatcher, WorthUiFilesystemWatcherBackend,
    WorthUiFilesystemWatcherDenial, WorthUiFilesystemWatcherReadiness,
    WorthUiFilesystemWatcherShutdownReceipt, WorthUiReloadDebounce, WorthUiSemanticHandoffEvidence,
    WorthUiSemanticHandoffPreparationCause, WorthUiSemanticHandoffPreparationDenial,
    WorthUiSemanticHandoffPreparationStop, WorthUiSettledSourceSnapshot, WorthUiSourceEventIngress,
    WorthUiSourceEventIngressSession, WorthUiSourceIngressCounters, WorthUiSourceIngressDenial,
    WorthUiSourceIngressDenialReason, WorthUiSourceIngressExt, WorthUiSourcePackageRevision,
    WorthUiSourceProvider, WorthUiSourceProviderKind, WorthUiWatchedCandidateSubmission,
    WorthUiWatchedCandidateSubmissionDenial, WorthUiWatcherEvent,
};

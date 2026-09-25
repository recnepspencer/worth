mod outcome;
mod pending;
mod presentation;

pub struct WorthUiPendingMountedPreview<'session> {
    generation:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    visual_trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    graph: crate::graph::UiGraphAuthority<'session>,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    plan_digest: u64,
    transition: crate::runtime::UiPendingMountedPreviewTransition<'session>,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
    consumed_facts: &'session crate::graph::UiGraphConsumedFactIndex,
    capabilities: &'session crate::capability::CapabilitySnapshot,
    intent_catalog: &'session crate::declaration::UiIntentCatalog,
    presentation: &'session crate::runtime::presentation_state::UiApplicationPresentationState,
    appearance_owner_snapshot:
        &'session Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
    preview_theme_observation: crate::mounting::UiMountedPreviewThemeObservation,
    ports: WorthUiMountedPreviewPorts<'session>,
}

pub struct WorthUiPreparedMountedPreview<'session> {
    frame: crate::mounting::UiPreparedMountedFrame,
    transition: crate::runtime::UiPendingMountedPreviewTransition<'session>,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
    ports: WorthUiMountedPreviewPorts<'session>,
}

pub struct WorthUiMountedPreviewPreparationRejection<'session> {
    denial: WorthUiMountedPreviewPreparationDenial,
    pending: Box<WorthUiPendingMountedPreview<'session>>,
}

pub struct WorthUiMountedPreviewAdmissionRejection<'session> {
    denial: crate::mounting::UiMountedPresentationAdmissionDenial,
    preview: WorthUiPreparedMountedPreview<'session>,
}

pub struct WorthUiMountedPreviewRetentionRejection<'session> {
    denial: crate::mounting::UiMountedFrameRetentionDenial,
    preview: WorthUiPreparedMountedPreview<'session>,
}

pub struct WorthUiMountedPreviewInFlight<'session> {
    handle: crate::mounting::UiMountedPresentationInFlight,
    before: crate::runtime::UiAllocationTruthRevision,
    transition: crate::runtime::UiPendingMountedPreviewTransition<'session>,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
    ports: WorthUiMountedPreviewPorts<'session>,
}

pub struct WorthUiMountedPreviewCompletionRejection<'session> {
    denial: crate::mounting::UiMountedPresentationCompletionDenial,
    in_flight: WorthUiMountedPreviewInFlight<'session>,
}

struct WorthUiMountedPreviewPorts<'session> {
    motion: Option<&'session mut crate::runtime::motion::UiMotionRuntimeState>,
    scroll: Option<&'session mut crate::runtime::scroll::UiScrollRuntimeState>,
    application_session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    generation_identity:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    host_session: &'session crate::facade::WorthUiHostSessionAuthority,
    mounted: &'session mut crate::mounting::WorthUiMountedSessionState,
    focus: Option<&'session mut crate::runtime::focus::UiFocusRuntimeState>,
    portal: Option<&'session mut crate::runtime::portal::UiPortalRuntimeState>,
    interaction: &'session mut crate::runtime::interaction::UiInteractionRuntimeState,
    host_exchange: &'session mut crate::host_exchange::WorthUiHostExchangeSessionState,
    owed_scroll_settles: &'session super::active_application_session::UiOwedScrollSettles,
}

#[derive(Debug, PartialEq)]
pub enum WorthUiMountedPreviewPreparationDenial {
    UnpresentedScrollLayout,
    UnknownMountedInstance,
    PreviewTargetMismatch,
    MissingSurfaceBinding,
    Frame(crate::mounting::UiMountedFramePreparationDenial),
}

pub enum WorthUiMountedPreviewDisposition {
    Published(crate::mounting::UiMountedFramePublicationReceipt),
    RejectedBeforeEffects(crate::mounting::UiMountedRejectedFrame),
    PresentationIndeterminate(crate::mounting::UiMountedIndeterminateFrame),
    Superseded,
}

pub struct WorthUiResolvedMountedPreview {
    disposition: WorthUiMountedPreviewDisposition,
    isolation: crate::runtime::UiPreviewPaintIsolationOutcome,
    follow_on: crate::runtime::WorthUiMountedPreviewFollowOn,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
}

pub enum WorthUiMountedPreviewOutcome<'session> {
    Resolved(Box<WorthUiResolvedMountedPreview>),
    InFlight(Box<WorthUiMountedPreviewInFlight<'session>>),
    RetentionDenied(Box<WorthUiMountedPreviewRetentionRejection<'session>>),
    AdmissionDenied(Box<WorthUiMountedPreviewAdmissionRejection<'session>>),
    CompletionDenied(Box<WorthUiMountedPreviewCompletionRejection<'session>>),
}

use super::{WorthUiActiveApplicationSession, WorthUiActiveApplicationSessionIdentity, WorthUiApp};
use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;

#[cfg(test)]
#[path = "application_replacement_exact_authority_tests.rs"]
mod application_replacement_exact_authority_tests;

mod basis;
mod candidate;
mod candidate_pipeline;
mod candidate_scroll_geometry;
mod cutover;
mod cutover_generation;
#[cfg(test)]
mod cutover_test_access;
mod mounted;
mod mounted_frame;
mod portal_lifecycle;
mod prepared_activation_access;
mod publication_observation;
mod rebind_preparation;
mod receipt;
mod retry;
mod scroll_replacement;
mod selection_replacement;
mod service_installation_reconciliation;

pub use candidate::{WorthUiReplacementCandidateSummary, WorthUiReplacementPlannedCostEnvelope};
pub(crate) use mounted::{
    WorthUiDetachedMountedApplicationReplacementInFlight,
    WorthUiDetachedPreparedMountedApplicationReplacement,
};
pub use mounted::{
    WorthUiMountedApplicationReplacementInFlight,
    WorthUiMountedApplicationReplacementIndeterminate, WorthUiMountedApplicationReplacementOutcome,
    WorthUiMountedReplacementAdmissionDenial, WorthUiMountedReplacementCompletionDenial,
    WorthUiMountedReplacementPreparationOutcome, WorthUiMountedReplacementRetentionDenial,
    WorthUiPreparedMountedApplicationReplacement,
};
pub use publication_observation::WorthUiApplicationPublicationObservation;

pub struct WorthUiPreparedApplicationReplacement {
    next_app: WorthUiApp,
    semantic_input: WorthUiPreparedReplacementSemanticInput,
    basis: WorthUiPreparedApplicationReplacementBasis,
    candidate_query_binding: worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    candidate_graph_changed_nodes: std::collections::BTreeSet<crate::graph::UiGraphNodeIdentity>,
}

enum WorthUiPreparedReplacementSemanticInput {
    Admitted(crate::runtime::WorthUiAdmittedReplacementCandidate),
    Prelowered(crate::runtime::WorthUiReplacementLoweringReady),
}

impl WorthUiPreparedReplacementSemanticInput {
    fn admitted(&self) -> &crate::runtime::WorthUiAdmittedReplacementCandidate {
        match self {
            Self::Admitted(admitted) => admitted,
            Self::Prelowered(lowering) => lowering.admitted(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiPreparedApplicationReplacementBasis {
    origin_session: WorthUiActiveApplicationSessionIdentity,
    next_generation: WorthUiPreparedApplicationGenerationIdentity,
    candidate_basis: crate::runtime::WorthUiReplacementCandidateBasis,
    graph_authority_identity: crate::graph::UiGraphAuthorityIdentity,
    candidate_application_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationLoweringAuthority,
}

pub struct WorthUiCandidateInspectionReceipt {
    generation_identity: WorthUiPreparedApplicationGenerationIdentity,
    candidate_basis: crate::runtime::WorthUiReplacementCandidateBasis,
    receipt: crate::facade::inspection_bridge::UiInspectionReceipt,
}

pub struct WorthUiLoweredApplicationReplacement {
    next_app: WorthUiApp,
    lowering: crate::runtime::WorthUiReplacementLoweringReady,
    basis: WorthUiPreparedApplicationReplacementBasis,
    candidate_query_binding: worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    candidate_graph_changed_nodes: std::collections::BTreeSet<crate::graph::UiGraphNodeIdentity>,
    reload_cost_seed: crate::runtime::WorthUiReloadCostSeed,
    active_generation: WorthUiPreparedApplicationGenerationIdentity,
}

pub struct WorthUiPendingApplicationCutover {
    next_app: WorthUiApp,
    pending_activation: crate::runtime::WorthUiPendingActivation,
    basis: WorthUiPreparedApplicationReplacementBasis,
    candidate_query_binding: worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    candidate_graph_changed_nodes: std::collections::BTreeSet<crate::graph::UiGraphNodeIdentity>,
    reload_cost_seed: crate::runtime::WorthUiReloadCostSeed,
}

/// Candidate ownership returned when a transient frame boundary cannot admit
/// publication yet.
#[must_use = "a denied frame-boundary cutover remains retryable"]
pub struct WorthUiApplicationCutoverRetry {
    pending: WorthUiPendingApplicationCutover,
    admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
    lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
}

#[must_use = "application cutover receipts may carry Query resources requiring explicit retirement"]
pub struct WorthUiApplicationCutoverReceipt {
    transition: Box<WorthUiPreparedApplicationActivation>,
    observation_resources: crate::runtime::observation::UiObservationResourceRetirementReport,
    intent_evidence: worth_ui_inspection::UiIntentEvidenceRetirementReport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPortalExitRetentionPendingKind {
    InFlight,
    Indeterminate,
    Reconstruction,
}

enum WorthUiApplicationCutoverTransition {
    Prepared(crate::runtime::WorthUiPreparedApplicationPlanSwap),
    Committed {
        plan_swap: Box<crate::runtime::WorthUiPlanSwapReceipt>,
        plan_decision: crate::runtime::WorthUiExecutablePlanDecision,
        query_retirement: worth_ui_query_binding::WorthUiOperationLiveRetirement,
        allocation_catalog_successor: crate::runtime::UiAllocationCatalogSuccessorReceipt,
    },
}

pub(super) struct WorthUiPreparedApplicationActivation {
    identity: Box<WorthUiApplicationCutoverIdentityEvidence>,
    publication: Box<WorthUiApplicationPublicationObservation>,
    visual_trace_source:
        crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    candidate_graph: crate::graph::UiGraphSnapshot,
    candidate_application_authority:
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationLoweringAuthority,
    candidate_service_policy_plan: crate::declaration::UiNormalizedServicePolicyPlan,
    reload_cost: Result<
        crate::runtime::WorthUiReloadLoweringCounterReceipt,
        crate::runtime::WorthUiReloadCounterBoundaryDenial,
    >,
    transition: Option<WorthUiApplicationCutoverTransition>,
}

pub struct WorthUiApplicationSemanticNoOpReceipt {
    receipt: crate::runtime::WorthUiSemanticNoOpReceipt,
    reload_cost: Result<
        crate::runtime::WorthUiReloadLoweringCounterReceipt,
        crate::runtime::WorthUiReloadCounterBoundaryDenial,
    >,
}

#[must_use = "replacement outcomes distinguish authority-preserving no-op from publication"]
pub enum WorthUiApplicationReplacementOutcome {
    SemanticNoOp(Box<WorthUiApplicationSemanticNoOpReceipt>),
    Activated(Box<WorthUiApplicationCutoverReceipt>),
}

#[derive(Debug)]
pub enum WorthUiApplicationReplacementPreparationDenial {
    Preparation(crate::facade::lifecycle::WorthUiApplicationPreparationDenial),
    Admission(crate::runtime::WorthUiCandidateAdmissionReport),
    PreparedApplicationBindingMismatch,
}

#[derive(Debug)]
pub enum WorthUiApplicationReplacementLoweringDenial {
    ForeignActiveApplicationSession,
    Lowering(crate::runtime::WorthUiReplacementLoweringDenial),
}

#[derive(Debug)]
pub enum WorthUiApplicationReplacementStagingDenial {
    ForeignActiveApplicationSession,
    Staging(crate::runtime::WorthUiActivationStagingDenial),
}

#[derive(Debug)]
pub enum WorthUiApplicationCutoverDenial {
    MountedPresentationInFlight,
    ForeignActiveApplicationSession,
    AppearanceOwnerUnavailable(worth_ui_dsl::UiAppearanceStateAxis),
    PreparedApplicationGraphMismatch,
    PreparedApplicationAuthorityMismatch,
    FrameBoundaryUnavailable {
        reason: crate::runtime::WorthUiActivationGateDenialReason,
        retry: Box<WorthUiApplicationCutoverRetry>,
    },
    MountedIdentity(crate::mounting::UiMountedIdentityDenial),
    MountedFrame(crate::mounting::UiMountedFramePreparationDenial),
    MountedPresentationRequired {
        retry: Box<WorthUiApplicationCutoverRetry>,
    },
    PortalExitRetentionPending {
        kind: WorthUiPortalExitRetentionPendingKind,
        retry: Box<WorthUiApplicationCutoverRetry>,
    },
    MissingAllocationCatalogSuccessorReceipt,
    Activation(crate::runtime::WorthUiAllocationCatalogActivationDenial),
}

struct WorthUiApplicationCutoverIdentityEvidence {
    prior_generation: WorthUiPreparedApplicationGenerationIdentity,
    active_generation: WorthUiPreparedApplicationGenerationIdentity,
}

pub(super) enum WorthUiPreparedApplicationCutoverOutcome {
    SemanticNoOp(Box<WorthUiApplicationSemanticNoOpReceipt>),
    Activation(Box<WorthUiPreparedApplicationActivation>),
}

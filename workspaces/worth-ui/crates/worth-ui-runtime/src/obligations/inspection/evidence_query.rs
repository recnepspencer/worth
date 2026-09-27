use crate::obligations::catalog::UiObligationFamily;
use worth_ui_inspection::{
    UiInspectionEvidenceSource, UiInspectionObligationDenialPosture, UiInspectionObligationFamily,
    UiInspectionQuery, UiInspectionTarget,
};

use super::{UiObligationEvidenceDenialPosture, UiObligationEvidencePrerequisiteSource};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiObligationEvidenceQuery {
    handle_digest: Option<u64>,
    graph_node_digest: Option<u64>,
    touch_identity_digest: Option<u64>,
    family: Option<UiObligationFamily>,
    denial_posture: Option<UiObligationEvidenceDenialPosture>,
    prerequisite_source: Option<UiObligationEvidencePrerequisiteSource>,
}

impl UiObligationEvidenceQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_graph_node_digest(mut self, graph_node_digest: u64) -> Self {
        self.graph_node_digest = Some(graph_node_digest);
        self
    }

    pub fn for_handle_digest(mut self, handle_digest: u64) -> Self {
        self.handle_digest = Some(handle_digest);
        self
    }

    pub fn for_touch_identity_digest(mut self, touch_identity_digest: u64) -> Self {
        self.touch_identity_digest = Some(touch_identity_digest);
        self
    }

    pub fn with_family(mut self, family: UiObligationFamily) -> Self {
        self.family = Some(family);
        self
    }

    pub fn with_denial_posture(
        mut self,
        denial_posture: UiObligationEvidenceDenialPosture,
    ) -> Self {
        self.denial_posture = Some(denial_posture);
        self
    }

    pub fn with_prerequisite_source(
        mut self,
        prerequisite_source: UiObligationEvidencePrerequisiteSource,
    ) -> Self {
        self.prerequisite_source = Some(prerequisite_source);
        self
    }

    pub fn graph_node_digest(&self) -> Option<u64> {
        self.graph_node_digest
    }

    pub fn handle_digest(&self) -> Option<u64> {
        self.handle_digest
    }

    pub fn touch_identity_digest(&self) -> Option<u64> {
        self.touch_identity_digest
    }

    pub fn family(&self) -> Option<UiObligationFamily> {
        self.family
    }

    pub fn denial_posture(&self) -> Option<UiObligationEvidenceDenialPosture> {
        self.denial_posture
    }

    pub fn prerequisite_source(&self) -> Option<UiObligationEvidencePrerequisiteSource> {
        self.prerequisite_source
    }

    pub(crate) fn from_inspection_query(query: &UiInspectionQuery) -> Self {
        let mut evidence_query = Self::new();
        match query.target() {
            UiInspectionTarget::ObligationGraphNode { graph_node_digest } => {
                evidence_query = evidence_query.for_graph_node_digest(*graph_node_digest);
            }
            UiInspectionTarget::GraphNodeIdentity { graph_node_digest } => {
                evidence_query = evidence_query.for_graph_node_digest(*graph_node_digest);
            }
            UiInspectionTarget::ObligationTouch {
                graph_node_digest,
                touch_identity_digest,
            } => {
                evidence_query = evidence_query
                    .for_graph_node_digest(*graph_node_digest)
                    .for_touch_identity_digest(*touch_identity_digest);
            }
            UiInspectionTarget::ObligationEvidenceHandle { handle_digest } => {
                evidence_query = evidence_query.for_handle_digest(*handle_digest);
            }
            UiInspectionTarget::ProductRoot | UiInspectionTarget::DeclaredSurface { .. } => {}
            UiInspectionTarget::PublishedAspect { .. }
            | UiInspectionTarget::ConsumedAspect { .. }
            | UiInspectionTarget::DeclarationIdentity { .. }
            | UiInspectionTarget::AuthoredSourceProvenance { .. }
            | _ => {}
        }

        if let Some(obligation_detail) = query.relevance().obligation_detail() {
            if let Some(family) = obligation_detail.family() {
                evidence_query = evidence_query.with_family(runtime_family(family));
            }
            if let Some(denial_posture) = obligation_detail.denial_posture() {
                evidence_query =
                    evidence_query.with_denial_posture(runtime_denial_posture(denial_posture));
            }
            if let Some(prerequisite_source) = obligation_detail.prerequisite_source() {
                evidence_query = evidence_query
                    .with_prerequisite_source(runtime_prerequisite_source(prerequisite_source));
            }
        }

        evidence_query
    }
}

fn runtime_family(family: UiInspectionObligationFamily) -> UiObligationFamily {
    match family {
        UiInspectionObligationFamily::StructuralLegality => UiObligationFamily::StructuralLegality,
        UiInspectionObligationFamily::ParticipationLegality => {
            UiObligationFamily::ParticipationLegality
        }
        UiInspectionObligationFamily::SlotContract => UiObligationFamily::SlotContract,
        UiInspectionObligationFamily::MeasurementRequirement => {
            UiObligationFamily::MeasurementRequirement
        }
        UiInspectionObligationFamily::QueryBindingRequirement => {
            UiObligationFamily::QueryBindingRequirement
        }
        UiInspectionObligationFamily::IntentOperabilityRequirement => {
            UiObligationFamily::IntentOperabilityRequirement
        }
        UiInspectionObligationFamily::PortalHostRequirement => {
            UiObligationFamily::PortalHostRequirement
        }
        UiInspectionObligationFamily::FocusRouteRequirement => {
            UiObligationFamily::FocusRouteRequirement
        }
        UiInspectionObligationFamily::MotionSupportRequirement => {
            UiObligationFamily::MotionSupportRequirement
        }
        UiInspectionObligationFamily::ScrollRoutingRequirement => {
            UiObligationFamily::ScrollRoutingRequirement
        }
        UiInspectionObligationFamily::SelectionStateRequirement => {
            UiObligationFamily::SelectionStateRequirement
        }
        UiInspectionObligationFamily::CommandRouteRequirement => {
            UiObligationFamily::CommandRouteRequirement
        }
        UiInspectionObligationFamily::AccessibilityRequirement => {
            UiObligationFamily::AccessibilityRequirement
        }
        UiInspectionObligationFamily::HostCapabilityRequirement => {
            UiObligationFamily::HostCapabilityRequirement
        }
        UiInspectionObligationFamily::DiagnosticSurfaceRequirement => {
            UiObligationFamily::DiagnosticSurfaceRequirement
        }
    }
}

fn runtime_denial_posture(
    posture: UiInspectionObligationDenialPosture,
) -> UiObligationEvidenceDenialPosture {
    match posture {
        UiInspectionObligationDenialPosture::Unsupported => {
            UiObligationEvidenceDenialPosture::Unsupported
        }
        UiInspectionObligationDenialPosture::Deferred => {
            UiObligationEvidenceDenialPosture::Deferred
        }
        UiInspectionObligationDenialPosture::DiagnosticOnly => {
            UiObligationEvidenceDenialPosture::DiagnosticOnly
        }
        UiInspectionObligationDenialPosture::WrongWorld => {
            UiObligationEvidenceDenialPosture::WrongWorld
        }
        UiInspectionObligationDenialPosture::WrongQueryBasis { required, observed } => {
            UiObligationEvidenceDenialPosture::WrongQueryBasis {
                required: required.into(),
                observed: observed.into(),
            }
        }
        UiInspectionObligationDenialPosture::WrongHostCapability { required, observed } => {
            UiObligationEvidenceDenialPosture::WrongHostCapability {
                required: required.into(),
                observed: observed.into(),
            }
        }
        UiInspectionObligationDenialPosture::Stale {
            required,
            observed,
            evidence,
        } => UiObligationEvidenceDenialPosture::Stale {
            required: required.into(),
            observed: observed.into(),
            evidence: evidence.into(),
        },
        UiInspectionObligationDenialPosture::Ambiguous {
            required_query_basis,
            observed_query_basis,
            required_host_capability,
            observed_host_capability,
        } => UiObligationEvidenceDenialPosture::Ambiguous {
            required_query_basis: required_query_basis.map(Into::into),
            observed_query_basis: observed_query_basis.map(Into::into),
            required_host_capability: required_host_capability.map(Into::into),
            observed_host_capability: observed_host_capability.map(Into::into),
        },
        UiInspectionObligationDenialPosture::BudgetExceeded {
            budget,
            attempted_lane_cost,
        } => UiObligationEvidenceDenialPosture::BudgetExceeded {
            budget: budget.into(),
            attempted_lane_cost,
        },
    }
}

fn runtime_prerequisite_source(
    source: UiInspectionEvidenceSource,
) -> UiObligationEvidencePrerequisiteSource {
    match source {
        UiInspectionEvidenceSource::WorthLocal => {
            UiObligationEvidencePrerequisiteSource::QueryBasis
        }
        UiInspectionEvidenceSource::QueryInspection => {
            UiObligationEvidencePrerequisiteSource::QueryInspection
        }
        UiInspectionEvidenceSource::QueryProjectionConsumption => {
            UiObligationEvidencePrerequisiteSource::QueryProjectionConsumption
        }
        UiInspectionEvidenceSource::QueryCausalExplanation => {
            UiObligationEvidencePrerequisiteSource::QueryCausalExplanation
        }
        UiInspectionEvidenceSource::HostCapability => {
            UiObligationEvidencePrerequisiteSource::HostCapability
        }
    }
}

impl From<worth_ui_inspection::UiInspectionAdmissionQueryBasis>
    for crate::admission::UiAdmissionQueryBasis
{
    fn from(value: worth_ui_inspection::UiInspectionAdmissionQueryBasis) -> Self {
        match value {
            worth_ui_inspection::UiInspectionAdmissionQueryBasis::GraphAligned => {
                Self::GraphAligned
            }
            worth_ui_inspection::UiInspectionAdmissionQueryBasis::WrongWorldProjection => {
                Self::WrongWorldProjection
            }
            worth_ui_inspection::UiInspectionAdmissionQueryBasis::RebindRequired => {
                Self::RebindRequired
            }
            worth_ui_inspection::UiInspectionAdmissionQueryBasis::StaleReceipt => {
                Self::StaleReceipt
            }
            worth_ui_inspection::UiInspectionAdmissionQueryBasis::AmbiguousSources => {
                Self::AmbiguousSources
            }
        }
    }
}

impl From<worth_ui_inspection::UiInspectionAdmissionHostCapability>
    for crate::admission::UiAdmissionHostCapability
{
    fn from(value: worth_ui_inspection::UiInspectionAdmissionHostCapability) -> Self {
        match value {
            worth_ui_inspection::UiInspectionAdmissionHostCapability::Available => Self::Available,
            worth_ui_inspection::UiInspectionAdmissionHostCapability::Missing => Self::Missing,
            worth_ui_inspection::UiInspectionAdmissionHostCapability::Ambiguous => Self::Ambiguous,
        }
    }
}

impl From<worth_ui_inspection::UiInspectionAdmissionStaleEvidence>
    for crate::admission::UiAdmissionStaleEvidence
{
    fn from(value: worth_ui_inspection::UiInspectionAdmissionStaleEvidence) -> Self {
        match value {
            worth_ui_inspection::UiInspectionAdmissionStaleEvidence::DeclarationArtifactMissing => {
                Self::DeclarationArtifactMissing
            }
            worth_ui_inspection::UiInspectionAdmissionStaleEvidence::QueryReceiptExpired => {
                Self::QueryReceiptExpired
            }
        }
    }
}

impl From<worth_ui_inspection::UiInspectionSelectionBudget>
    for crate::admission::UiAdmissionSelectionBudget
{
    fn from(value: worth_ui_inspection::UiInspectionSelectionBudget) -> Self {
        match value {
            worth_ui_inspection::UiInspectionSelectionBudget::Unbounded => Self::Unbounded,
            worth_ui_inspection::UiInspectionSelectionBudget::OrdinaryLaneBudget { lane_limit } => {
                Self::OrdinaryLaneBudget { lane_limit }
            }
        }
    }
}

//! Original READ and metadata are admitted before any protected target traversal.
mod plan;

mod source_scope;
pub(crate) mod subject;
pub(super) use subject::InspectionSubject;
pub(super) mod connected;
pub(super) mod policy;
pub(crate) mod preparation;
pub(super) use preparation::prepare;
mod affinity;
pub(super) use affinity::validate_original_affinity;
mod comparison;
mod currentness;
mod governance_binding;
mod materialization;

pub(crate) mod rich;
pub(crate) mod terminal;
use super::target::InspectionTargetDenial;
use super::WorthQueryApplicationInspectionTarget;
use crate::domain_computation::{
    authorization::{
        application_disclosure::WorthQueryApplicationQueryGovernance,
        WorthQueryRetainedAuthorizationDecisionFacts,
    },
    primary_graph::{
        WorthQueryApplicationBasisIdentity, WorthQueryApplicationEntityIdentity,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    provider_session::{WorthQueryManagedGraphWorkSession, WorthQuerySessionInspectionReadProof},
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub enum InspectionAdmissionDenial {
    Request(worth_execution::WorkCeilingDenial),
    RequestControl(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption),
    ForeignIssuedAccess,
    MissingGraph,
    MissingInspection,
    #[allow(dead_code, reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it")]
    Authorization(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial),
    #[allow(dead_code, reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it")]
    CurrentAuthorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind),
    #[allow(dead_code, reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it")]
    Capability(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    #[allow(dead_code, reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it")]
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    #[allow(dead_code, reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it")]
    Security(crate::basis::WorthQueryProductBranchAdmissionDenial),
    #[allow(dead_code, reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it")]
    Governance(crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernanceDenialKind),
    FacetDenied,
    DiagnosticAdmission(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial>),
    ComparisonParameters(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<()>),
    Connection(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<super::connection::ConnectionRefusal>),
    MissingOriginalProjection,
    RichSource(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<super::rich::source::SourceMaterializationDenial>),
    Rich(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<crate::domain_computation::primary_graph::application_query::WorthQueryProjectionMaterializationRefusal>),
    Comparison(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial>),
    DiagnosticContract(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial>),
    DiagnosticPreparation(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<crate::domain_computation::authorization::application_disclosure::DiagnosticPreparationDenial>),
    Target(InspectionTargetDenial),
    Storage(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<crate::domain_computation::primary_graph::admitted_capture::RetainedStorageDenial>),
    Plan(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<plan::InspectionPlanDenial>),
    #[allow(dead_code, reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it")]
    Session(crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSessionStartDenial),
    Read(crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial),
    Terminal(crate::domain_computation::provider_session::WorthQueryInspectionReadTerminalDenial),
    Report(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<super::report::InspectionReportDenial>),
}

pub struct InspectionRead<'a, 'storage, Schema, Scope> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    request: &'a WorthQueryRequestScope,
    principal: worth_relational::facade::identity::EntityId,
    basis: WorthQueryApplicationBasisIdentity,
    target: Arc<WorthQueryApplicationInspectionTarget>,
    session: WorthQueryManagedGraphWorkSession,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    governance: WorthQueryApplicationQueryGovernance,
    diagnostic: Option<WorthQueryApplicationQueryGovernance>,
    source: Option<connected::SourceRead<'a>>,
    memory: &'storage mut Option<ExecutionMemoryReservation>,
}

pub struct InspectionComparisonPreparation<'original> {
    pub(super) before: &'original super::super::application_observation::WorthQueryApplicationQueryObservation,
    pub(super) pending: crate::domain_computation::authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance,
}

pub enum InspectionDiagnosticPreparation<'o> {
    Comparison(InspectionComparisonPreparation<'o>),
    Rich {
        request:super::rich::WorthQueryApplicationMaterializationRequest,
        pending:crate::domain_computation::authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance,
    },
}
impl<'o> InspectionDiagnosticPreparation<'o> {
    fn before(
        &self,
    ) -> Option<&'o super::super::application_observation::WorthQueryApplicationQueryObservation>
    {
        match self {
            Self::Comparison(value) => Some(value.before),
            Self::Rich { .. } => None,
        }
    }
    fn rich_request(&self) -> Option<super::rich::WorthQueryApplicationMaterializationRequest> {
        match self {
            Self::Rich { request, .. } => Some(*request),
            _ => None,
        }
    }
    fn into_pending(self)->crate::domain_computation::authorization::application_disclosure::WorthQueryPendingApplicationQueryGovernance{
        match self {
            Self::Comparison(value) => value.pending,
            Self::Rich { pending, .. } => pending,
        }
    }
}
pub fn require_request(
    execution: &ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
) -> Result<(), InspectionAdmissionDenial> {
    if let Some(cause) = request.interruption() {
        return Err(InspectionAdmissionDenial::RequestControl(cause));
    }
    execution
        .consult()
        .map_err(InspectionAdmissionDenial::Request)
}

mod court_doors_for_test;

pub use plan::InspectionPlanDenial;

//! Original READ and metadata are admitted before any protected target traversal.
mod plan;
pub(super) use plan::InspectionPlanDenial;
mod preparation;
pub(super) use preparation::{prepare, validate_original_affinity};
mod currentness;
mod materialization;
#[cfg(test)]
mod publication_tests;
mod terminal;
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
pub(super) use terminal::InspectionTerminal;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

pub(super) enum InspectionAdmissionDenial {
    Request(worth_execution::WorkCeilingDenial),
    RequestControl(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption),
    ForeignIssuedAccess,
    MissingGraph,
    MissingInspection,
    Authorization(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial),
    CurrentAuthorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind),
    Capability(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Security(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Governance(crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernanceDenialKind),
    FacetDenied,
    Target(InspectionTargetDenial),
    Storage(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<crate::domain_computation::primary_graph::admitted_capture::RetainedStorageDenial>),
    Plan(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<plan::InspectionPlanDenial>),
    Session(crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSessionStartDenial),
    Read(crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial),
    Terminal(crate::domain_computation::provider_session::WorthQueryInspectionReadTerminalDenial),
    Report(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<super::report::InspectionReportDenial>),
}

pub(super) struct InspectionRead<'a, 'storage, Schema, Scope> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    request: &'a WorthQueryRequestScope,
    principal: worth_relational::facade::identity::EntityId,
    basis: WorthQueryApplicationBasisIdentity,
    target: Arc<WorthQueryApplicationInspectionTarget>,
    session: WorthQueryManagedGraphWorkSession,
    proof: Option<WorthQuerySessionInspectionReadProof>,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    governance: WorthQueryApplicationQueryGovernance,
    memory: &'storage mut Option<ExecutionMemoryReservation>,
}

pub(super) fn require_request(
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

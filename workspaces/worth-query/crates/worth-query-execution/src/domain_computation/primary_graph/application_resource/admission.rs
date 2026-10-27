//! Fresh Resource action and independent facet admission on one real session.
mod currentness;
mod plan;
mod preparation;
use crate::domain_computation::{
    authorization::{
        application_disclosure::WorthQueryApplicationQueryGovernance,
        WorthQueryRetainedAuthorizationDecisionFacts,
    },
    primary_graph::{
        WorthQueryApplicationBasisIdentity, WorthQueryApplicationEntityIdentity,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    provider_session::{WorthQueryManagedGraphWorkSession, WorthQuerySessionResourceReadProof},
};
pub(super) use preparation::prepare;
use std::sync::Arc;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::WorthQueryInstalledResourceAccessContract;

pub(super) enum ResourceAdmissionDenial {
    Request(worth_execution::WorkCeilingDenial),
    RequestControl(worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption),
    ForeignIssuedAccess,
    OriginalScopeGenerationChanged,
    MissingGraph,
    MissingPrincipalBinding,
    Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    DisclosureAuthorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Security(crate::basis::WorthQueryProductBranchAdmissionDenial),
    Contract(crate::domain_computation::authorization::application_disclosure::contract::WorthQueryApplicationDisclosureContractDenial),
    Governance(crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernanceDenialKind),
    FacetDenied,
    Plan(crate::domain_computation::primary_graph::admitted_capture::CaptureStop<plan::ResourcePlanDenial>),
    Session(crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSessionStartDenial),
    Read(crate::domain_computation::provider_session::WorthQueryManagedGraphReadDenial),
}

/// The issued typed scope stays borrowed through the final currentness check.
/// No request, permission or native cleanup capability is put into a receipt.
pub(super) struct ResourceActionRead<'a, Schema, Scope> {
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    request: &'a WorthQueryRequestScope,
    principal: worth_relational::facade::identity::EntityId,
    parameters: CanonicalDigestId,
    basis: WorthQueryApplicationBasisIdentity,
    contract: Arc<WorthQueryInstalledResourceAccessContract>,
    session: WorthQueryManagedGraphWorkSession,
    proof: Option<WorthQuerySessionResourceReadProof>,
    authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    governance: WorthQueryApplicationQueryGovernance,
    _admission: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(super) struct ResourceActionTerminal {
    session: WorthQueryManagedGraphWorkSession,
    proof: WorthQuerySessionResourceReadProof,
    _authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    _governance: WorthQueryApplicationQueryGovernance,
    _admission: Option<worth_execution::ExecutionMemoryReservation>,
}

impl ResourceActionTerminal {
    pub(super) fn complete(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        release: crate::domain_computation::primary_graph::WorthQueryApplicationBasisReleaseReceipt,
    ) -> Result<
        crate::domain_computation::provider_session::WorthQueryResourceReadCompletion,
        ResourceAdmissionDenial,
    > {
        self.session
            .complete_resource_read(execution, self.proof, release)
            .map_err(ResourceAdmissionDenial::Read)
    }
}

fn require_request(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
) -> Result<(), ResourceAdmissionDenial> {
    if let Some(cause) = request.interruption() {
        return Err(ResourceAdmissionDenial::RequestControl(cause));
    }
    execution
        .consult()
        .map_err(ResourceAdmissionDenial::Request)
}

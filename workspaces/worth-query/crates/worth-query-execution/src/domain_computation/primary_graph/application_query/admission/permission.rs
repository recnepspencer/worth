//! Fresh Query permission retained across a Clean probe or one graph read.

use worth_query_declaration::facade::application_query::ApplicationQueryDisclosurePosture;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryAuthorization;

use super::super::basis::WorthQueryApplicationQueryBasisCustody;
use super::super::controls::WorthQueryAdmittedApplicationQueryControls;
use super::super::disclosure::compile_disclosure_contract;
use super::super::disclosure::WorthQueryAdmittedApplicationDisclosureContract;
use super::super::WorthQueryApplicationAuthorizationWorkEvidence;
use super::super::WorthQueryApplicationQueryAccessContext;
use super::super::WorthQueryApplicationQueryAdmissionDenial;
use super::super::WorthQueryApplicationQueryAdmissionDenialKind;
use super::source_readmission::PreparedApplicationQuerySourceReadmission;
use crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts;

use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedPrincipal;
use crate::domain_computation::primary_graph::WorthQueryIssuedSelectedScope;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use crate::domain_computation::provider_session::WorthQueryPreparedQuerySessionIdentity;

mod current_ready;
mod fresh;
mod principal_dependency;
mod selected_access;
mod source_parameters;
pub(in crate::domain_computation::primary_graph) use fresh::FreshQueryPermissionStop;
pub(in crate::domain_computation::primary_graph::application_query) use selected_access::SelectedIssuedAccessRoot;

/// Query-owned permission for the exact prepared source, access and Product.
/// Its fields cannot be separated into reusable raw decisions by a caller.
pub(in crate::domain_computation::primary_graph) struct PreparedApplicationQueryPermission<
    'a,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    pub(super) issuer: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) query:
        &'a WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    pub(super) access:
        WorthQueryApplicationQueryAccessContext<'a, Schema, Principal, PrincipalIdentity, Scope>,
    pub(super) parameters:
        worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters,
    pub(super) controls: WorthQueryAdmittedApplicationQueryControls<'a>,
    pub(super) basis: WorthQueryApplicationQueryBasisCustody,
    pub(super) security_product: crate::basis::WorthQueryProductObservationLease,
    pub(super) session: WorthQueryPreparedQuerySessionIdentity,
    pub(super) disclosure: WorthQueryAdmittedApplicationDisclosureContract,
    pub(super) authorization: WorthQueryRetainedAuthorizationDecisionFacts,
    pub(super) authorization_work: WorthQueryApplicationAuthorizationWorkEvidence,
    pub(super) selected_access: SelectedIssuedAccessRoot<'a, Schema, Principal, PrincipalIdentity, Scope>,
    pub(super) shape: super::super::execution_shape::PreparedOneShotShape<'a>,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    fn prepare_application_query_permission<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        source: PreparedApplicationQuerySourceReadmission<'a, Schema>,
        selected: &'a WorthQuerySelectedProductOperation<'_, Schema>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        issued_principal: &'a WorthQueryIssuedSelectedPrincipal<
            '_,
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        issued_scope: &'a WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>,
    ) -> Result<
        PreparedApplicationQueryPermission<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let (query_identity, parameters, controls) = source.into_permission_parts();
        self.prepare_application_query_permission_with_parameters(
            execution,
            query,
            access,
            &query_identity,
            parameters,
            controls,
            selected,
            request,
            issued_principal,
            issued_scope,
            &mut super::super::ApplicationQueryNativeAccess::acquire(),
        )
    }
}

fn denial(
    kind: WorthQueryApplicationQueryAdmissionDenialKind,
    subject: &str,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::new(kind, subject)
}

use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;

use super::super::WorthQueryAdmittedApplicationQueryPlan;
use super::super::WorthQueryApplicationQueryAccessContext;
use super::super::WorthQueryApplicationQueryAdmissionDenial;
use super::super::WorthQueryApplicationQueryAdmissionDenialKind;
use super::super::WorthQueryApplicationQueryControls;
use super::super::WorthQueryObservedSource;

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

/// The original selector and parameters have passed fresh installed-query,
/// principal, scope, request and parent-meter admission. The selected basis
/// remains owned here until Query read permission and graph work are admitted.
pub(in crate::domain_computation::primary_graph) struct PreparedApplicationQuerySourceReadmission<
    'a,
    Schema,
> {
    query_identity: WorthQueryInstalledApplicationQueryIdentity,
    parameters: WorthQueryAdmittedApplicationQueryParameters,
    controls: WorthQueryApplicationQueryControls<'a, Schema>,
}

impl<'a, Schema> PreparedApplicationQuerySourceReadmission<'a, Schema> {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        WorthQueryAdmittedApplicationQueryParameters,
        WorthQueryApplicationQueryControls<'a, Schema>,
    ) {
        (self.parameters, self.controls)
    }

    pub(in crate::domain_computation::primary_graph::application_query) fn into_permission_parts(
        self,
    ) -> (
        WorthQueryInstalledApplicationQueryIdentity,
        WorthQueryAdmittedApplicationQueryParameters,
        WorthQueryApplicationQueryControls<'a, Schema>,
    ) {
        (self.query_identity, self.parameters, self.controls)
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Reissue the retained source's parameters only after a fresh principal,
    /// scope, installed query, request, and basis have been selected.
    pub(in crate::domain_computation::primary_graph) fn readmit_application_query_from_observed<
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
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        observed: &WorthQueryObservedSource<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
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
        let prepared =
            self.prepare_application_query_from_observed(query, access, observed, controls)?;
        let (parameters, controls) = prepared.into_parts();
        self.finish_application_query_readmission(execution, query, access, parameters, controls)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_application_query_from_observed<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        observed: &WorthQueryObservedSource<Query>,
        controls: WorthQueryApplicationQueryControls<'a, Schema>,
    ) -> Result<
        PreparedApplicationQuerySourceReadmission<'a, Schema>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let selector = observed.retained_scope_selector();

        if observed.runtime_authority != self.runtime.authority_identity().as_u64()
            || observed.query_identity != *query.identity()
            || observed.schema_binding != *query.binding_identity()
        {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::InstalledQuery(
                    worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenialKind::StaleGeneration,
                ),
                query.name(),
            ));
        }
        if selector.locator() != access.scope().identity_locator()
            || selector.value() != access.scope().identity_value()
        {
            return Err(WorthQueryApplicationQueryAdmissionDenial::new(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            ));
        }
        let (parameters, controls) = self.prepare_application_query_source_readmission(
            query,
            access,
            &observed.parameters,
            controls,
        )?;
        Ok(PreparedApplicationQuerySourceReadmission {
            query_identity: query.identity().clone(),
            parameters,
            controls,
        })
    }
}

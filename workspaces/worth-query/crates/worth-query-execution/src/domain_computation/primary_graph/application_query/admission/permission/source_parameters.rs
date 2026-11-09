//! One permission owner for fresh source intent and retained-source readmission.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph::application_query) fn prepare_application_query_permission_with_parameters<
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
        source_query_identity: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        parameters: worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters,
        controls: super::super::WorthQueryApplicationQueryControls<'a, Schema>,
        selected: &'a WorthQuerySelectedProductOperation<'_, Schema>,
        truth: Option<&'a WorthQuerySelectedProductOperation<'_, Schema>>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        issued_principal: &'a WorthQueryIssuedSelectedPrincipal<
            '_,
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        issued_scope: &'a WorthQueryIssuedSelectedScope<'_, '_, Schema, Scope>,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
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
        self.prepare_application_query_permission_on_root(
            execution,
            query,
            access,
            source_query_identity,
            parameters,
            controls,
            selected,
            truth,
            request,
            |access| {
                let selected_access = SelectedIssuedAccessRoot::from_issued(
                    selected,
                    access,
                    issued_principal,
                    issued_scope,
                    request,
                    query.name(),
                )?;
                let selected_access = match truth {
                    Some(truth) => selected_access.with_truth_from_selected(truth),
                    None => selected_access,
                };
                Ok(selected_access)
            },
            native,
        )
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph::application_query) fn prepare_historical_source_query_permission_with_parameters<
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
        source_query_identity: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        parameters: worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters,
        controls: super::super::WorthQueryApplicationQueryControls<'a, Schema>,
        selected: &'a WorthQuerySelectedProductOperation<'_, Schema>,
        truth: Option<&'a WorthQuerySelectedProductOperation<'_, Schema>>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        issued_principal: &'a WorthQueryIssuedSelectedPrincipal<
            '_,
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation<'a>,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
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
        let source = access.scope().entity_id();
        self.prepare_application_query_permission_on_root(
            execution,
            query,
            access,
            source_query_identity,
            parameters,
            controls,
            selected,
            truth,
            request,
            |access| {
                let selected_access = SelectedIssuedAccessRoot::from_historical_source(
                    selected,
                    truth.ok_or_else(|| {
                        denial(
                            WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                            query.name(),
                        )
                    })?,
                    access,
                    issued_principal,
                    history,
                    request,
                    query.name(),
                )?;
                Ok(selected_access)
            },
            native,
        )
        .map_err(|cause| cause.with_historical_source(source, history.family()))
    }
}

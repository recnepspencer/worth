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
        let selected_observation = selected.product().observation();

        let selected_snapshot = selected.application_basis().snapshot_handle();

        // A disclosure-owner denial and its Query translation can own the
        // query name simultaneously. Reserve both before that owner runs.

        let same_basis = controls
            .selected_basis()
            .is_some_and(|(product, snapshot)| {
                product.observation() == selected_observation && snapshot == selected_snapshot
            });
        if source_query_identity != query.identity() || !same_basis {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                query.name(),
            ));
        }

        let selected_access = SelectedIssuedAccessRoot::from_issued(
            selected,
            &access,
            issued_principal,
            issued_scope,
            request,
            query.name(),
        )?;
        crate::domain_computation::primary_graph::application_query::admission_preparation::validate_admission_request(request, query.name())?;
        let (basis_selection, security_product, controls) = controls.into_admission_parts();
        let shape = crate::domain_computation::primary_graph::application_query::execution_shape::prepare_one_shot_shape_admitted(query, &controls)?;
        let basis =
            crate::domain_computation::primary_graph::application_query::basis::admit_application_query_permission_basis_with_native_access(
                execution,
                self,
                basis_selection,
                native,
            )?;
        if !selected.application_basis().selected_program_inspected() {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                query.name(),
            ));
        }
        if query.disclosure().posture() == ApplicationQueryDisclosurePosture::Governed {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureGovernanceRequired,
                query.name(),
            ));
        }
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                query.name(),
            )
        })?;
        let disclosure = compile_disclosure_contract(query, &graph.layout).map_err(|stop| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::DisclosureContractInvalid,
                stop.subject(),
            )
        })?;

        let session = WorthQueryPreparedProductReadSessionIdentity::reserve().map_err(|_| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable,
                query.name(),
            )
        })?;
        let (authorization, authorization_work) = self
            .observe_issued_query_read_authorization_with_native(
                execution,
                query,
                &selected_access,
                &session,
                native,
            )?;
        Ok(PreparedApplicationQueryPermission {
            issuer: self,
            query,
            access,
            parameters,
            controls,
            basis,
            security_product,
            session,
            disclosure,
            authorization,
            authorization_work: authorization_work.with_admission_security_product_resolution(),
            selected_access,
            shape,
        })
    }
}

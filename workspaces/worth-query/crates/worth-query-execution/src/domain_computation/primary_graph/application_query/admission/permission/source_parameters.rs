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
        admission: &mut InvalidationEditAdmission,
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
        let name_bytes = u64::try_from(query.name().len()).map_err(|_| work_denial())?;
        let selected_observation = selected.product().observation();
        let basis_work = u64::try_from(
            worth_runtime_world::facade::CurrentProductHead::comparison_work_bound(
                selected_observation,
            ),
        )
        .map_err(|_| work_denial())?;
        let selected_snapshot = selected.application_basis().snapshot_handle();
        let native_branch_bytes =
            u64::try_from(selected_snapshot.branch_id().0.len()).map_err(|_| work_denial())?;
        let prepared_branch_bytes = controls
            .selected_basis()
            .map(|(_, snapshot)| snapshot.branch_id().0.len())
            .map(u64::try_from)
            .transpose()
            .map_err(|_| work_denial())?
            .unwrap_or(0);
        // A disclosure-owner denial and its Query translation can own the
        // query name simultaneously. Reserve both before that owner runs.
        let denial_names = name_bytes.checked_mul(2).ok_or_else(work_denial)?;
        let entry_work = denial_names
            .checked_add(8)
            .and_then(|n| n.checked_add(basis_work))
            .and_then(|n| n.checked_add(native_branch_bytes))
            .and_then(|n| n.checked_add(prepared_branch_bytes))
            .and_then(|n| n.checked_add(32))
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(entry_work)
            .map_err(|_| work_denial())?;
        admission
            .admit_read_scratch(denial_names)
            .map_err(resource_denial)?;
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
        admission
            .charge_external_work(8)
            .map_err(|_| work_denial())?;
        if !std::ptr::eq(issued_principal.selected(), selected)
            || !std::ptr::eq(issued_scope.selected(), selected)
            || !std::ptr::eq(issued_principal.principal(), access.principal())
            || !std::ptr::eq(issued_scope.scope(), access.scope())
            || !issued_principal.request().same_request(request)
            || !issued_scope.request().same_request(request)
            || issued_principal.issued_root().admission_identity()
                != issued_scope.issued_root().admission_identity()
            || !std::ptr::eq(
                issued_principal.issued_snapshot(),
                issued_scope.issued_snapshot(),
            )
        {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                query.name(),
            ));
        }
        let selected_access = SelectedIssuedAccessRoot {
            principal: issued_principal.principal(),
            scope: issued_scope.scope(),
            root: issued_principal.issued_root(),
            snapshot: issued_principal.issued_snapshot(),
            product: selected.product().observation(),
            request: issued_principal.request(),
        };
        crate::domain_computation::primary_graph::application_query::admission_preparation::validate_admission_request(request, query.name())?;
        let (basis_selection, security_product, controls) = controls.into_admission_parts();
        let shape = crate::domain_computation::primary_graph::application_query::execution_shape::prepare_one_shot_shape_admitted(
            query, &controls, admission,
        )?;
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
        admission
            .charge_external_work(2)
            .map_err(|_| work_denial())?;
        let session = WorthQueryPreparedQuerySessionIdentity::reserve().map_err(|_| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable,
                query.name(),
            )
        })?;
        let principal_currentness =
            principal_dependency::capture(self, access.principal(), session.identity(), admission)?;
        let (authorization, authorization_work) = match query.authorization() {
            WorthQueryInstalledApplicationQueryAuthorization::Public => (
                WorthQueryRetainedAuthorizationDecisionFacts::principal(principal_currentness),
                WorthQueryApplicationAuthorizationWorkEvidence::from_dependencies(&[]),
            ),
            WorthQueryInstalledApplicationQueryAuthorization::Ability(requirement) => {
                // A semantic refusal retains the original operation denial in
                // one Query-owned Box. Reserve its backing before observation.
                admission
                    .charge_external_work(1)
                    .map_err(|_| work_denial())?;
                admission
                    .admit_read_scratch(u64::try_from(std::mem::size_of::<
                        crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial,
                    >()).map_err(|_| work_denial())?)
                    .map_err(resource_denial)?;
                admission
                    .charge_external_work(native_branch_bytes.saturating_add(1))
                    .map_err(|_| work_denial())?;
                admission
                    .admit_read_scratch(native_branch_bytes)
                    .map_err(resource_denial)?;
                let decisions = native.with_runtime_mut(graph, execution, || AdmittedQueryAuthorizationStop::Authorization(crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenialKind::Refused(
                            crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::ForeignRuntime), query.name())), |relational| {
                        self.observe_query_authorization_requirement_admitted(
                            execution,
                            session.identity(),
                            relational,
                            selected_snapshot.clone(),
                            query,
                            &access,
                            requirement,
                            admission,
                        )
                    })
                    .map_err(|stop| match stop {
                        AdmittedQueryAuthorizationStop::Authorization(authorization) => {
                            WorthQueryApplicationQueryAdmissionDenial::from_authorization(
                                authorization,
                            )
                        }
                        AdmittedQueryAuthorizationStop::Preparation(resource) => {
                            resource_denial(resource)
                        }
                        AdmittedQueryAuthorizationStop::AccountingOverflow => work_denial(),
                    })?;
                admission
                    .charge_external_work(1)
                    .map_err(|_| work_denial())?;
                let work =
                    WorthQueryApplicationAuthorizationWorkEvidence::from_dependencies(&decisions);
                (
                    WorthQueryRetainedAuthorizationDecisionFacts::abilities(
                        principal_currentness,
                        decisions,
                    ),
                    work,
                )
            }
        };
        if !authorization.belongs_to_session(session.identity()) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable,
                query.name(),
            ));
        }
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

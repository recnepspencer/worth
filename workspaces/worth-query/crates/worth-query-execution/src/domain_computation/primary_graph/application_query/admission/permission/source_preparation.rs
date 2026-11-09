//! One permission owner for fresh source intent and retained-source readmission.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph::application_query) fn prepare_application_query_permission_on_root<
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
        issue_access: impl FnOnce(
            &WorthQueryApplicationQueryAccessContext<
                'a,
                Schema,
                Principal,
                PrincipalIdentity,
                Scope,
            >,
        ) -> Result<
            SelectedIssuedAccessRoot<'a, Schema, Principal, PrincipalIdentity, Scope>,
            WorthQueryApplicationQueryAdmissionDenial,
        >,

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
        let truth_selected = truth.unwrap_or(selected);
        let selected_observation = truth_selected.product().observation();

        let selected_snapshot = truth_selected.application_basis().snapshot_handle();

        // A disclosure-owner denial and its Query translation can own the
        // query name simultaneously. Reserve both before that owner runs.

        let same_basis = controls
            .selected_basis()
            .is_some_and(|(product, snapshot)| {
                product.observation() == selected_observation && snapshot == selected_snapshot
            });
        if source_query_identity != query.identity()
            || !same_basis
            || !controls.has_security_product(selected.product().observation())
        {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                query.name(),
            ));
        }

        let selected_access = issue_access(&access)?;
        // Reuse the actual funded C share. D remains the independent truth basis;
        // retaining C here creates no new Native snapshot or authorization.
        let security_basis = truth.map(|_| WorthQueryApplicationQueryBasisCustody::from_issued_selected(self, selected))
            .transpose().map_err(|cause| {
                use crate::domain_computation::primary_graph::product_operation::SelectedQueryBasisRetentionStop as Stop;
                match cause {
                    Stop::Basis => denial(WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis, query.name()),
                    Stop::Admission(stop) => denial(match stop {
                        worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => WorthQueryApplicationQueryAdmissionDenialKind::CanonicalWorkDenied,
                        _ => WorthQueryApplicationQueryAdmissionDenialKind::ReadmissionPreparationMemoryExhausted,
                    }, query.name()),
                }
            })?;
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
        if !selected.application_basis().selected_program_inspected()
            || !truth_selected
                .application_basis()
                .selected_program_inspected()
        {
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
        selected_access.require_source_after_authorization(
            self,
            execution,
            native,
            query.name(),
        )?;
        Ok(PreparedApplicationQueryPermission {
            issuer: self,
            query,
            access,
            parameters,
            controls,
            basis,
            security_basis,
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

use super::*;
use crate::domain_computation::primary_graph::product_operation::MatchingHeadScopedOps;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind;

/// The exact prepared Query permission after its issued World head was
/// checked. Actor currentness still runs outside the World guard.
pub(in crate::domain_computation::primary_graph) struct SealedCurrentReadyQueryPermission<'basis> {
    security: crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedPermissionSecurityBasis<'basis>,
    truth: Option<crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedPermissionSecurityBasis<'basis>>,
    authentication_valid_until: std::time::Instant,
    query_name: &'basis str,
    _query_identity: &'basis worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
    _parameter_identity: &'basis worth_foundational::facade::CanonicalDigestId,
    _request: &'basis worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    _product: &'basis crate::basis::WorthQueryProductObservationLease,
    _authorization: &'basis WorthQueryRetainedAuthorizationDecisionFacts,
}

impl SealedCurrentReadyQueryPermission<'_> {
    pub(in crate::domain_computation::primary_graph) fn validate_request(
        &self,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        validate_current_request(
            self._request,
            self.authentication_valid_until,
            self.query_name,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn product(
        &self,
    ) -> &crate::basis::WorthQueryProductObservationLease {
        self._product
    }

    pub(in crate::domain_computation::primary_graph) fn snapshot_handle(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.truth
            .as_ref()
            .unwrap_or(&self.security)
            .snapshot_handle()
    }
}

fn validate_current_request(
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    valid_until: std::time::Instant,
    query: &str,
) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
    super::super::super::admission_preparation::validate_admission_request(request, query)?;
    if request.clock().now() >= valid_until {
        return Err(denial(WorthQueryApplicationQueryAdmissionDenialKind::Authorization(
            crate::domain_computation::authorization::WorthQueryAdmissionLapse::AuthenticationExpired.denial_kind(),
        ), query));
    }
    Ok(())
}

impl<Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    PreparedApplicationQueryPermission<
        '_,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn validate_current_request(
        &self,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        validate_current_request(
            self.controls.request_scope(),
            self.access.principal().valid_until(),
            self.query.name(),
        )
    }

    /// A moved head makes the old selected basis ineligible for a Clean result.
    /// The caller may continue only through fresh disclosure/reselection.
    pub(in crate::domain_computation::primary_graph) fn seal_current_ready<'basis>(
        &'basis self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<SealedCurrentReadyQueryPermission<'basis>>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        self.seal_current_ready_with_control(execution, None)
    }

    /// Each exact C/D input keeps its own genuine Query basis. The security
    /// snapshot cannot stand in for the output's current truth snapshot.
    pub(in crate::domain_computation::primary_graph) fn seal_current_ready_with_control<'basis>(
        &'basis self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: Option<&'basis WorthQueryApplicationQueryBasisCustody>,
    ) -> Result<
        Option<SealedCurrentReadyQueryPermission<'basis>>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        use crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop;
        let security = self
            .issuer
            .admit_matching_head_query_permission_basis(
                execution,
                &self.security_product,
                control.unwrap_or(&self.basis),
            )
            .map_err(|SelectedPermissionSecurityStop| {
                denial(
                    WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                    self.query.name(),
                )
            })?;

        let Some(security) = security else {
            return Ok(None);
        };
        let truth = if control.is_some() {
            let truth = self
                .issuer
                .admit_matching_head_query_permission_basis(
                    execution,
                    self.basis.selected_product(),
                    &self.basis,
                )
                .map_err(|SelectedPermissionSecurityStop| {
                    denial(
                        WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                        self.query.name(),
                    )
                })?;
            let Some(truth) = truth else {
                return Ok(None);
            };
            Some(truth)
        } else {
            None
        };
        super::super::super::admission_preparation::validate_admission_request(
            self.controls.request_scope(),
            self.query.name(),
        )?;
        let sealed = SealedCurrentReadyQueryPermission {
            security,
            truth,
            authentication_valid_until: self.access.principal().valid_until(),
            query_name: self.query.name(),
            _query_identity: self.query.identity(),
            _parameter_identity: self.parameters.identity(),
            _request: self.controls.request_scope(),
            _product: self.basis.selected_product(),
            _authorization: &self.authorization,
        };
        sealed.validate_request()?;
        Ok(Some(sealed))
    }

    pub(in crate::domain_computation::primary_graph) fn require_original_scope(
        &self,
        observed: &crate::domain_computation::primary_graph::WorthQueryObservedSource<Query>,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        if self.access.scope().entity_id() != observed.retained_scope_selector().subject() {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                self.query.name(),
            ));
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn require_history_principal(
        &self,
        history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerInvocation<'_>,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        let principal = crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryOperationScopeEntityBinding::from_entity(
            self.access.principal().principal_entity_id(),
        );
        if !history.admits_principal(&principal) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
                self.query.name(),
            ));
        }
        Ok(())
    }
}

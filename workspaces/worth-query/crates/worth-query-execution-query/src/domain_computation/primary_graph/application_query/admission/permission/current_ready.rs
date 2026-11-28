use super::*;
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;

/// The exact prepared Query permission after its issued World head was
/// checked. Actor currentness still runs outside the World guard.
pub struct SealedCurrentReadyQueryPermission<'basis> {
    security: crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedPermissionSecurityBasis<'basis>,
    authentication_valid_until: std::time::Instant,
    query_name: &'basis str,
    _query_identity: &'basis worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
    _parameter_identity: &'basis worth_foundational::facade::CanonicalDigestId,
    _request: &'basis worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    _product: &'basis crate::basis::WorthQueryProductObservationLease,
    _authorization: &'basis WorthQueryRetainedAuthorizationDecisionFacts,
}

impl SealedCurrentReadyQueryPermission<'_> {
    pub fn validate_request(&self) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        validate_current_request(
            self._request,
            self.authentication_valid_until,
            self.query_name,
        )
    }
    pub fn product(&self) -> &crate::basis::WorthQueryProductObservationLease {
        self._product
    }

    pub fn snapshot_handle(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.security.snapshot_handle()
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
    pub fn validate_current_request(
        &self,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        validate_current_request(
            self.controls.request_scope(),
            self.access.principal().valid_until(),
            self.query.name(),
        )
    }

    pub fn seal_current_ready<'basis>(
        &'basis self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
                &self.basis,
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
        super::super::super::admission_preparation::validate_admission_request(
            self.controls.request_scope(),
            self.query.name(),
        )?;
        let sealed = SealedCurrentReadyQueryPermission {
            security,
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


    pub fn require_current_principal(
        &self,
        control: crate::domain_computation::primary_graph::product_operation::current_read::CurrentReadControl<'_>,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        let principal = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(
            self.access.principal().principal_entity_id(),
        );
        if !control.admits_principal(&principal) {
            return Err(denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
                self.query.name(),
            ));
        }
        Ok(())
    }
}

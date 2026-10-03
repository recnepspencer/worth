use super::*;

/// The exact prepared Query permission after its issued World head was
/// checked. Actor currentness still runs outside the World guard.
pub(in crate::domain_computation::primary_graph) struct SealedCurrentReadyQueryPermission<'basis> {
    security: crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedPermissionSecurityBasis<'basis>,
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        super::super::super::admission_preparation::validate_admission_request(
            self._request,
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
        self.security.snapshot_handle()
    }
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
    /// A moved head makes the old selected basis ineligible for a Clean result.
    /// The caller may continue only through fresh disclosure/reselection.
    pub(in crate::domain_computation::primary_graph) fn seal_current_ready<'basis>(
        &'basis self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Option<SealedCurrentReadyQueryPermission<'basis>>,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        use crate::domain_computation::primary_graph::product_operation::SelectedPermissionSecurityStop;
        let security = self
            .issuer
            .admit_matching_head_query_permission_basis(
                &self.security_product,
                &self.basis,
                admission,
            )
            .map_err(|stop| match stop {
                SelectedPermissionSecurityStop::Admission(stop) => resource_denial(stop),
                SelectedPermissionSecurityStop::AccountingOverflow => work_denial(),
                SelectedPermissionSecurityStop::World(_) => denial(
                    WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                    self.query.name(),
                ),
            })?;
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        super::super::super::admission_preparation::validate_admission_request(
            self.controls.request_scope(),
            self.query.name(),
        )?;
        Ok(security.map(|security| SealedCurrentReadyQueryPermission {
            security,
            query_name: self.query.name(),
            _query_identity: self.query.identity(),
            _parameter_identity: self.parameters.identity(),
            _request: self.controls.request_scope(),
            _product: &self.security_product,
            _authorization: &self.authorization,
        }))
    }
}

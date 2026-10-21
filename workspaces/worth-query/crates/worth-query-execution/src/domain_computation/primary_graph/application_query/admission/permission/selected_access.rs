use worth_relational::facade::mvcc::CompanionPreflightStop;

/// Both typed identities came from their owners on one selected immutable
/// native root and one request. The HRTB Query continuation keeps their
/// issuers, selected operation and local identities alive until read release.
pub(in crate::domain_computation::primary_graph::application_query) struct SelectedIssuedAccessRoot<
    'a,
    Schema,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    pub(super) principal:
        &'a crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
            Schema,
            Principal,
            PrincipalIdentity,
        >,
    pub(super) scope:
        &'a crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
    pub(super) root: &'a worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pub(super) snapshot: &'a worth_relational::facade::snapshots::SnapshotHandle,
    pub(super) product: &'a worth_runtime_world::facade::ProductBranchObservation,
    pub(super) request:
        &'a worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
}

impl<Schema, Principal, PrincipalIdentity, Scope>
    SelectedIssuedAccessRoot<'_, Schema, Principal, PrincipalIdentity, Scope>
{
    pub(in crate::domain_computation::primary_graph::application_query) fn matches(
        &self,
        principal: &crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal<
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        scope: &crate::domain_computation::primary_graph::WorthQueryApplicationEntityIdentity<
            Schema,
            Scope,
        >,
        product: &crate::basis::WorthQueryProductObservationLease,
        security: &worth_relational::facade::snapshots::SnapshotHandle,
        basis: &worth_relational::facade::snapshots::SnapshotHandle,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<bool, CompanionPreflightStop> {
        Ok(std::ptr::eq(self.principal, principal)
            && std::ptr::eq(self.scope, scope)
            && self.request.same_request(request)
            && self.product == product.observation()
            && self.root.admission_identity() == product.relational_basis().admission_identity()
            && self.snapshot == security
            && self.snapshot == basis)
    }
}

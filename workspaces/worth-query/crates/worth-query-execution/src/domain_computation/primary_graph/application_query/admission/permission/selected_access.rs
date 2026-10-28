use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryApplicationQueryAdmissionDenial,
    WorthQueryApplicationQueryAdmissionDenialKind, WorthQueryIssuedSelectedPrincipal,
    WorthQueryIssuedSelectedScope, WorthQuerySelectedProductOperation,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::mvcc::CompanionPreflightStop;

/// Both typed identities came from their owners on one selected immutable
/// native root and one request. The HRTB Query continuation keeps their
/// issuers, selected operation and local identities alive until read release.
pub(in crate::domain_computation::primary_graph) struct SelectedIssuedAccessRoot<
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

impl<'a, Schema, Principal, PrincipalIdentity, Scope>
    SelectedIssuedAccessRoot<'a, Schema, Principal, PrincipalIdentity, Scope>
{
    /// Borrow only the identities actually issued on this current root/request.
    pub(in crate::domain_computation::primary_graph) fn from_issued<'selected, 'runtime>(
        selected: &'selected WorthQuerySelectedProductOperation<'runtime, Schema>,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        principal: &'a WorthQueryIssuedSelectedPrincipal<
            'selected,
            'runtime,
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        scope: &'a WorthQueryIssuedSelectedScope<'selected, 'runtime, Schema, Scope>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        subject: &str,
    ) -> Result<Self, WorthQueryApplicationQueryAdmissionDenial>
    where
        Schema: ApplicationSchema,
    {
        if !std::ptr::eq(principal.selected(), selected)
            || !std::ptr::eq(scope.selected(), selected)
            || !std::ptr::eq(principal.principal(), access.principal())
            || !std::ptr::eq(scope.scope(), access.scope())
            || !principal.request().same_request(request)
            || !scope.request().same_request(request)
            || principal.issued_root().admission_identity()
                != scope.issued_root().admission_identity()
            || !std::ptr::eq(principal.issued_snapshot(), scope.issued_snapshot())
        {
            return Err(super::denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                subject,
            ));
        }
        Ok(Self {
            principal: principal.principal(),
            scope: scope.scope(),
            root: principal.issued_root(),
            snapshot: principal.issued_snapshot(),
            product: selected.product().observation(),
            request: principal.request(),
        })
    }

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

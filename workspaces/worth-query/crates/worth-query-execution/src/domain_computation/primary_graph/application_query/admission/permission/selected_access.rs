use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        admission.charge_external_work(5)?;
        let product_work = u64::try_from(
            worth_runtime_world::facade::CurrentProductHead::comparison_work_bound(self.product),
        )
        .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
        let snapshot_text = self
            .snapshot
            .branch_id()
            .0
            .len()
            .checked_mul(2)
            .and_then(|bytes| bytes.checked_add(security.branch_id().0.len()))
            .and_then(|bytes| bytes.checked_add(basis.branch_id().0.len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        let comparison_work = product_work
            .checked_add(snapshot_text)
            .and_then(|work| work.checked_add(8))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(comparison_work)?;
        Ok(std::ptr::eq(self.principal, principal)
            && std::ptr::eq(self.scope, scope)
            && self.request.same_request(request)
            && self.product == product.observation()
            && self.root.admission_identity() == product.relational_basis().admission_identity()
            && self.snapshot == security
            && self.snapshot == basis)
    }
}

//! The principal and selected proof basis retained by fresh resolution.
use super::*;
/// Principal accepted by the installed decoder on this exact selected root.
/// The selected operation remains borrowed for the whole scoped Query read.
pub(in crate::domain_computation) struct WorthQueryIssuedSelectedPrincipal<
    'selected,
    'runtime,
    Schema,
    Principal,
    PrincipalIdentity,
> {
    pub(super) principal: WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    pub(super) selected:
        &'selected crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            'runtime,
            Schema,
        >,
    pub(super) request: WorthQueryRequestScope,
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryIssuedSelectedPrincipal<'_, '_, Schema, Principal, PrincipalIdentity>
{
    pub(in crate::domain_computation) fn principal(
        &self,
    ) -> &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
        &self.principal
    }

    pub(in crate::domain_computation) fn issued_root(
        &self,
    ) -> &worth_relational::facade::branch::AdmittedRelationalBranchBasis {
        self.selected.product().relational_basis()
    }

    pub(in crate::domain_computation) fn issued_snapshot(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.selected.application_basis().snapshot_handle()
    }

    pub(in crate::domain_computation) fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }

    pub(in crate::domain_computation) fn selected(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, Schema>
    {
        self.selected
    }
}

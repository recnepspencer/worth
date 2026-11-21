//! The principal and selected proof basis retained by fresh resolution.
use super::*;
/// Principal accepted by the installed decoder on this exact selected root.
/// The selected operation remains borrowed for the whole scoped Query read.
pub struct WorthQueryIssuedSelectedPrincipal<
    'selected,
    'runtime,
    Schema,
    Principal,
    PrincipalIdentity,
> {
    pub(crate) principal: WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    pub(crate) selected:
        &'selected crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            'runtime,
            Schema,
        >,
    pub(crate) request: WorthQueryRequestScope,
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryIssuedSelectedPrincipal<'_, '_, Schema, Principal, PrincipalIdentity>
{
    pub fn principal(
        &self,
    ) -> &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
        &self.principal
    }

    pub fn issued_root(
        &self,
    ) -> &worth_relational::facade::branch::AdmittedRelationalBranchBasis {
        self.selected.product().relational_basis()
    }

    pub fn issued_snapshot(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.selected.application_basis().snapshot_handle()
    }

    pub fn request_lapse(
        &self,
    ) -> Option<crate::domain_computation::authorization::WorthQueryAdmissionLapse> {
        crate::domain_computation::authorization::admitted_request::request_lapse(
            &self.request,
            self.principal.valid_until(),
        )
    }

    pub fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }

    pub fn selected(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, Schema>
    {
        self.selected
    }
}

#[path = "issued_principal_court_doors_for_test.rs"]
mod court_doors_for_test;

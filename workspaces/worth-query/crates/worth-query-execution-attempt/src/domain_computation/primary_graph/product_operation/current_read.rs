//! Read-only current-product control, independent of navigation and publication.
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::authorization::{
    admitted_request::AdmittedRequestCurrentness, WorthQueryAdmissionLapse,
    WorthQueryOperationScopeEntityBinding,
};
use crate::domain_computation::primary_graph::WorthQueryAdvancementDenial;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};

#[derive(Debug)]
pub enum CurrentReadDenial {
    HeadDiverged,
    NativeBasis(RelationalBranchBasisDenial),
    HeadAdmission(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
    Stopped(WorthQueryAdvancementDenial),
    Authorization(WorthQueryAdmissionLapse),
    Busy,
}

/// Borrows genuine admission; it grants neither a historical basis nor a write.
#[derive(Clone, Copy)]
pub struct CurrentReadControl<'a> {
    pub current: &'a WorthQueryProductObservationLease,
    pub request: &'a AdmittedRequestCurrentness,
    pub principal: &'a WorthQueryOperationScopeEntityBinding,
}
impl<'a> CurrentReadControl<'a> {
    pub fn new(
        current: &'a WorthQueryProductObservationLease,
        request: &'a AdmittedRequestCurrentness,
        principal: &'a WorthQueryOperationScopeEntityBinding,
    ) -> Self {
        Self {
            current,
            request,
            principal,
        }
    }
    pub fn current(&self) -> &WorthQueryProductObservationLease {
        self.current
    }
    pub fn admits_request(&self, request: &WorthQueryRequestScope) -> bool {
        self.request.same_request(request)
    }
    pub fn admits_principal(&self, principal: &WorthQueryOperationScopeEntityBinding) -> bool {
        self.principal == principal
    }
    pub fn request_lapse(&self) -> Option<WorthQueryAdmissionLapse> {
        self.request.lapse()
    }
    pub fn admitted_request(&self) -> &AdmittedRequestCurrentness {
        self.request
    }
}

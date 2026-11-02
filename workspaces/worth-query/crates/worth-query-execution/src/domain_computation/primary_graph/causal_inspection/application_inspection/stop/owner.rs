//! Preserve exact admission causes through the public inspection denial.
use super::*;
pub use crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernanceDenialKind;
use crate::domain_computation::authorization::{
    application_disclosure::DiagnosticPreparationDenial, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};
use worth_execution::{MapKernelFailure, ScopeStop};

/// The retained owner cause; no protected observation payload is projected.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryApplicationInspectionOwnerRefusal<'a> {
    Authorization(&'a WorthQueryApplicationQueryAdmissionDenial),
    CurrentAuthorization(&'a WorthQueryOperationAuthorizationDenialKind),
    Capability(&'a WorthQueryOperationAuthorizationDenial),
    Scope(&'a WorthQueryEntityResolutionDenial),
    Security(&'a crate::basis::WorthQueryProductBranchAdmissionDenial),
    Governance(&'a WorthQueryApplicationQueryGovernanceDenialKind),
}
type Refusal<'a> = WorthQueryApplicationInspectionOwnerRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    match cause {
        InspectionCause::Query(cause) => Some(Refusal::Authorization(cause)),
        InspectionCause::Scope(cause) => Some(Refusal::Scope(cause)),
        InspectionCause::Selection(cause) => Some(Refusal::Security(cause)),
        InspectionCause::Admission(cause) => admission(cause),
        _ => None,
    }
}

fn admission(cause: &InspectionAdmissionDenial) -> Option<Refusal<'_>> {
    use InspectionAdmissionDenial as Cause;
    match cause {
        Cause::Authorization(cause) => Some(Refusal::Authorization(cause)),
        Cause::CurrentAuthorization(cause) => Some(Refusal::CurrentAuthorization(cause)),
        Cause::Capability(cause) => Some(Refusal::Capability(cause)),
        Cause::Scope(cause) => Some(Refusal::Scope(cause)),
        Cause::Security(cause) => Some(Refusal::Security(cause)),
        Cause::Governance(cause)
        | Cause::DiagnosticPreparation(CaptureStop::Kernel(ScopeStop::Failure(
            MapKernelFailure::Domain(DiagnosticPreparationDenial::Governance(cause)),
        ))) => Some(Refusal::Governance(cause)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;

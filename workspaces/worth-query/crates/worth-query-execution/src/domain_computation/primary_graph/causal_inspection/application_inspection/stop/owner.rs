//! Preserve exact admission causes through the public inspection denial.
use super::*;
pub use crate::domain_computation::authorization::application_disclosure::contract::WorthQueryApplicationDisclosureContractDenial;
pub use crate::domain_computation::authorization::application_disclosure::WorthQueryApplicationQueryGovernanceDenialKind;
use crate::domain_computation::authorization::{
    application_disclosure::DiagnosticPreparationDenial, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind,
};
pub use crate::domain_computation::primary_graph::{
    application_attempt::{
        IndexedSelectionReobserveDenial, OutputFieldCaptureFailure, StoreDenial,
    },
    provider::{OwnEffectOnReads, RebaseVerificationReason},
};
pub use crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSessionStartDenial;
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
    Principal(&'a WorthQueryPrincipalResolutionDenial),
    Installation(
        &'a worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial,
    ),
    DiagnosticContract(&'a WorthQueryApplicationDisclosureContractDenial),
    Graph(&'a worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkAdmissionDenial),
    ReportingInput(&'a worth_runtime_bridge::facade::BridgeQueryObservationReportingInputDenial),
    ForeignTarget,
    ProducerCapture(&'a OutputFieldCaptureFailure),
    Session(&'a WorthQueryManagedGraphWorkSessionStartDenial),
}
type Refusal<'a> = WorthQueryApplicationInspectionOwnerRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    match cause {
        InspectionCause::Query(cause) => Some(Refusal::Authorization(cause)),
        InspectionCause::Scope(cause) => Some(Refusal::Scope(cause)),
        InspectionCause::Selection(cause) => Some(Refusal::Security(cause)),
        InspectionCause::Principal(cause) => Some(Refusal::Principal(cause)),
        InspectionCause::Binding(stop) => match domain(stop)? {
            InspectionBindingDenial::Installation(cause) => Some(Refusal::Installation(cause)),
            InspectionBindingDenial::Storage(_) => None,
        },
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
        Cause::Session(cause) => Some(Refusal::Session(cause)),
        Cause::Connection(stop) => match domain(stop)? {
            super::super::connection::ConnectionRefusal::Capture(cause) => Some(Refusal::ProducerCapture(cause)),
            _ => None,
        },
        Cause::Target(cause) => match cause {
            super::super::target::InspectionTargetDenial::ForeignBinding => Some(Refusal::ForeignTarget),
            super::super::target::InspectionTargetDenial::Storage(_) => None,
        },
        Cause::DiagnosticContract(stop) => match domain(stop)? {
            crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::Contract(cause) => Some(Refusal::DiagnosticContract(cause)),
            _ => None,
        },
        Cause::Plan(stop) => match domain(stop)? {
            super::super::admission::InspectionPlanDenial::Graph(cause) => Some(Refusal::Graph(cause)),
            _ => None,
        },
        Cause::Report(stop) => match domain(stop)? {
            super::super::report::InspectionReportDenial::Input(cause) => Some(Refusal::ReportingInput(cause)),
            _ => None,
        },
        Cause::Governance(cause)
        | Cause::DiagnosticPreparation(CaptureStop::Kernel(ScopeStop::Failure(
            MapKernelFailure::Domain(DiagnosticPreparationDenial::Governance(cause)),
        ))) => Some(Refusal::Governance(cause)),
        _ => None,
    }
}

fn domain<E>(stop: &CaptureStop<E>) -> Option<&E> {
    match stop {
        CaptureStop::Kernel(ScopeStop::Failure(MapKernelFailure::Domain(cause))) => Some(cause),
        _ => None,
    }
}

#[cfg(test)]
mod extended_tests;
#[cfg(test)]
mod tests;

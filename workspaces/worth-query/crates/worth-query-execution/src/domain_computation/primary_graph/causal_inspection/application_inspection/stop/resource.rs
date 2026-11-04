//! Borrow original resource causes through their real owner wrappers.
use super::super::{
    admission::InspectionPlanDenial, identity::InspectionIdentityStop,
    report::InspectionReportDenial, target::InspectionTargetDenial,
};
use super::*;
use crate::domain_computation::{
    primary_graph::admitted_capture::RetainedStorageDenial,
    provider_session::{WorthQueryInspectionReadTerminalDenial, WorthQueryManagedGraphReadDenial},
};
use worth_execution::{LeaseDenial, MapKernelFailure, MapKernelStop, MemoryLimitDenial, ScopeStop};
use worth_foundational::facade::{CanonicalBasisPreparationStop, CanonicalDigestAdmissionStop};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop;

/// A borrowed original stop; reading it creates no replacement work or authority.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryApplicationInspectionResourceRefusal<'a> {
    Request(&'a WorkCeilingDenial),
    Advancement(&'a crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementDenial),
    Kernel(&'a MapKernelStop),
    Native(&'a RelationalExecutionDenialCause),
    Lease(&'a LeaseDenial),
    Memory(&'a MemoryLimitDenial),
    Allocation(&'a std::collections::TryReserveError),
    Array(&'a worth_execution::ExecutionRequestArrayDenial),
}
type Refusal<'a> = WorthQueryApplicationInspectionResourceRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    match cause {
        InspectionCause::Binding(stop) => capture(stop, |cause| match cause {
            InspectionBindingDenial::Storage(cause) => storage(cause),
            InspectionBindingDenial::Installation(_) => None,
        }),
        InspectionCause::Admission(cause) => admission(cause),
        InspectionCause::Advancement(
            cause
            @ crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementDenial::Resource(
                _,
            ),
        ) => Some(Refusal::Advancement(cause)),
        InspectionCause::Unavailable
        | InspectionCause::Selection(_)
        | InspectionCause::Principal(_)
        | InspectionCause::Scope(_)
        | InspectionCause::Query(_)
        | InspectionCause::Advancement(
            WorthQueryAdvancementDenial::Interrupted(_)
            | WorthQueryAdvancementDenial::NestedOpening
            | WorthQueryAdvancementDenial::NestedStopped
            | WorthQueryAdvancementDenial::Panicked,
        ) => None,
    }
}

fn admission(cause: &InspectionAdmissionDenial) -> Option<Refusal<'_>> {
    use InspectionAdmissionDenial as Cause;
    match cause {
        Cause::Request(cause) => Some(Refusal::Request(cause)),
        Cause::Storage(stop) | Cause::Target(InspectionTargetDenial::Storage(stop)) => {
            capture(stop, storage)
        }
        Cause::DiagnosticContract(stop) => capture(stop, |cause| {
            match cause {
            crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::Storage(cause) => storage(cause),
            crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::Contract(_)
            | crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::NotDeclared
            | crate::domain_computation::authorization::application_disclosure::DiagnosticContractDenial::UnsupportedShape => None,
        }
        }),
        Cause::DiagnosticPreparation(stop) => capture(stop, |cause| {
            match cause { crate::domain_computation::authorization::application_disclosure::DiagnosticPreparationDenial::Governance(_) => None }
        }),
        Cause::DiagnosticAdmission(stop) => capture(stop, |cause| {
            match cause { crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial::NotDeclared | crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial::UnsupportedShape | crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial::ForeignBinding | crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial::FieldDenied | crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial::RelationDenied | crate::domain_computation::authorization::application_disclosure::DiagnosticComparisonDenial::InternalInfluenceDenied => None }
        }),
        Cause::ComparisonParameters(stop) => capture(stop, |()| None),
        Cause::Rich(stop) => capture(stop, |cause| {
            match cause { crate::domain_computation::primary_graph::application_query::WorthQueryProjectionMaterializationRefusal::DisclosureInterpretationChanged | crate::domain_computation::primary_graph::application_query::WorthQueryProjectionMaterializationRefusal::OriginalDisclosureIncomplete => None }
        }),
        Cause::RichSource(stop) => capture(stop, |cause| match cause {
            super::super::rich::source::SourceMaterializationDenial::Array(cause) => {
                Some(Refusal::Array(cause))
            }
            super::super::rich::source::SourceMaterializationDenial::UnresolvedField
            | super::super::rich::source::SourceMaterializationDenial::MissingOriginalValue => None,
        }),
        Cause::Connection(stop) => capture(stop, |cause| match cause {
            super::super::connection::ConnectionRefusal::General(_)
            | super::super::connection::ConnectionRefusal::Output(_)
            | super::super::connection::ConnectionRefusal::Capture(_) => None,
        }),
        Cause::Comparison(stop) => capture(stop, |cause| {
            match cause {
            crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial::Storage(cause) => storage(cause),
            crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial::Allocation(cause) => Some(Refusal::Allocation(cause)),
            crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial::DisclosureInterpretationChanged
            | crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial::OriginalDisclosureIncomplete
            | crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial::NestingExceedsDeclaredShape => None,
        }
        }),
        Cause::Plan(stop) => capture(stop, |cause| match cause {
            InspectionPlanDenial::Storage(cause) => storage(cause),
            InspectionPlanDenial::Graph(_) | InspectionPlanDenial::Overflow => None,
        }),
        Cause::Report(stop) => capture(stop, report),
        Cause::Target(InspectionTargetDenial::ForeignBinding) => None,
        Cause::Read(cause) => read(cause),
        Cause::Terminal(cause) => match cause {
            WorthQueryInspectionReadTerminalDenial::Read(cause) => read(cause),
            WorthQueryInspectionReadTerminalDenial::Request(cause) => Some(Refusal::Request(cause)),
        },
        Cause::RequestControl(_)
        | Cause::ForeignIssuedAccess
        | Cause::MissingGraph
        | Cause::MissingInspection
        | Cause::Authorization(_)
        | Cause::CurrentAuthorization(_)
        | Cause::Capability(_)
        | Cause::Scope(_)
        | Cause::Security(_)
        | Cause::Governance(_)
        | Cause::FacetDenied
        | Cause::MissingOriginalProjection
        | Cause::Session(_) => None,
    }
}
fn read(cause: &WorthQueryManagedGraphReadDenial) -> Option<Refusal<'_>> {
    match cause {
        WorthQueryManagedGraphReadDenial::RequestStop(cause) => Some(Refusal::Native(cause)),
        WorthQueryManagedGraphReadDenial::SnapshotUnavailable
        | WorthQueryManagedGraphReadDenial::IndexNotInstalled(_)
        | WorthQueryManagedGraphReadDenial::MutationSession
        | WorthQueryManagedGraphReadDenial::ForeignBasis
        | WorthQueryManagedGraphReadDenial::ForeignGraph
        | WorthQueryManagedGraphReadDenial::ForeignReadProof
        | WorthQueryManagedGraphReadDenial::ForeignReadIntent => None,
    }
}

fn capture<'a, E>(
    stop: &'a CaptureStop<E>,
    domain: impl FnOnce(&'a E) -> Option<Refusal<'a>>,
) -> Option<Refusal<'a>> {
    match stop {
        CaptureStop::Request(cause) => Some(Refusal::Request(cause)),
        CaptureStop::Kernel(ScopeStop::Admission(cause)) => Some(Refusal::Lease(cause)),
        CaptureStop::Kernel(ScopeStop::Failure(cause)) => kernel(cause, domain),
    }
}

fn kernel<'a, E>(
    stop: &'a MapKernelFailure<E>,
    domain: impl FnOnce(&'a E) -> Option<Refusal<'a>>,
) -> Option<Refusal<'a>> {
    match stop {
        MapKernelFailure::Stop(cause) => Some(Refusal::Kernel(cause)),
        MapKernelFailure::Domain(cause) => domain(cause),
        MapKernelFailure::Panic | MapKernelFailure::ResultCapacityExceeded => None,
    }
}

fn storage(cause: &RetainedStorageDenial) -> Option<Refusal<'_>> {
    match cause {
        RetainedStorageDenial::Memory(cause) => Some(Refusal::Memory(cause)),
        RetainedStorageDenial::Overflow => None,
    }
}

fn report(cause: &InspectionReportDenial) -> Option<Refusal<'_>> {
    match cause {
        InspectionReportDenial::Storage(cause) => storage(cause),
        InspectionReportDenial::Identity(cause) => match cause {
            InspectionIdentityStop::Resource(cause) => kernel(cause, storage),
            InspectionIdentityStop::Basis(cause) => basis(cause),
            InspectionIdentityStop::Digest(cause) => digest(cause),
            InspectionIdentityStop::Overflow
            | InspectionIdentityStop::ForeignSession
            | InspectionIdentityStop::ProductRequired => None,
        },
        InspectionReportDenial::Bridge(cause) => match cause {
            BridgeQueryObservationReportingStop::Resource(cause) => kernel(cause, storage),
            BridgeQueryObservationReportingStop::Basis(cause) => basis(cause),
            BridgeQueryObservationReportingStop::Digest(cause) => digest(cause),
            BridgeQueryObservationReportingStop::AccountingOverflow
            | BridgeQueryObservationReportingStop::InvalidReportingAssociation => None,
        },
        InspectionReportDenial::Input(_)
        | InspectionReportDenial::ForeignComparison
        | InspectionReportDenial::ForeignMaterialization => None,
    }
}
fn basis(
    cause: &CanonicalBasisPreparationStop<MapKernelFailure<RetainedStorageDenial>>,
) -> Option<Refusal<'_>> {
    match cause {
        CanonicalBasisPreparationStop::Resource { cause, .. } => kernel(cause, storage),
        CanonicalBasisPreparationStop::Construction(_)
        | CanonicalBasisPreparationStop::UnsupportedSortImplementation(_)
        | CanonicalBasisPreparationStop::AccountingOverflow => None,
    }
}
fn digest(
    cause: &CanonicalDigestAdmissionStop<MapKernelFailure<RetainedStorageDenial>>,
) -> Option<Refusal<'_>> {
    match cause {
        CanonicalDigestAdmissionStop::Resource(cause) => kernel(cause, storage),
        CanonicalDigestAdmissionStop::Derivation(_)
        | CanonicalDigestAdmissionStop::AccountingOverflow
        | CanonicalDigestAdmissionStop::AllocationUnavailable => None,
    }
}

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
    Advancement(&'a crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    Kernel(&'a MapKernelStop),
    Native(&'a RelationalExecutionDenialCause),
    Lease(&'a LeaseDenial),
    Memory(&'a MemoryLimitDenial),
    Allocation(&'a std::collections::TryReserveError),
}
type Refusal<'a> = WorthQueryApplicationInspectionResourceRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    match cause {
        InspectionCause::Binding(stop) => capture(stop, |cause| match cause {
            InspectionBindingDenial::Storage(cause) => storage(cause),
            _ => None,
        }),
        InspectionCause::Admission(cause) => admission(cause),
        InspectionCause::Advancement(
            cause
            @ crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(
                _,
            ),
        ) => Some(Refusal::Advancement(cause)),
        _ => None,
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
            _ => None,
        }
        }),
        Cause::DiagnosticPreparation(stop) => capture(stop, |_| None),
        Cause::DiagnosticAdmission(stop) => capture(stop, |_| None),
        Cause::ComparisonParameters(stop) => capture(stop, |_| None),
        Cause::Connection(stop) => capture(stop, |_| None),
        Cause::Comparison(stop) => capture(stop, |cause| {
            match cause {
            crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial::Storage(cause) => storage(cause),
            crate::domain_computation::primary_graph::application_query::ProjectionComparisonDenial::Allocation(cause) => Some(Refusal::Allocation(cause)),
            _ => None,
        }
        }),
        Cause::Plan(stop) => capture(stop, |cause| match cause {
            InspectionPlanDenial::Storage(cause) => storage(cause),
            _ => None,
        }),
        Cause::Report(stop) => capture(stop, report),
        Cause::Read(WorthQueryManagedGraphReadDenial::RequestStop(cause))
        | Cause::Terminal(WorthQueryInspectionReadTerminalDenial::Read(
            WorthQueryManagedGraphReadDenial::RequestStop(cause),
        )) => Some(Refusal::Native(cause)),
        Cause::Terminal(WorthQueryInspectionReadTerminalDenial::Request(cause)) => {
            Some(Refusal::Request(cause))
        }
        _ => None,
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
            InspectionIdentityStop::Resource(cause)
            | InspectionIdentityStop::Basis(CanonicalBasisPreparationStop::Resource {
                cause,
                ..
            })
            | InspectionIdentityStop::Digest(CanonicalDigestAdmissionStop::Resource(cause)) => {
                kernel(cause, storage)
            }
            _ => None,
        },
        InspectionReportDenial::Bridge(cause) => match cause {
            BridgeQueryObservationReportingStop::Resource(cause)
            | BridgeQueryObservationReportingStop::Basis(
                CanonicalBasisPreparationStop::Resource { cause, .. },
            )
            | BridgeQueryObservationReportingStop::Digest(
                CanonicalDigestAdmissionStop::Resource(cause),
            ) => kernel(cause, storage),
            _ => None,
        },
        _ => None,
    }
}

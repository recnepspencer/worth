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
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop;

/// A borrowed original stop; reading it creates no replacement work or authority.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryApplicationInspectionResourceRefusal<'a> {
    Request(&'a WorkCeilingDenial),
    Kernel(&'a MapKernelStop),
    Lease(&'a LeaseDenial),
    Memory(&'a MemoryLimitDenial),
}
type Refusal<'a> = WorthQueryApplicationInspectionResourceRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    match cause {
        InspectionCause::Binding(stop) => capture(stop, |cause| match cause {
            InspectionBindingDenial::Storage(cause) => storage(cause),
            _ => None,
        }),
        InspectionCause::Admission(cause) => admission(cause),
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
        Cause::Plan(stop) => capture(stop, |cause| match cause {
            InspectionPlanDenial::Storage(cause) => storage(cause),
            _ => None,
        }),
        Cause::Report(stop) => capture(stop, report),
        Cause::Read(WorthQueryManagedGraphReadDenial::RequestStop(cause))
        | Cause::Terminal(WorthQueryInspectionReadTerminalDenial::Read(
            WorthQueryManagedGraphReadDenial::RequestStop(cause),
        )) => Some(Refusal::Kernel(cause)),
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

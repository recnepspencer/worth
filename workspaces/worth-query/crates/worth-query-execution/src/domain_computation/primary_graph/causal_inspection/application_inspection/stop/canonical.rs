//! Canonical owner denials remain distinct from original request resources.
use super::super::{identity::InspectionIdentityStop, report::InspectionReportDenial};
use super::*;
use worth_execution::{MapKernelFailure, ScopeStop};
use worth_foundational::facade::{
    CanonicalBasisConstructionDenial, CanonicalBasisPreparationStop, CanonicalDigestAdmissionStop,
    CanonicalDigestDerivationDenial,
};
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop;

/// Non-resource canonical refusal borrowed from the actual Basis/Digest owner.
#[derive(Clone, Copy, Debug)]
pub enum WorthQueryApplicationInspectionCanonicalRefusal<'a> {
    Basis(&'a CanonicalBasisConstructionDenial),
    Digest(&'a CanonicalDigestDerivationDenial),
    UnsupportedSortImplementation,
    AccountingOverflow,
    AllocationUnavailable,
}
type Refusal<'a> = WorthQueryApplicationInspectionCanonicalRefusal<'a>;

pub(super) fn cause(cause: &InspectionCause) -> Option<Refusal<'_>> {
    let InspectionCause::Admission(InspectionAdmissionDenial::Report(CaptureStop::Kernel(
        ScopeStop::Failure(MapKernelFailure::Domain(cause)),
    ))) = cause
    else {
        return None;
    };
    match cause {
        InspectionReportDenial::Identity(InspectionIdentityStop::Basis(cause))
        | InspectionReportDenial::Bridge(BridgeQueryObservationReportingStop::Basis(cause)) => {
            basis(cause)
        }
        InspectionReportDenial::Identity(InspectionIdentityStop::Digest(cause))
        | InspectionReportDenial::Bridge(BridgeQueryObservationReportingStop::Digest(cause)) => {
            digest(cause)
        }
        _ => None,
    }
}

fn basis<S>(cause: &CanonicalBasisPreparationStop<S>) -> Option<Refusal<'_>> {
    match cause {
        CanonicalBasisPreparationStop::Construction(cause) => Some(Refusal::Basis(cause)),
        CanonicalBasisPreparationStop::UnsupportedSortImplementation => {
            Some(Refusal::UnsupportedSortImplementation)
        }
        CanonicalBasisPreparationStop::AccountingOverflow => Some(Refusal::AccountingOverflow),
        CanonicalBasisPreparationStop::Resource { .. } => None,
    }
}

fn digest<S>(cause: &CanonicalDigestAdmissionStop<S>) -> Option<Refusal<'_>> {
    match cause {
        CanonicalDigestAdmissionStop::Derivation(cause) => Some(Refusal::Digest(cause)),
        CanonicalDigestAdmissionStop::AccountingOverflow => Some(Refusal::AccountingOverflow),
        CanonicalDigestAdmissionStop::AllocationUnavailable => Some(Refusal::AllocationUnavailable),
        CanonicalDigestAdmissionStop::Resource(_) => None,
    }
}

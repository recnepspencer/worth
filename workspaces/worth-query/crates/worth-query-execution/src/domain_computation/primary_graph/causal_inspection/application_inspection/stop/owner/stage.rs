//! Exact non-resource stage diagnostics, leaving canonical/resource payloads with their readers.
use super::super::super::{identity::InspectionIdentityStop, report::InspectionReportDenial};
use super::*;
use crate::domain_computation::vocabulary::primary_graph::admitted_capture::CaptureStop;
use crate::domain_computation::vocabulary::primary_graph::retained_stage::RetainedStorageDenial;
use worth_foundational::facade::{CanonicalBasisPreparationStop, CanonicalDigestAdmissionStop};
use worth_runtime_bridge::facade::BridgeQueryObservationReportingStop;

pub(super) fn capture<'a, E>(
    owner_stage: Stage,
    stop: &'a CaptureStop<E>,
    project: impl FnOnce(&'a E) -> Option<Refusal<'a>>,
) -> Option<Refusal<'a>> {
    match stop {
        CaptureStop::Kernel(ScopeStop::Failure(cause)) => kernel(owner_stage, cause, project),
        // Exact request/lease causes are exposed by resource_refusal.
        CaptureStop::Request(_) | CaptureStop::Kernel(ScopeStop::Admission(_)) => None,
    }
}
fn kernel<'a, E>(
    owner_stage: Stage,
    cause: &'a MapKernelFailure<E>,
    project: impl FnOnce(&'a E) -> Option<Refusal<'a>>,
) -> Option<Refusal<'a>> {
    match cause {
        MapKernelFailure::Domain(cause) => project(cause),
        MapKernelFailure::Panic => Some(Refusal::KernelPanic(owner_stage)),
        MapKernelFailure::ResultCapacityExceeded => {
            Some(Refusal::KernelResultCapacityExceeded(owner_stage))
        }
        MapKernelFailure::Stop(_) => None, // exact kernel resource reader.
    }
}
pub(super) fn storage(owner_stage: Stage, cause: &RetainedStorageDenial) -> Option<Refusal<'_>> {
    match cause {
        RetainedStorageDenial::Overflow => Some(Refusal::StorageAccountingOverflow(owner_stage)),
        RetainedStorageDenial::Memory(_) => None, // exact memory resource reader.
    }
}
pub(super) fn report(cause: &InspectionReportDenial) -> Option<Refusal<'_>> {
    match cause {
        InspectionReportDenial::Input(cause) => Some(Refusal::ReportingInput(cause)),
        InspectionReportDenial::ForeignComparison => Some(Refusal::ReportForeignComparison),
        InspectionReportDenial::ForeignMaterialization => {
            Some(Refusal::ReportForeignMaterialization)
        }
        InspectionReportDenial::Storage(cause) => storage(Stage::Report, cause),
        InspectionReportDenial::Identity(cause) => match cause {
            InspectionIdentityStop::Overflow => Some(Refusal::ReportIdentityOverflow),
            InspectionIdentityStop::ForeignSession => Some(Refusal::ReportIdentityForeignSession),
            InspectionIdentityStop::ProductRequired => Some(Refusal::ReportIdentityProductRequired),
            InspectionIdentityStop::Resource(cause) => {
                kernel(Stage::ReportIdentity, cause, |cause| {
                    storage(Stage::ReportIdentity, cause)
                })
            }
            InspectionIdentityStop::Basis(cause) => basis(Stage::ReportIdentity, cause),
            InspectionIdentityStop::Digest(cause) => digest(Stage::ReportIdentity, cause),
        },
        InspectionReportDenial::Bridge(cause) => match cause {
            BridgeQueryObservationReportingStop::AccountingOverflow => {
                Some(Refusal::ReportBridgeAccountingOverflow)
            }
            BridgeQueryObservationReportingStop::InvalidReportingAssociation => {
                Some(Refusal::ReportBridgeInvalidReportingAssociation)
            }
            BridgeQueryObservationReportingStop::Resource(cause) => {
                kernel(Stage::ReportBridge, cause, |cause| {
                    storage(Stage::ReportBridge, cause)
                })
            }
            BridgeQueryObservationReportingStop::Basis(cause) => basis(Stage::ReportBridge, cause),
            BridgeQueryObservationReportingStop::Digest(cause) => {
                digest(Stage::ReportBridge, cause)
            }
        },
    }
}
fn basis(
    owner_stage: Stage,
    cause: &CanonicalBasisPreparationStop<MapKernelFailure<RetainedStorageDenial>>,
) -> Option<Refusal<'_>> {
    match cause {
        CanonicalBasisPreparationStop::Resource { cause, .. } => {
            kernel(owner_stage, cause, |cause| storage(owner_stage, cause))
        }
        // Exact immutable canonical causes are already exposed by canonical_refusal.
        CanonicalBasisPreparationStop::Construction(_)
        | CanonicalBasisPreparationStop::UnsupportedSortImplementation(_)
        | CanonicalBasisPreparationStop::AccountingOverflow => None,
    }
}
fn digest(
    owner_stage: Stage,
    cause: &CanonicalDigestAdmissionStop<MapKernelFailure<RetainedStorageDenial>>,
) -> Option<Refusal<'_>> {
    match cause {
        CanonicalDigestAdmissionStop::Resource(cause) => {
            kernel(owner_stage, cause, |cause| storage(owner_stage, cause))
        }
        CanonicalDigestAdmissionStop::Derivation(_)
        | CanonicalDigestAdmissionStop::AccountingOverflow
        | CanonicalDigestAdmissionStop::AllocationUnavailable => None,
    }
}

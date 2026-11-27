//! Audit inspections: source provenance of an authored operation and the
//! records referencing one record.
//!
//! An inspection stop is read in the owner's order: a request interruption,
//! then the exact resource stop, then the owner's own cause, and only then
//! the coarse stop kind. Cancellations, deadlines, budgets and storage
//! faults therefore never render as a permission or capacity refusal.
use worth_execution::ExecutionRequestArrayDenial as Array;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
use worth_query_execution::facade::application_inspection::{
    WorthQueryApplicationInspectionDenial as Inspection,
    WorthQueryApplicationInspectionResourceRefusal as Resource,
    WorthQueryApplicationInspectionStopKind as Stop,
};

use crate::audit::{WorthQueryReferencedByDenial, WorthQuerySourceProvenanceDenial};

use super::correction::OTHER_VERSION;
use super::inspection_owner::{owner, EVIDENCE_WITHHELD, SOURCE_UNAVAILABLE};
use super::request::{BUSY, CANCELLED, CAPACITY, INTERNAL, TIMED_OUT};
use super::stops::{kernel, native, work};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

pub(super) const FIELD_DETAIL_UNSUPPORTED: Summary = Summary::with_message(
    Category::Unsupported,
    "Field-by-field detail isn't available for this kind of change.",
);

impl WorthQueryUserFacingRefusal for WorthQuerySourceProvenanceDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ForeignBranch => OTHER_VERSION,
            Self::Admission(cause) => cause.user_summary(),
            Self::Inspection(_) if self.is_unsupported_field_shape() => FIELD_DETAIL_UNSUPPORTED,
            Self::Inspection(cause) => inspection(cause),
        }
    }
}

/// The inspection's own stop, most exact reader first.
fn inspection(cause: &Inspection) -> Summary {
    if let Some(interruption) = cause.request_interruption() {
        return match interruption {
            Interruption::Cancelled => CANCELLED,
            Interruption::DeadlineExceeded => TIMED_OUT,
        };
    }
    if let Some(stop) = cause.resource_refusal() {
        return resource(stop);
    }
    if let Some(cause) = cause.owner_refusal() {
        return owner(cause);
    }
    match cause.kind() {
        Stop::StaleIncarnation => BUSY,
        Stop::Unavailable => SOURCE_UNAVAILABLE,
        Stop::Request => CAPACITY,
        Stop::Admission => EVIDENCE_WITHHELD,
    }
}

/// The exact original resource stop of any inspection stage.
pub(super) fn resource(stop: Resource<'_>) -> Summary {
    match stop {
        Resource::Request(cause) | Resource::Array(Array::Request(cause)) => work(cause),
        Resource::Advancement(cause) => cause.user_summary(),
        Resource::Kernel(stop) => kernel(stop),
        Resource::Native(cause) => native(cause),
        Resource::Array(Array::Allocation(_))
        | Resource::Lease(_)
        | Resource::Memory(_)
        | Resource::Allocation(_) => CAPACITY,
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryReferencedByDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Query(cause) => cause.user_summary(),
            // An uninstalled or non-reverse query is a declaration defect.
            Self::Installation(_) | Self::NotAnIncomingReference { .. } => INTERNAL,
        }
    }
}

#[cfg(test)]
#[path = "audit_inspection_tests.rs"]
mod tests;

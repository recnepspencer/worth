//! Audit inspections: source provenance of an authored operation and the
//! records referencing one record.
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption as Interruption;
use worth_query_execution::facade::application_inspection::{
    WorthQueryApplicationInspectionDenial as Inspection,
    WorthQueryApplicationInspectionStopKind as Stop,
};

use crate::audit::{WorthQueryReferencedByDenial, WorthQuerySourceProvenanceDenial};

use super::correction::OTHER_VERSION;
use super::request::{BUSY, CANCELLED, CAPACITY, INTERNAL, TIMED_OUT};
use super::stops::work;
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const SOURCE_UNAVAILABLE: Summary = Summary::with_message(
    Category::NotFound,
    "The record of how this was changed is no longer available.",
);
const FIELD_DETAIL_UNSUPPORTED: Summary = Summary::with_message(
    Category::Unsupported,
    "Field-by-field detail isn't available for this kind of change.",
);
const EVIDENCE_WITHHELD: Summary = Summary::with_message(
    Category::PermissionDenied,
    "You don't have permission to see how this was changed.",
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

/// The inspection's own stop: an interruption or resource stop first, then
/// its coarse kind. Disclosure refusals never name what was withheld.
fn inspection(cause: &Inspection) -> Summary {
    if let Some(interruption) = cause.request_interruption() {
        return match interruption {
            Interruption::Cancelled => CANCELLED,
            Interruption::DeadlineExceeded => TIMED_OUT,
        };
    }
    if let Some(stop) = cause.execution_stop() {
        return work(stop);
    }
    match cause.kind() {
        Stop::StaleIncarnation => BUSY,
        Stop::Unavailable => SOURCE_UNAVAILABLE,
        Stop::Request => CAPACITY,
        Stop::Admission => EVIDENCE_WITHHELD,
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

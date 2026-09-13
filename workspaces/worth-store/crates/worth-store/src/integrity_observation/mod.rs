//! Descriptive export of managed physical-integrity observations.
//!
//! This diagnostic protocol adapter consumes Store's bounded scrub facade.
//! It neither serializes authority nor participates in physical admission.
mod runtime_report;

pub use runtime_report::{
    PhysicalIntegrityRuntimeReportContext, PhysicalIntegrityRuntimeReportDenial,
};

//! Typed boundary denials and policy-selected operator diagnostics.
//!
//! Lower a boundary error through [`Reportable::to_operator_report`] and pass
//! the resulting value to a [`DiagnosticSink`] selected by the boundary owner.

mod code;
mod disposition;
mod json;
mod operator_report;
mod sink;

pub(crate) use crate::ChargedBytes;
pub use code::{InvalidReportCode, ReportCode};
pub use disposition::{DenialClass, Disposition, RetryHint};
pub use operator_report::{
    DiagnosticDetail, OperatorReport, ReportDetail, ReportFrame, Reportable,
};
pub use sink::{DiagnosticSink, NoopDiagnosticSink, RecordingDiagnosticSink};

#[cfg(test)]
mod tests;

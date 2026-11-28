use core::error::Error;

use super::{DenialClass, Disposition, ReportCode};

/// Disclosure level selected by the boundary's audience policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DiagnosticDetail {
    /// Classification, disposition, code, and source depth with a fixed summary.
    #[default]
    Redacted,
    /// Include the boundary owner's safe static summary.
    Standard,
    /// Include the safe summary and any expressly selected forensic text.
    Forensic,
}

/// Forensic text or an explicit reason for its omission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReportDetail {
    Withheld,
    Unavailable,
    Disclosed(String),
}

/// One source of a boundary error, in immediate-cause to root-cause order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportFrame {
    pub detail: ReportDetail,
}

/// A diagnostic snapshot with value equality and no live custody or trace metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OperatorReport {
    pub denial_class: DenialClass,
    pub disposition: Disposition,
    pub report_code: ReportCode,
    pub summary: &'static str,
    pub detail: ReportDetail,
    pub chain: Vec<ReportFrame>,
}

/// A boundary error with an owned classification and explicit diagnostic projection.
///
/// ```
/// use std::{error::Error, fmt};
/// use worth_execution::report::{
///     DenialClass, DiagnosticDetail, DiagnosticSink, Disposition,
///     RecordingDiagnosticSink, ReportCode, Reportable, RetryHint,
/// };
///
/// #[derive(Debug)]
/// struct StaleBasis;
///
/// impl fmt::Display for StaleBasis {
///     fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
///         formatter.write_str("The write basis is stale.")
///     }
/// }
///
/// impl Error for StaleBasis {}
///
/// impl Reportable for StaleBasis {
///     fn denial_class(&self) -> DenialClass { DenialClass::Conflict }
///     fn disposition(&self) -> Disposition { Disposition::Retry(RetryHint::AfterBasisAdvance) }
///     fn report_code(&self) -> ReportCode { ReportCode::new("example.stale_basis").unwrap() }
///     fn diagnostic_summary(&self) -> &'static str { "The write basis is stale." }
/// }
///
/// let denial: &dyn Reportable = &StaleBasis;
/// let report = denial.to_operator_report(DiagnosticDetail::Standard);
/// let sink = RecordingDiagnosticSink::default();
/// sink.emit(&report);
/// assert_eq!(sink.take_reports(), vec![report]);
/// ```
pub trait Reportable: Error + Send + Sync + 'static {
    fn denial_class(&self) -> DenialClass;
    fn disposition(&self) -> Disposition;
    fn report_code(&self) -> ReportCode;

    /// Static wording approved for the standard audience.
    fn diagnostic_summary(&self) -> &'static str {
        "Operation denied."
    }

    /// Deterministic, policy-approved detail about this denial.
    /// Omit credentials and per-occurrence correlation or trace metadata.
    fn forensic_detail(&self) -> Option<String> {
        None
    }

    /// Selects policy-approved detail for a member of this error's source chain.
    /// Omit credentials and per-occurrence correlation or trace metadata.
    fn forensic_source_detail(&self, _source: &(dyn Error + 'static)) -> Option<String> {
        None
    }

    /// Projects the denial and each member of its finite [`Error::source`] chain.
    /// Select `detail` using the receiving audience's disclosure policy. The
    /// default projection only calls forensic hooks at `Forensic` detail.
    fn to_operator_report(&self, detail: DiagnosticDetail) -> OperatorReport {
        let summary = match detail {
            DiagnosticDetail::Redacted => "Operation denied.",
            DiagnosticDetail::Standard | DiagnosticDetail::Forensic => self.diagnostic_summary(),
        };
        let mut chain = Vec::new();
        let mut cause = self.source();
        while let Some(source) = cause {
            chain.push(ReportFrame {
                detail: disclose(detail, || self.forensic_source_detail(source)),
            });
            cause = source.source();
        }

        OperatorReport {
            denial_class: self.denial_class(),
            disposition: self.disposition(),
            report_code: self.report_code(),
            summary,
            detail: disclose(detail, || self.forensic_detail()),
            chain,
        }
    }
}

fn disclose(level: DiagnosticDetail, detail: impl FnOnce() -> Option<String>) -> ReportDetail {
    match level {
        DiagnosticDetail::Redacted | DiagnosticDetail::Standard => ReportDetail::Withheld,
        DiagnosticDetail::Forensic => {
            detail().map_or(ReportDetail::Unavailable, ReportDetail::Disclosed)
        }
    }
}

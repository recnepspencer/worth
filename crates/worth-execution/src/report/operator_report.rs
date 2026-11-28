use core::error::Error;
use std::collections::BTreeSet;

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
    detail: ReportDetail,
}

impl ReportFrame {
    pub fn detail(&self) -> &ReportDetail {
        &self.detail
    }
}

/// Whether the source chain was captured or omitted after detecting a cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportChainEnd {
    Complete,
    Cycle,
}

/// A diagnostic snapshot with value equality and no live custody or trace metadata.
///
/// ```compile_fail,E0616
/// use worth_execution::report::OperatorReport;
/// fn erase_sources(report: &mut OperatorReport) {
///     report.chain.clear();
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperatorReport {
    denial_class: DenialClass,
    disposition: Disposition,
    report_code: ReportCode,
    summary: &'static str,
    detail: ReportDetail,
    chain: Vec<ReportFrame>,
    chain_end: ReportChainEnd,
}

impl OperatorReport {
    pub fn denial_class(&self) -> DenialClass {
        self.denial_class
    }

    pub fn disposition(&self) -> Disposition {
        self.disposition
    }

    pub fn report_code(&self) -> ReportCode {
        self.report_code
    }

    pub fn summary(&self) -> &'static str {
        self.summary
    }

    pub fn detail(&self) -> &ReportDetail {
        &self.detail
    }

    pub fn chain(&self) -> &[ReportFrame] {
        &self.chain
    }

    pub fn chain_end(&self) -> ReportChainEnd {
        self.chain_end
    }
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
pub trait Reportable: Error + Send + Sync + ReportProjection + 'static {
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
}

/// The canonical projection that every [`Reportable`] inherits.
///
/// Import this trait to project a concrete denial. A `&dyn Reportable` already
/// exposes this method. Every reportable type has the same projection, with
/// classification and audience-safe wording supplied by its owner's hooks.
///
/// ```compile_fail,E0119
/// use std::{error::Error, fmt};
/// use worth_execution::report::{
///     DenialClass, DiagnosticDetail, Disposition, OperatorReport, ReportCode,
///     ReportProjection, Reportable,
/// };
/// #[derive(Debug)]
/// struct CustomProjection;
/// impl fmt::Display for CustomProjection {
///     fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
///         formatter.write_str("Denied.")
///     }
/// }
/// impl Error for CustomProjection {}
/// impl Reportable for CustomProjection {
///     fn denial_class(&self) -> DenialClass { DenialClass::Defect }
///     fn disposition(&self) -> Disposition { Disposition::Escalate }
///     fn report_code(&self) -> ReportCode { ReportCode::new("example.defect").unwrap() }
/// }
/// impl ReportProjection for CustomProjection {
///     fn to_operator_report(&self, _: DiagnosticDetail) -> OperatorReport {
///         unreachable!()
///     }
/// }
/// ```
pub trait ReportProjection {
    /// Select `detail` using the receiving audience's disclosure policy.
    /// Cyclic source chains are omitted with a typed reason.
    fn to_operator_report(&self, detail: DiagnosticDetail) -> OperatorReport;
}

impl<T: Reportable> ReportProjection for T {
    fn to_operator_report(&self, detail: DiagnosticDetail) -> OperatorReport {
        project(self, detail)
    }
}

fn project(error: &dyn Reportable, detail: DiagnosticDetail) -> OperatorReport {
    let summary = match detail {
        DiagnosticDetail::Redacted => "Operation denied.",
        DiagnosticDetail::Standard | DiagnosticDetail::Forensic => error.diagnostic_summary(),
    };
    let (chain, chain_end) = project_chain(error, detail);

    OperatorReport {
        denial_class: error.denial_class(),
        disposition: error.disposition(),
        report_code: error.report_code(),
        summary,
        detail: disclose(detail, || error.forensic_detail()),
        chain,
        chain_end,
    }
}

fn project_chain(
    error: &dyn Reportable,
    detail: DiagnosticDetail,
) -> (Vec<ReportFrame>, ReportChainEnd) {
    let mut seen = BTreeSet::new();
    let mut sources: Vec<&dyn Error> = Vec::new();
    let mut cause = error.source();
    while let Some(source) = cause {
        if !seen.insert(core::ptr::from_ref(source)) {
            // An error can have several vtables. Omit the malformed chain
            // instead of encoding where one particular vtable first repeated.
            return (Vec::new(), ReportChainEnd::Cycle);
        }
        sources.push(source);
        cause = source.source();
    }
    let chain = sources
        .into_iter()
        .map(|source| ReportFrame {
            detail: disclose(detail, || error.forensic_source_detail(source)),
        })
        .collect();
    (chain, ReportChainEnd::Complete)
}

fn disclose(level: DiagnosticDetail, detail: impl FnOnce() -> Option<String>) -> ReportDetail {
    match level {
        DiagnosticDetail::Redacted | DiagnosticDetail::Standard => ReportDetail::Withheld,
        DiagnosticDetail::Forensic => {
            detail().map_or(ReportDetail::Unavailable, ReportDetail::Disclosed)
        }
    }
}

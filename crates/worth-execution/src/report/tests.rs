use core::{error::Error, fmt};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Barrier,
    },
    thread,
};

use super::*;

const CODE: ReportCode = match ReportCode::new("fixture.write_conflict") {
    Ok(code) => code,
    Err(_) => panic!("fixture report code must be valid"),
};
const FORENSIC_DETAIL: &str = "Sensitive diagnostic: \"private/β\"\\\n\r\t\u{08}\u{0c}\0\u{1f}";

#[derive(Debug)]
struct Denial {
    trace_id: usize,
    cause: Option<Box<Denial>>,
    cycle_visits: Option<AtomicUsize>,
}

impl Denial {
    fn with_depth(depth: usize, trace_id: usize) -> Self {
        let mut cause = None;
        for _ in 0..depth {
            cause = Some(Box::new(Self {
                trace_id,
                cause,
                cycle_visits: None,
            }));
        }
        Self {
            trace_id,
            cause,
            cycle_visits: None,
        }
    }
}

impl fmt::Display for Denial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{FORENSIC_DETAIL} (trace {})", self.trace_id)
    }
}

impl Error for Denial {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Some(visits) = &self.cycle_visits {
            assert!(
                visits.fetch_add(1, Ordering::Relaxed) < 2,
                "cyclic source revisited"
            );
            return Some(self);
        }
        self.cause.as_deref().map(|cause| cause as &dyn Error)
    }
}

impl Reportable for Denial {
    fn denial_class(&self) -> DenialClass {
        DenialClass::Conflict
    }

    fn disposition(&self) -> Disposition {
        Disposition::Retry(RetryHint::AfterBasisAdvance)
    }

    fn report_code(&self) -> ReportCode {
        CODE
    }

    fn diagnostic_summary(&self) -> &'static str {
        "Write conflicted."
    }

    fn forensic_detail(&self) -> Option<String> {
        Some(FORENSIC_DETAIL.to_owned())
    }

    fn forensic_source_detail(&self, source: &(dyn Error + 'static)) -> Option<String> {
        let cause = source.downcast_ref::<Denial>()?;
        cause.cause.as_ref()?;
        Some("A prior write conflicted.".to_owned())
    }
}

#[test]
fn report_codes_reject_invalid_identities() {
    for invalid in [
        "",
        "7conflict",
        ".conflict",
        "Conflict",
        "x y",
        "x\n",
        "x.β",
    ] {
        assert_eq!(ReportCode::new(invalid), Err(InvalidReportCode));
    }
    let valid = "runtime.v2.write-conflict";
    assert_eq!(ReportCode::new(valid).unwrap().to_string(), valid);
}

#[test]
fn disclosure_projects_only_approved_text_and_preserves_omissions() {
    let denial = Denial::with_depth(2, 41);
    let reportable: &dyn Reportable = &denial;
    let redacted = reportable.to_operator_report(DiagnosticDetail::default());
    assert_eq!(redacted.summary(), "Operation denied.");
    assert_eq!(redacted.detail(), &ReportDetail::Withheld);
    assert_eq!(redacted.chain().len(), 2);
    assert!(redacted
        .chain()
        .iter()
        .all(|frame| frame.detail() == &ReportDetail::Withheld));
    for rendering in [redacted.to_canonical_json(), format!("{redacted:?}")] {
        assert!(!rendering.contains("Sensitive diagnostic"));
        assert!(!rendering.contains("trace 41"));
    }

    let standard = reportable.to_operator_report(DiagnosticDetail::Standard);
    assert_eq!(standard.summary(), "Write conflicted.");
    assert_eq!(standard.detail(), &ReportDetail::Withheld);
    assert_eq!(standard.chain(), redacted.chain());

    let forensic = reportable.to_operator_report(DiagnosticDetail::Forensic);
    assert_eq!(forensic.summary(), standard.summary());
    assert_eq!(
        forensic.detail(),
        &ReportDetail::Disclosed(FORENSIC_DETAIL.to_owned())
    );
    assert_eq!(
        forensic.chain()[0].detail(),
        &ReportDetail::Disclosed("A prior write conflicted.".to_owned())
    );
    assert_eq!(forensic.chain()[1].detail(), &ReportDetail::Unavailable);
    let json = forensic.to_canonical_json();
    assert!(!json.contains("trace 41"));
    let wire: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(wire["detail"]["message"], FORENSIC_DETAIL);
    assert_eq!(
        wire["chain"][0]["detail"]["message"],
        "A prior write conflicted."
    );
    assert_eq!(wire["chain"][1]["detail"]["kind"], "unavailable");
}

#[test]
fn independent_threads_produce_equal_reports_and_canonical_bytes_at_each_depth() {
    let barrier = Barrier::new(4);
    let runs = thread::scope(|scope| {
        let handles: Vec<_> = (0..4)
            .map(|trace_id| {
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    (1..=5)
                        .map(|depth| {
                            let denial = Denial::with_depth(depth, trace_id);
                            let report = denial.to_operator_report(DiagnosticDetail::Forensic);
                            let mut source_depth = 0;
                            let mut current: &dyn Error = &denial;
                            while let Some(source) = current.source() {
                                source_depth += 1;
                                current = source;
                            }
                            assert_eq!(source_depth, depth);
                            assert_eq!(report.chain().len(), source_depth);
                            assert_eq!(report.chain_end(), ReportChainEnd::Complete);
                            let json = report.to_canonical_json();
                            (report, json)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    for run in &runs[1..] {
        assert_eq!(run, &runs[0]);
    }
    for (index, (_, json)) in runs[0].iter().enumerate() {
        let wire: serde_json::Value = serde_json::from_str(json).unwrap();
        assert_eq!(wire["chain"].as_array().unwrap().len(), index + 1);
    }
    assert_eq!(
        runs[0][0].1,
        concat!(
            r#"{"schema":"worth.operator_report.v1","denial_class":"conflict","disposition":{"kind":"retry","hint":"after_basis_advance"},"#,
            r#""report_code":"fixture.write_conflict","summary":"Write conflicted.","detail":{"kind":"disclosed","#,
            r#""message":"Sensitive diagnostic: \"private/β\"\\\n\r\t\b\f\u0000\u001f"},"chain":[{"detail":{"kind":"unavailable"}}],"chain_end":"complete"}"#,
        ),
    );
}

#[test]
fn cyclic_chain_preserves_unique_sources_and_records_the_omission() {
    let mut denial = Denial::with_depth(2, 41);
    denial
        .cause
        .as_mut()
        .unwrap()
        .cause
        .as_mut()
        .unwrap()
        .cycle_visits = Some(AtomicUsize::new(0));

    let report = denial.to_operator_report(DiagnosticDetail::Redacted);
    assert_eq!(report.chain().len(), 2);
    assert_eq!(report.chain_end(), ReportChainEnd::Cycle);
    let wire: serde_json::Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(wire["chain_end"], "cycle");
}

#[test]
fn concurrent_sink_emissions_are_retained_exactly_once() {
    fn requires_report_value<T: Clone + fmt::Debug + Eq + Send + Sync + 'static>() {}
    requires_report_value::<OperatorReport>();

    let recorder = RecordingDiagnosticSink::default();
    let sink: &dyn DiagnosticSink = &recorder;
    let reports: Vec<_> = (0..4)
        .map(|depth| Denial::with_depth(depth, 0).to_operator_report(DiagnosticDetail::Redacted))
        .collect();
    thread::scope(|scope| {
        for report in &reports {
            scope.spawn(move || sink.emit(report));
        }
    });
    let mut captured = recorder.take_reports();
    captured.sort_by_key(|report| report.chain().len());
    assert_eq!(captured, reports);
    assert!(recorder.take_reports().is_empty());
    NoopDiagnosticSink.emit(&reports[0]);
}

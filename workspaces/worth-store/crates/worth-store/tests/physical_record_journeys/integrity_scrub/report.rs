use super::*;
use worth_store::integrity_observation::{
    PhysicalIntegrityRuntimeReportContext as Context,
    PhysicalIntegrityRuntimeReportDenial as ReportDenial,
};

fn capture(
    handle: &mut worth_store::physical_runtime::ManagedPhysicalIntegrityScrubHandle,
    run: &str,
) -> String {
    // Output storage belongs to this caller, not to the Store's source budget.
    let mut bytes = Vec::new();
    let count = handle
        .write_observation_report(
            Context::new(run, "runtime-report").unwrap(),
            16_384,
            &mut bytes,
        )
        .unwrap();
    assert_eq!(count, bytes.len() as u64);
    String::from_utf8(bytes).unwrap()
}

#[test]
fn runtime_report_uses_actual_source_and_effect_counters_without_mutation() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let path = parent.path().join("families/records/root-current.selector");
    let before = std::fs::read(&path).unwrap();
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let wire = capture(&mut handle, "actual-read");
    let report: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(report["protocol"], "store.physical.integrity-observation");
    assert_eq!(report["role"], "runtime-integrity-observer");
    assert_eq!(report["process"], std::process::id().to_string());
    assert_eq!(report["consumed"]["report_bytes"], wire.len());
    assert_eq!(report["consumed"]["bytes"], before.len());
    assert_eq!(report["consumed"]["validator_entries"], 1);
    assert_eq!(report["consumed"]["owner_decoder_entries"], 0);
    assert_eq!(
        report["artifacts"][0]["path"],
        "families/records/root-current.selector"
    );
    assert_eq!(
        report["artifacts"][0]["identity"],
        "selector:0000000000000001"
    );
    assert_eq!(report["artifacts"][0]["outcome"]["posture"], "intact");
    assert_eq!(report["completeness"], "complete");
    assert_eq!(std::fs::read(path).unwrap(), before);
    assert_released(&serving);
    serving.close();
}

#[test]
fn report_bounds_and_pause_do_not_hide_extra_acquisition_or_retry() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    assert!(matches!(
        handle.write_observation_report(
            Context::new("bounds", "runtime-report").unwrap(),
            1,
            &mut Vec::new()
        ),
        Err(ReportDenial::ReportBoundExceeded)
    ));
    assert_eq!(handle.counters().acquired_bytes, 0);
    let pause = handle.pause();
    let wire = capture(&mut handle, "paused");
    let report: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(report["completeness"], "indeterminate");
    assert_eq!(report["artifacts"].as_array().unwrap().len(), 0);
    assert_eq!(report["consumed"]["bytes"], 0);
    handle.resume(pause).unwrap();
    assert!(matches!(handle.next_window(), Progress::WindowInspected(_)));
    assert_eq!(handle.counters().completed_windows, 1);
    serving.close();
}

#[test]
fn partial_runtime_report_preserves_observed_range_without_a_damage_claim() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    std::fs::OpenOptions::new()
        .write(true)
        .open(parent.path().join("families/records/root-current.selector"))
        .unwrap()
        .set_len(16)
        .unwrap();
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    let wire = capture(&mut handle, "partial");
    let report: serde_json::Value = serde_json::from_str(&wire).unwrap();
    assert_eq!(
        report["artifacts"][0]["outcome"]["posture"],
        "indeterminate"
    );
    assert_eq!(
        report["artifacts"][0]["outcome"]["observed_range"]["length"],
        16
    );
    assert_eq!(report["consumed"]["validator_entries"], 0);
    assert_eq!(report["consumed"]["bytes"], 16);
    assert_eq!(report["completeness"], "indeterminate");
    serving.abort();
}

#[test]
fn sink_failure_after_effect_preserves_the_cursor_and_releases_source_resources() {
    struct RejectOutcome;
    impl std::io::Write for RejectOutcome {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes == b"outcome" {
                Err(std::io::ErrorKind::BrokenPipe.into())
            } else {
                Ok(bytes.len())
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(parent.path());
    let target = target(&serving, parent.path());
    let mut handle = serving
        .start_physical_integrity_scrub(request(&serving, target))
        .unwrap();
    assert_eq!(
        handle.write_observation_report(
            Context::new("failed-sink", "runtime-report").unwrap(),
            16_384,
            &mut RejectOutcome
        ),
        Err(ReportDenial::SinkWriteFailure)
    );
    assert_eq!(handle.counters().completed_windows, 1);
    assert_released(&serving);
    assert!(
        matches!(handle.next_window(), Progress::Completed(counters) if counters.completed_windows == 1)
    );
    serving.close();
}

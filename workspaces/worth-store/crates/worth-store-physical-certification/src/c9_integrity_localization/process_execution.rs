use super::process_protocol::{
    executable_sha256, read_wire, write_create_new, ProcessSubjectReport, ProcessSubjectRequest,
    SUBJECT_REQUEST_ENV,
};
use super::ExternalReportPaths;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::{Command, ExitStatus};

pub(super) struct SubjectExecution {
    pub(super) process_id: u32,
    pub(super) report: ProcessSubjectReport,
}

pub(super) fn run_subject(
    executable: &Path,
    reports: &Path,
    label: &str,
    request: ProcessSubjectRequest,
) -> SubjectExecution {
    let request_path = reports.join(format!("{label}.request"));
    let report_path =
        ExternalReportPaths::authorize(request.store_root(), request.report_path().to_path_buf())
            .expect("external process report path");
    let request_path = ExternalReportPaths::authorize(request.store_root(), request_path)
        .expect("external process request path");
    write_create_new(request_path.as_path(), &request).expect("write subject request");
    let mut child = Command::new(executable)
        .args([
            "--exact",
            "c9_integrity_localization::c9_root_process_subject",
            "--nocapture",
        ])
        .env(SUBJECT_REQUEST_ENV, request_path.as_path())
        .spawn()
        .expect("launch process subject");
    let process_id = child.id();
    assert_ne!(
        process_id,
        std::process::id(),
        "subject must be a child process"
    );
    let status = child.wait().expect("wait for process subject");
    assert_success(status, "process subject");
    let report = read_wire(report_path.as_path()).expect("read process report");
    SubjectExecution { process_id, report }
}

pub(super) struct OfflineExecution {
    pub(super) process_id: u32,
    pub(super) executable_sha256: [u8; 32],
    pub(super) report: Value,
}

pub(super) fn run_offline_observer(
    executable: &Path,
    store_root: &Path,
    report_path: &Path,
    run: [u8; 32],
    scenario: [u8; 32],
) -> OfflineExecution {
    let report_path = ExternalReportPaths::authorize(store_root, report_path.to_path_buf())
        .expect("external offline report path");
    let mut child = Command::new(executable)
        .args([
            "observe",
            "--store-root",
            store_root.to_str().expect("UTF-8 Store root"),
            "--report",
            report_path.as_path().to_str().expect("UTF-8 report path"),
            "--max-entries",
            "4096",
            "--max-bytes",
            "134217728",
            "--max-open-files",
            "16",
            "--max-depth",
            "16",
            "--max-symlinks",
            "1",
            "--max-elapsed-ms",
            "30000",
            "--max-report-bytes",
            "1048576",
            "--run",
            &hex(run),
            "--scenario",
            &hex(scenario),
        ])
        .spawn()
        .expect("launch independent offline observer");
    let process_id = child.id();
    assert_ne!(
        process_id,
        std::process::id(),
        "offline observer must be a child process"
    );
    let status = child.wait().expect("wait for offline observer");
    assert_success(status, "offline observer");
    let report =
        serde_json::from_slice(&std::fs::read(report_path.as_path()).expect("read offline report"))
            .expect("parse independent report wire");
    OfflineExecution {
        process_id,
        executable_sha256: executable_sha256(executable).expect("observer digest"),
        report,
    }
}

fn assert_success(status: ExitStatus, role: &str) {
    assert!(status.success(), "{role} exited with {status}");
}

pub(super) fn fresh_identity(label: &str, root: &Path) -> [u8; 32] {
    Sha256::digest(format!("{label}:{}:{}", root.display(), std::process::id())).into()
}

pub(super) fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

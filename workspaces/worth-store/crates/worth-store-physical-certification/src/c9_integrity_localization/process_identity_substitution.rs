use std::path::Path;
use std::process::{Command, Stdio};

use super::process_protocol::{write_create_new, ProcessSubjectRequest, SUBJECT_REQUEST_ENV};
use super::{ClosedStoreProcessManifest, ExternalReportPaths};

pub(super) fn assert_recovery_store_substitution_is_denied(
    executable: &Path,
    stores: &Path,
    reports: &Path,
    baseline: &Path,
    manifest: &ClosedStoreProcessManifest,
    scenario: [u8; 32],
    run: [u8; 32],
) {
    let row = stores.join("recovery-store-substitution");
    manifest
        .copy_to(baseline, &row)
        .expect("fresh substitution row");
    let mut substituted_store = manifest.store_identity();
    substituted_store[0] ^= 1;
    let request = ProcessSubjectRequest::recovery(
        scenario,
        run,
        row,
        reports.join("recovery-store-substitution.report"),
        substituted_store,
    );
    let request_path = ExternalReportPaths::authorize(
        request.store_root(),
        reports.join("recovery-store-substitution.request"),
    )
    .expect("external substitution request path");
    let report_path =
        ExternalReportPaths::authorize(request.store_root(), request.report_path().to_path_buf())
            .expect("external substitution report path");
    write_create_new(request_path.as_path(), &request).expect("write substitution request");
    let child = Command::new(executable)
        .args([
            "--exact",
            "c9_integrity_localization::c9_root_process_subject",
            "--nocapture",
        ])
        .env(SUBJECT_REQUEST_ENV, request_path.as_path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch recovery substitution subject");
    assert_ne!(child.id(), std::process::id());
    let output = child
        .wait_with_output()
        .expect("wait for recovery substitution subject");
    assert!(
        !output.status.success(),
        "substituted Store identity passed"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("recovery Store identity denied: Substituted")
            || stderr.contains("recovery Store identity denied: Substituted"),
        "identity denial was not the child failure: stdout={stdout} stderr={stderr}",
    );
    assert!(!report_path.as_path().exists());
}

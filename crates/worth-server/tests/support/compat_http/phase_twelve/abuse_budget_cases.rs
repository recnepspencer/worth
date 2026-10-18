use serde_json::json;
use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerBinaryDownloadRequest, WorthServerCompatHttpRouteFamily,
    WorthServerTransferByteClass,
};

use {
    crate::shared_support_compat_http_phase_twelve_assertions::{
        assert_binary_counter, assert_budget_receipt, assert_budget_scope, assert_external_counter,
    },
    crate::shared_support_compat_http_phase_twelve_runtime::{
        build_phase_twelve_budget_limited_server, build_phase_twelve_server, download_input,
        head_read_request, prepared_request, read_request, upload_input,
    },
};

#[test]
fn compat_http_phase_twelve_budget_posture_stays_exact_for_structured_binary_and_metadata_lanes() {
    let server = build_phase_twelve_server();

    let read = prepared_request(&server, read_request(DiagnosticRichnessProfile::Standard));
    let read_budget = read.abuse_budget_receipt();
    assert_budget_receipt(
        &read_budget,
        WorthServerCompatHttpRouteFamily::Read,
        WorthServerTransferByteClass::StructuredPayload,
        None,
    );
    assert_budget_scope(&read_budget, "tenant-a", "workspace-42", "branch-9");
    let read_counters = read_budget
        .external_counters()
        .expect("structured route should expose external counters");
    assert_external_counter(read_counters, "compat_http.abuse.admitted", 1);
    assert_external_counter(read_counters, "compat_http.abuse.denied", 0);
    assert_external_counter(
        read_counters,
        "compat_http.abuse.structured_lane_assertions",
        1,
    );
    assert_external_counter(read_counters, "compat_http.abuse.binary_lane_assertions", 0);

    let upload = canonical_upload("phase-twelve-upload");
    let upload = match server.compat_http().upload(upload_input(&server, upload)) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected upload success, got {other:?}"),
    };
    let upload_budget = upload.abuse_budget_receipt();
    assert_budget_receipt(
        &upload_budget,
        WorthServerCompatHttpRouteFamily::Upload,
        WorthServerTransferByteClass::BinaryAuthoritative,
        None,
    );
    assert_budget_scope(&upload_budget, "tenant-a", "workspace-42", "branch-9");
    let upload_counters = upload_budget
        .binary_counters()
        .expect("upload budget should expose binary counters");
    assert_binary_counter(upload_counters, "compat_http.abuse.admitted", 1);
    assert_binary_counter(upload_counters, "compat_http.abuse.denied", 0);
    assert_binary_counter(
        upload_counters,
        "compat_http.abuse.binary_lane_assertions",
        1,
    );

    let download = match server.compat_http().download(download_input(
        &server,
        WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected download success, got {other:?}"),
    };
    let download_budget = download.abuse_budget_receipt();
    assert_budget_receipt(
        &download_budget,
        WorthServerCompatHttpRouteFamily::Download,
        WorthServerTransferByteClass::BinaryWire,
        None,
    );
    assert_budget_scope(&download_budget, "tenant-a", "workspace-42", "branch-9");
    let download_counters = download_budget
        .binary_counters()
        .expect("download budget should expose binary counters");
    assert_binary_counter(download_counters, "compat_http.abuse.admitted", 1);
    assert_binary_counter(
        download_counters,
        "compat_http.abuse.binary_lane_assertions",
        1,
    );

    let head_read = prepared_request(&server, head_read_request());
    let head_budget = head_read.abuse_budget_receipt();
    assert_budget_receipt(
        &head_budget,
        WorthServerCompatHttpRouteFamily::Read,
        WorthServerTransferByteClass::MetadataOnly,
        None,
    );
    assert_budget_scope(&head_budget, "tenant-a", "workspace-42", "branch-9");
    let head_counters = head_budget
        .external_counters()
        .expect("metadata-only route should expose external counters");
    assert_external_counter(
        head_counters,
        "compat_http.abuse.metadata_only_assertions",
        1,
    );
    assert_external_counter(
        head_counters,
        "compat_http.transfer.semantic_truth_drift",
        0,
    );
}

#[test]
fn compat_http_phase_twelve_budget_denials_retain_typed_scope_and_zero_truth_drift() {
    let server = build_phase_twelve_budget_limited_server();
    let outcome = server
        .compat_http()
        .prepare_request(read_request(DiagnosticRichnessProfile::Forensic));
    let denial = match outcome {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility denial, got {other:?}"),
    };
    let receipt = denial
        .abuse_budget_receipt()
        .expect("budget denial should retain an abuse budget receipt");
    assert_budget_receipt(
        receipt,
        WorthServerCompatHttpRouteFamily::Read,
        WorthServerTransferByteClass::StructuredPayload,
        Some("cannot admit diagnostics profile"),
    );
    assert_budget_scope(receipt, "tenant-a", "workspace-42", "branch-9");
    let counters = receipt
        .external_counters()
        .expect("structured budget denial should expose external counters");
    assert_external_counter(counters, "compat_http.abuse.denied", 1);
    assert_external_counter(counters, "compat_http.abuse.admitted", 0);
    assert_external_counter(counters, "compat_http.transfer.semantic_truth_drift", 0);
}

fn canonical_upload(identity: &str) -> worth_server::WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": identity,
                    "title.value": format!("Title for {identity}")
                }
            }
        }))
        .with_file_part("blob"),
    )
    .with_part(
        worth_server::WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(11)
            .with_body_bytes(b"hello-world".to_vec()),
    )
}

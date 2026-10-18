use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerBinaryDownloadRequest, WorthServerStreamSelection, WorthServerStreamingResponse,
};

use crate::compat_http_phase_eleven::compat_http_phase_eleven_assertions::{
    assert_binary_counter, assert_external_counter, finish_stream,
};
use crate::shared_support_compat_http_phase_four_runtime::request_support::{
    build_phase_four_server, build_phase_four_server_with_workspace_provider, compat_stream_input,
    oversized_streaming_provider, streaming_response_success,
};
use crate::shared_support_compat_http_phase_ten_runtime::{
    build_phase_ten_server_with_workspace_provider, canonical_upload,
    compat_download_execution_input, compat_download_success, compat_read_execution_input,
    compat_read_success, compat_upload_execution_input, compat_upload_success,
};
use crate::shared_support_query_handoff_runtime::ProfiledTestWorkspaceProvider;

#[test]
fn compat_http_phase_eleven_read_certification_preserves_exact_zeroed_external_rows() {
    let server = phase_eleven_server();
    let read = compat_read_success(server.compat_http().read(compat_read_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-11",
        "files.asset",
        DiagnosticRichnessProfile::Forensic,
    )));
    let read_certification = read.certification_bundle();
    assert_eq!(
        read_certification
            .operator_evidence_record()
            .classification_label(),
        "compatibility_read_succeeded"
    );
    assert_external_counter(
        read_certification.external_counters(),
        "compat_http.external.read.successes",
        1,
    );
    assert_external_counter(
        read_certification.external_counters(),
        "compat_http.external.upload.successes",
        0,
    );
    assert_external_counter(
        read_certification.external_counters(),
        "compat_http.external.download.successes",
        0,
    );
    assert_external_counter(
        read_certification.external_counters(),
        "compat_http.external.buffered_export.successes",
        0,
    );
}

#[test]
fn compat_http_phase_eleven_upload_and_download_certifications_keep_binary_counters_honest() {
    let server = phase_eleven_server();
    let upload = compat_upload_success(server.compat_http().upload(compat_upload_execution_input(
        &server,
        "tenant-a",
        "workspace-42",
        "branch-11",
        "files.asset",
        DiagnosticRichnessProfile::Forensic,
        canonical_upload("phase-eleven-upload"),
    )));
    let upload_certification = upload.certification_bundle();
    assert_eq!(
        upload_certification
            .operator_evidence_record()
            .classification_label(),
        "compatibility_upload_succeeded"
    );
    assert_external_counter(
        upload_certification.external_counters(),
        "compat_http.external.upload.successes",
        1,
    );
    assert_external_counter(
        upload_certification.external_counters(),
        "compat_http.external.read.successes",
        0,
    );
    assert_binary_counter(
        upload_certification.binary_counters(),
        "compat_http.upload.ingress_sessions_started",
        1,
    );
    assert_binary_counter(
        upload_certification.binary_counters(),
        "compat_http.upload.cleanup_operations",
        0,
    );

    let download = compat_download_success(server.compat_http().download(
        compat_download_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-11",
            "files.asset",
            DiagnosticRichnessProfile::Forensic,
            WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ));
    let download_certification = download.certification_bundle();
    assert_eq!(
        download_certification
            .operator_evidence_record()
            .classification_label(),
        "compatibility_download_succeeded"
    );
    assert_external_counter(
        download_certification.external_counters(),
        "compat_http.external.download.successes",
        1,
    );
    assert_external_counter(
        download_certification.external_counters(),
        "compat_http.external.read.successes",
        0,
    );
    assert_binary_counter(
        download_certification.binary_counters(),
        "compat_http.download.requests",
        1,
    );
    assert_binary_counter(
        download_certification.binary_counters(),
        "compat_http.download.forbidden_fallbacks",
        0,
    );
}

#[test]
fn compat_http_phase_eleven_incremental_stream_completion_keeps_buffering_and_fallback_zeroed() {
    let stream_server = build_phase_four_server();
    let streaming = streaming_response_success(stream_server.compat_http().stream(
        compat_stream_input(&stream_server, "users.profile"),
        WorthServerStreamSelection::incremental().with_chunk_bytes(8),
    ));
    let export = match streaming {
        WorthServerStreamingResponse::Stream(stream) => finish_stream(stream),
        other => panic!("expected incremental stream, got {other:?}"),
    };
    let stream_certification = export.certification_bundle();
    assert_eq!(
        stream_certification
            .operator_evidence_record()
            .classification_label(),
        "compatibility_stream_succeeded"
    );
    assert_external_counter(
        stream_certification.external_counters(),
        "compat_http.external.streaming.successes",
        1,
    );
    assert_binary_counter(
        stream_certification.binary_counters(),
        "compat_http.streaming.full_buffer_materializations",
        0,
    );
    assert_binary_counter(
        stream_certification.binary_counters(),
        "compat_http.streaming.background_export_fallbacks",
        0,
    );
}

#[test]
fn compat_http_phase_eleven_buffered_export_counts_full_buffering_without_claiming_fallback() {
    let stream_server = build_phase_four_server();
    let buffered = streaming_response_success(stream_server.compat_http().stream(
        compat_stream_input(&stream_server, "users.profile"),
        WorthServerStreamSelection::buffered(),
    ));
    let buffered_export = match buffered {
        WorthServerStreamingResponse::Buffered(export) => export,
        other => panic!("expected buffered export, got {other:?}"),
    };
    let buffered_certification = buffered_export.certification_bundle();
    assert_external_counter(
        buffered_certification.external_counters(),
        "compat_http.external.buffered_export.successes",
        1,
    );
    assert_binary_counter(
        buffered_certification.binary_counters(),
        "compat_http.streaming.full_buffer_materializations",
        1,
    );
    assert_binary_counter(
        buffered_certification.binary_counters(),
        "compat_http.streaming.background_export_fallbacks",
        0,
    );
}

#[test]
fn compat_http_phase_eleven_background_export_proves_no_sync_lane_byte_motion() {
    let background_server =
        build_phase_four_server_with_workspace_provider(oversized_streaming_provider(64, 64));
    let background = streaming_response_success(background_server.compat_http().stream(
        compat_stream_input(&background_server, "users.profile"),
        WorthServerStreamSelection::incremental().with_background_export_threshold_bytes(32),
    ));
    let background_export = match background {
        WorthServerStreamingResponse::BackgroundExport(export) => export,
        other => panic!("expected background export, got {other:?}"),
    };
    let background_certification = background_export.certification_bundle();
    assert_external_counter(
        background_certification.external_counters(),
        "compat_http.external.background_export.successes",
        1,
    );
    assert!(
        !background_export
            .file_envelope()
            .transfer_provenance()
            .byte_motion_observed(),
        "background export fallback must not claim emitted byte motion before delivery exists",
    );
    assert_binary_counter(
        background_certification.binary_counters(),
        "compat_http.streaming.background_export_fallbacks",
        1,
    );
    assert_binary_counter(
        background_certification.binary_counters(),
        "compat_http.streaming.full_buffer_materializations",
        0,
    );
}

fn phase_eleven_server() -> worth_server::WorthServer {
    build_phase_ten_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
    ))
}

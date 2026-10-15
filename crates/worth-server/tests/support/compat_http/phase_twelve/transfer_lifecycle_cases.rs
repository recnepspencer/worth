use serde_json::json;
use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerBinaryDownloadRequest, WorthServerCompatHttpRouteFamily,
    WorthServerTransferByteClass, WorthServerTransferCleanupReason,
};

use {crate::shared_support_compat_http_phase_eight_runtime::{
        build_phase_eight_server_with_workspace_provider, compat_download_success,
        compat_resume_success, download_input as resume_download_input,
        prepared_request as resume_prepared_request,
    }, crate::shared_support_compat_http_phase_four_runtime::request_support::{
        build_phase_four_server, compat_stream_input, default_stream_selection,
        streaming_response_success,
    }, crate::shared_support_compat_http_phase_twelve_assertions::{
        assert_binary_counter, assert_cleanup_evidence, assert_cleanup_scope,
        assert_external_counter,
    }, crate::shared_support_compat_http_phase_twelve_runtime::{build_phase_twelve_server, upload_input}, crate::shared_support_query_handoff_runtime::ProfiledTestWorkspaceProvider};
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;

#[test]
fn compat_http_phase_twelve_transfer_lifecycle_accounting_stays_narrow_and_reconstructable() {
    let streaming_server = build_phase_four_server();
    let stream = streaming_response_success(streaming_server.compat_http().stream(
        compat_stream_input(&streaming_server, "stream-users"),
        default_stream_selection(),
    ));
    let stream = match stream {
        worth_server::WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental stream, got {other:?}"),
    };
    let disconnect = stream.abort_due_to_disconnect();
    let disconnect_evidence = disconnect.transfer_cleanup_evidence();
    assert_cleanup_evidence(
        &disconnect_evidence,
        WorthServerCompatHttpRouteFamily::Streaming,
        WorthServerTransferByteClass::StructuredPayload,
        WorthServerTransferCleanupReason::ClientDisconnect,
    );
    assert_cleanup_scope(&disconnect_evidence, "tenant-a", "workspace-42", "branch-9");
    assert_external_counter(
        disconnect_evidence.external_counters(),
        "compat_http.transfer.disconnect_events",
        1,
    );
    assert_external_counter(
        disconnect_evidence.external_counters(),
        "compat_http.transfer.semantic_truth_drift",
        0,
    );
    assert_binary_counter(
        disconnect_evidence.binary_counters(),
        "compat_http.transfer.cleanup_operations",
        1,
    );

    let caller_cancel = streaming_response_success(streaming_server.compat_http().stream(
        compat_stream_input(&streaming_server, "stream-users"),
        default_stream_selection(),
    ));
    let caller_cancel = match caller_cancel {
        worth_server::WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental stream, got {other:?}"),
    };
    let caller_evidence = caller_cancel.cancel_by_caller().transfer_cleanup_evidence();
    assert_cleanup_evidence(
        &caller_evidence,
        WorthServerCompatHttpRouteFamily::Streaming,
        WorthServerTransferByteClass::StructuredPayload,
        WorthServerTransferCleanupReason::CallerCancelled,
    );
    assert_cleanup_scope(&caller_evidence, "tenant-a", "workspace-42", "branch-9");
    assert_external_counter(
        caller_evidence.external_counters(),
        "compat_http.transfer.caller_cancellations",
        1,
    );

    let backpressure = streaming_response_success(streaming_server.compat_http().stream(
        compat_stream_input(&streaming_server, "stream-users"),
        default_stream_selection(),
    ));
    let backpressure = match backpressure {
        worth_server::WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental stream, got {other:?}"),
    };
    let backpressure_evidence = backpressure
        .abort_due_to_backpressure()
        .transfer_cleanup_evidence();
    assert_cleanup_evidence(
        &backpressure_evidence,
        WorthServerCompatHttpRouteFamily::Streaming,
        WorthServerTransferByteClass::StructuredPayload,
        WorthServerTransferCleanupReason::DownstreamBackpressure,
    );
    assert_cleanup_scope(
        &backpressure_evidence,
        "tenant-a",
        "workspace-42",
        "branch-9",
    );
    assert_external_counter(
        backpressure_evidence.external_counters(),
        "compat_http.transfer.backpressure_aborts",
        1,
    );

    let upload_server = build_phase_twelve_server();
    let prepared = match upload_server.compat_http().prepare_upload(upload_input(
        &upload_server,
        canonical_upload("cleanup-target"),
    )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected upload preparation success, got {other:?}"),
    };
    let session = match upload_server.compat_http().begin_binary_ingress(prepared) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected upload ingress session, got {other:?}"),
    };
    let cleanup = upload_server
        .compat_http()
        .interrupt_binary_ingress(&session)
        .expect("cleanup should succeed");
    let cleanup_evidence = cleanup.transfer_cleanup_evidence();
    assert_cleanup_evidence(
        &cleanup_evidence,
        WorthServerCompatHttpRouteFamily::Upload,
        WorthServerTransferByteClass::BinaryAuthoritative,
        WorthServerTransferCleanupReason::UploadInterrupted,
    );
    assert_cleanup_scope(&cleanup_evidence, "tenant-a", "workspace-42", "branch-9");
    assert_external_counter(
        cleanup_evidence.external_counters(),
        "compat_http.transfer.staged_cleanup_events",
        1,
    );
    assert_binary_counter(
        cleanup_evidence.binary_counters(),
        "compat_http.transfer.cleanup_staged_bytes",
        11,
    );
    assert_binary_counter(
        cleanup_evidence.binary_counters(),
        "compat_http.transfer.semantic_truth_drift",
        0,
    );

    let retry_server =
        build_phase_eight_server_with_workspace_provider(ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ));
    let body = b"phase-twelve-retry-body".to_vec();
    let first = compat_download_success(
        retry_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                resume_prepared_request(
                    &retry_server,
                    resume_download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-9")
                    .build()
                    .expect("initial retry request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body.clone()),
            ),
        ),
    );
    let resume = compat_resume_success(
        retry_server
            .compat_http()
            .plan_binary_resume(first.session()),
    );
    let resumed = compat_download_success(
        retry_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                resume_prepared_request(
                    &retry_server,
                    resume_download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=10-")
                    .build()
                    .expect("resumed retry request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(body).with_resume_request(
                    worth_server::WorthServerBinaryResumeRequest::resume_from(resume),
                ),
            ),
        ),
    );
    let retry_evidence = resumed
        .transfer_cleanup_evidence()
        .expect("resumed downloads should emit retry lifecycle evidence");
    assert_cleanup_evidence(
        &retry_evidence,
        WorthServerCompatHttpRouteFamily::Download,
        WorthServerTransferByteClass::BinaryWire,
        WorthServerTransferCleanupReason::DownloadRetryAdmitted,
    );
    assert_cleanup_scope(&retry_evidence, "tenant-a", "workspace-42", "branch-9");
    assert_external_counter(
        retry_evidence.external_counters(),
        "compat_http.transfer.retry_events",
        1,
    );
    assert_binary_counter(
        retry_evidence.binary_counters(),
        "compat_http.transfer.semantic_truth_drift",
        0,
    );

    let ordinary_download = compat_download_success(
        retry_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                resume_prepared_request(
                    &retry_server,
                    resume_download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .build()
                    .expect("ordinary download request should validate"),
                ),
                "files.asset.download",
                WorthServerBinaryDownloadRequest::new(b"ordinary-download".to_vec()),
            ),
        ),
    );
    assert!(
        ordinary_download.transfer_cleanup_evidence().is_none(),
        "ordinary downloads must not fabricate retry lifecycle evidence"
    );
}

fn canonical_upload(identity: &str) -> worth_server::WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": identity,
                    "title.value": format!("Cleanup {identity}")
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

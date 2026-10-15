#![allow(dead_code)]

use std::sync::{atomic::AtomicUsize, Arc};

use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_proof::TransitionOutcome;
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServer, WorthServerBinaryDownload, WorthServerCompatibilityExport,
    WorthServerCompatibilityMutation, WorthServerCompatibilityRead, WorthServerDirectMutation,
    WorthServerDirectRead, WorthServerMultipartUpload, WorthServerQueryHandoffDenial,
    WorthServerStreamCancellationKind, WorthServerStreamCancellationReceipt,
    WorthServerStreamingResponse,
};

use crate::shared_support_compat_http_phase_four_runtime::request_support::{
    compat_stream_input, default_stream_selection, streaming_response_success,
};
use {crate::shared_support_compat_http_phase_eight_runtime as compat_http_phase_eight_runtime, crate::shared_support_compat_http_phase_four_assertions as compat_http_phase_four_assertions, crate::shared_support_compat_http_phase_nine_runtime as compat_http_phase_nine_runtime, crate::shared_support_compat_http_phase_ten_runtime as compat_http_phase_ten_runtime, crate::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime, crate::shared_support_compat_http_phase_two_runtime as compat_http_phase_two_runtime, crate::shared_support_query_handoff_runtime as query_handoff_runtime, crate::shared_support_worth_native_assertions as worth_native_assertions};

pub(crate) fn phase_thirteen_server() -> WorthServer {
    compat_http_phase_ten_runtime::build_phase_ten_server_with_workspace_provider(
        query_handoff_runtime::ProfiledTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ),
    )
}

pub(crate) fn phase_thirteen_counting_server() -> (WorthServer, Arc<AtomicUsize>) {
    let attempted_writes = Arc::new(AtomicUsize::new(0));
    let server = compat_http_phase_three_runtime::build_phase_three_server_with_workspace_provider(
        compat_http_phase_three_runtime::StatefulCountingMutationWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );
    (server, attempted_writes)
}

pub(crate) fn direct_and_compat_read(
    server: &WorthServer,
    operation_name: &str,
) -> (
    worth_server::WorthServerAdmittedDirectDeclaration,
    WorthServerDirectRead,
    WorthServerCompatibilityRead,
) {
    let (session, declaration) =
        compat_http_phase_two_runtime::worth_native_named_read(server, operation_name);
    let direct = match session.direct().read(&declaration) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected direct read success, got {other:?}"),
    };
    let compat =
        match server
            .compat_http()
            .read(compat_http_phase_two_runtime::compat_execution_input(
                server,
                operation_name,
            )) {
            TransitionOutcome::Success(value) => value,
            other => panic!("expected compat read success, got {other:?}"),
        };
    (declaration, direct, compat)
}

pub(crate) fn compat_read_with_diagnostics(
    server: &WorthServer,
    operation_name: &str,
    diagnostics_profile: DiagnosticRichnessProfile,
) -> WorthServerCompatibilityRead {
    match server
        .compat_http()
        .read(compat_http_phase_ten_runtime::compat_read_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            diagnostics_profile,
        )) {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected diagnostics-scoped read success, got {other:?}"),
    }
}

pub(crate) fn direct_and_compat_mutation(
    server: &WorthServer,
    identity: &str,
) -> (WorthServerDirectMutation, WorthServerCompatibilityMutation) {
    let compat =
        compat_http_phase_three_runtime::compat_mutation_success(server.compat_http().mutate(
            compat_http_phase_three_runtime::compat_mutation_execution_input(
                server,
                "tasks.insert",
                compat_http_phase_three_runtime::single_insert_body(identity),
            ),
        ));
    let direct = compat_http_phase_three_runtime::direct_mutation_success(
        worth_native_assertions::worth_native_session(server)
            .direct()
            .mutate(&worth_server::WorthServerQueryOperation::single_mutation(
                "tasks.insert",
                compat_http_phase_three_runtime::insert_task(identity),
            )),
    );
    (direct, compat)
}

pub(crate) fn idempotent_mutation(
    server: &WorthServer,
    identity: &str,
    idempotency_key: &str,
) -> WorthServerCompatibilityMutation {
    compat_http_phase_three_runtime::compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                compat_http_phase_three_runtime::prepared_mutation_request(
                    server,
                    compat_http_phase_three_runtime::mutation_input("tasks.insert")
                        .with_header("idempotency-key", idempotency_key)
                        .build()
                        .expect("idempotent mutation input should validate"),
                ),
                "tasks.insert",
                compat_http_phase_three_runtime::single_insert_body(identity),
            ),
        ),
    )
}

pub(crate) fn idempotent_mutation_conflict(
    server: &WorthServer,
    identity: &str,
    idempotency_key: &str,
) -> WorthServerQueryHandoffDenial {
    compat_http_phase_three_runtime::compat_mutation_denied(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                compat_http_phase_three_runtime::prepared_mutation_request(
                    server,
                    compat_http_phase_three_runtime::mutation_input("tasks.insert")
                        .with_header("idempotency-key", idempotency_key)
                        .build()
                        .expect("idempotent conflict input should validate"),
                ),
                "tasks.insert",
                compat_http_phase_three_runtime::single_insert_body(identity),
            ),
        ),
    )
}

pub(crate) fn finished_incremental_export(
    server: &WorthServer,
    operation_name: &str,
    chunk_bytes: usize,
) -> WorthServerCompatibilityExport {
    let response = streaming_response_success(server.compat_http().stream(
        compat_stream_input(server, operation_name),
        worth_server::WorthServerStreamSelection::incremental().with_chunk_bytes(chunk_bytes),
    ));
    let mut stream = match response {
        WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental export, got {other:?}"),
    };
    let _ = compat_http_phase_four_assertions::collect_stream_bytes(&mut stream)
        .expect("stream bytes should serialize");
    stream
        .finish()
        .expect("incremental stream should finish after full consumption")
}

pub(crate) fn buffered_export(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerCompatibilityExport {
    let response = streaming_response_success(server.compat_http().stream(
        compat_stream_input(server, operation_name),
        worth_server::WorthServerStreamSelection::buffered(),
    ));
    match response {
        WorthServerStreamingResponse::Buffered(value) => value,
        other => panic!("expected buffered export, got {other:?}"),
    }
}

pub(crate) fn cancellation_receipt(
    server: &WorthServer,
    operation_name: &str,
    kind: WorthServerStreamCancellationKind,
) -> WorthServerStreamCancellationReceipt {
    let response = streaming_response_success(server.compat_http().stream(
        compat_stream_input(server, operation_name),
        default_stream_selection(),
    ));
    let mut stream = match response {
        WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected stream cancellation probe, got {other:?}"),
    };
    let _ = stream
        .next_chunk()
        .expect("cancellation probe chunk should serialize")
        .expect("cancellation probe should emit a chunk");
    match kind {
        WorthServerStreamCancellationKind::ClientDisconnect => stream.abort_due_to_disconnect(),
        WorthServerStreamCancellationKind::DownstreamBackpressure => {
            stream.abort_due_to_backpressure()
        }
        WorthServerStreamCancellationKind::CallerCancelled => stream.cancel_by_caller(),
    }
}

pub(crate) fn canonical_upload_success(
    server: &WorthServer,
    operation_name: &str,
    identity: &str,
) -> worth_server::WorthServerCompatibilityUpload {
    compat_http_phase_ten_runtime::compat_upload_success(server.compat_http().upload(
        compat_http_phase_ten_runtime::compat_upload_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            DiagnosticRichnessProfile::Standard,
            compat_http_phase_ten_runtime::canonical_upload(identity),
        ),
    ))
}

pub(crate) fn upload_with_payload(
    server: &WorthServer,
    operation_name: &str,
    upload: WorthServerMultipartUpload,
) -> worth_server::WorthServerCompatibilityUploadOutcome<worth_server::WorthServerCompatibilityUpload>
{
    server.compat_http().upload(
        compat_http_phase_ten_runtime::compat_upload_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            DiagnosticRichnessProfile::Standard,
            upload,
        ),
    )
}

pub(crate) fn canonical_download_success(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerBinaryDownload {
    compat_http_phase_ten_runtime::compat_download_success(server.compat_http().download(
        compat_http_phase_ten_runtime::compat_download_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            DiagnosticRichnessProfile::Standard,
            worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ))
}

pub(crate) fn resumed_download_success(server: &WorthServer) -> WorthServerBinaryDownload {
    let body = b"phase-thirteen-retry-body".to_vec();
    let first = compat_http_phase_eight_runtime::compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                compat_http_phase_eight_runtime::prepared_request(
                    server,
                    compat_http_phase_eight_runtime::download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-9")
                    .build()
                    .expect("initial range request should validate"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(body.clone()),
            )),
    );
    let resume = compat_http_phase_eight_runtime::compat_resume_success(
        server.compat_http().plan_binary_resume(first.session()),
    );
    compat_http_phase_eight_runtime::compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                compat_http_phase_eight_runtime::prepared_request(
                    server,
                    compat_http_phase_eight_runtime::download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=10-")
                    .build()
                    .expect("resumed range request should validate"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(body).with_resume_request(
                    worth_server::WorthServerBinaryResumeRequest::resume_from(resume),
                ),
            )),
    )
}

pub(crate) fn malformed_upload_denial(server: &WorthServer) -> WorthServerQueryHandoffDenial {
    compat_http_phase_nine_runtime::compat_upload_denied(upload_with_payload(
        server,
        "files.asset",
        compat_http_phase_nine_runtime::malformed_upload(),
    ))
}

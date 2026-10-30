pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::Arc};

pub(crate) use worth_foundational::facade::DiagnosticRichnessProfile;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub(crate) use {
    worth_server::WorthServer, worth_server::WorthServerBinaryDownload,
    worth_server::WorthServerCompatibilityExport, worth_server::WorthServerCompatibilityMutation,
    worth_server::WorthServerCompatibilityRead, worth_server::WorthServerDirectMutation,
    worth_server::WorthServerDirectRead, worth_server::WorthServerMultipartUpload,
    worth_server::WorthServerQueryHandoffDenial, worth_server::WorthServerStreamCancellationKind,
    worth_server::WorthServerStreamCancellationReceipt, worth_server::WorthServerStreamingResponse,
};

pub(crate) use {
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime as compat_http_phase_eight_runtime,
    worth_testkit_server::shared_support_compat_http_phase_four_assertions as compat_http_phase_four_assertions,
    worth_testkit_server::shared_support_compat_http_phase_nine_runtime as compat_http_phase_nine_runtime,
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime as compat_http_phase_ten_runtime,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime,
    worth_testkit_server::shared_support_compat_http_phase_two_runtime as compat_http_phase_two_runtime,
    worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime,
    worth_testkit_server::shared_support_worth_native_assertions as worth_native_assertions,
};
pub(crate) use {
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::compat_stream_input,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::default_stream_selection,
    worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::streaming_response_success,
};

pub(crate) fn phase_thirteen_server() -> WorthServer {
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::server::build_phase_ten_server_with_workspace_provider(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
        ),
    )
}

pub(crate) fn phase_thirteen_counting_server() -> (WorthServer, Arc<AtomicUsize>) {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server_with_workspace_provider(
        worth_testkit_server::shared_support_compat_http_phase_three_runtime::StatefulCountingMutationWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
        worth_testkit_server::shared_support_compat_http_phase_two_runtime::worth_native_named_read(
            server,
            operation_name,
        );
    let direct = match session.direct().read(&declaration) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected direct read success, got {other:?}"),
    };
    let compat = match server.compat_http().read(
        worth_testkit_server::shared_support_compat_http_phase_two_runtime::compat_execution_input(
            server,
            operation_name,
        ),
    ) {
        worth_proof::TransitionOutcome::Success(value) => value,
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
        .read(worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_read_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            diagnostics_profile,
        )) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected diagnostics-scoped read success, got {other:?}"),
    }
}

pub(crate) fn direct_and_compat_mutation(
    server: &WorthServer,
    identity: &str,
) -> (WorthServerDirectMutation, WorthServerCompatibilityMutation) {
    let compat =
        worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_success(server.compat_http().mutate(
            worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_execution_input(
                server,
                "tasks.insert",
                worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body(identity),
            ),
        ));
    let direct = worth_testkit_server::shared_support_compat_http_phase_three_runtime::direct_mutation_success(
        worth_testkit_server::shared_support_worth_native_assertions::worth_native_session(server)
            .direct()
            .mutate(&worth_server::WorthServerQueryOperation::single_mutation(
                "tasks.insert",
                worth_testkit_server::shared_support_compat_http_phase_three_runtime::insert_task(identity),
            )),
    );
    (direct, compat)
}

pub(crate) fn idempotent_mutation(
    server: &WorthServer,
    identity: &str,
    idempotency_key: &str,
) -> WorthServerCompatibilityMutation {
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_success(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_three_runtime::prepared_mutation_request(
                    server,
                    worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_input("tasks.insert")
                        .with_header("idempotency-key", idempotency_key)
                        .build()
                        .expect("idempotent mutation input should validate"),
                ),
                "tasks.insert",
                worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body(identity),
            ),
        ),
    )
}

pub(crate) fn idempotent_mutation_conflict(
    server: &WorthServer,
    identity: &str,
    idempotency_key: &str,
) -> WorthServerQueryHandoffDenial {
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_denied(
        server.compat_http().mutate(
            worth_server::WorthServerCompatibilityMutationExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_three_runtime::prepared_mutation_request(
                    server,
                    worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_input("tasks.insert")
                        .with_header("idempotency-key", idempotency_key)
                        .build()
                        .expect("idempotent conflict input should validate"),
                ),
                "tasks.insert",
                worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body(identity),
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
        worth_server::WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental export, got {other:?}"),
    };
    let _ = worth_testkit_server::shared_support_compat_http_phase_four_assertions::collect_stream_bytes(&mut stream)
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
        worth_server::WorthServerStreamingResponse::Buffered(value) => value,
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
        worth_server::WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected stream cancellation probe, got {other:?}"),
    };
    let _ = stream
        .next_chunk()
        .expect("cancellation probe chunk should serialize")
        .expect("cancellation probe should emit a chunk");
    match kind {
        worth_server::WorthServerStreamCancellationKind::ClientDisconnect => {
            stream.abort_due_to_disconnect()
        }
        worth_server::WorthServerStreamCancellationKind::DownstreamBackpressure => {
            stream.abort_due_to_backpressure()
        }
        worth_server::WorthServerStreamCancellationKind::CallerCancelled => {
            stream.cancel_by_caller()
        }
    }
}

pub(crate) fn canonical_upload_success(
    server: &WorthServer,
    operation_name: &str,
    identity: &str,
) -> worth_server::WorthServerCompatibilityUpload {
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_upload_success(server.compat_http().upload(
        worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_upload_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
            worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::fixtures::canonical_upload(identity),
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
        worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_upload_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
            upload,
        ),
    )
}

pub(crate) fn canonical_download_success(
    server: &WorthServer,
    operation_name: &str,
) -> WorthServerBinaryDownload {
    worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::outcomes::compat_download_success(server.compat_http().download(
        worth_testkit_server::shared_support_compat_http_phase_ten_runtime::runtime_support::requests::compat_download_execution_input(
            server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            operation_name,
            worth_foundational::facade::DiagnosticRichnessProfile::Standard,
            worth_server::WorthServerBinaryDownloadRequest::new(b"hello-world".to_vec()),
        ),
    ))
}

pub(crate) fn resumed_download_success(server: &WorthServer) -> WorthServerBinaryDownload {
    let body = b"phase-thirteen-retry-body".to_vec();
    let first = worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_eight_runtime::prepared_request(
                    server,
                    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::download_input(
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
    let resume =
        worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_resume_success(
            server.compat_http().plan_binary_resume(first.session()),
        );
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_success(
        server
            .compat_http()
            .download(worth_server::WorthServerBinaryDownloadExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_eight_runtime::prepared_request(
                    server,
                    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::download_input(
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
    worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::outcomes::compat_upload_denied(upload_with_payload(
        server,
        "files.asset",
        worth_testkit_server::shared_support_compat_http_phase_nine_runtime::runtime_support::fixtures::malformed_upload(),
    ))
}

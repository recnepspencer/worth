#[allow(unused_imports)] use crate::compat_http_phase_thirteen_boundary::ordinary_context::compat_http_phase_thirteen_transfer_hostility_cases::*;

use {
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_binary_counter,
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_denial_contains,
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_external_counter,
    crate::shared_support_compat_http_phase_thirteen_runtime::cancellation_receipt,
    crate::shared_support_compat_http_phase_thirteen_runtime::phase_thirteen_server,
};

#[test]
fn compat_http_transfer_hostility_matrix_preserves_exact_counters_and_zero_truth_drift() {
    let slowloris_server = phase_thirteen_server();
    let cleanup_server = phase_thirteen_server();
    let slowloris = match slowloris_server.compat_http().upload(
        worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::upload_input(
            &slowloris_server,
            worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::drip_fed_upload(
            ),
        ),
    ) {
        worth_proof::TransitionOutcome::Denied(value) => value,
        other => panic!("expected slowloris denial, got {other:?}"),
    };
    let slowloris_receipt = slowloris
        .abuse_budget_receipt()
        .expect("slowloris denial should retain abuse budget receipt");
    assert_binary_counter(
        slowloris_receipt
            .binary_counters()
            .expect("slowloris denial should expose binary counters"),
        "compat_http.transfer.slowloris_cutoffs",
        1,
    );
    assert_binary_counter(
        slowloris_receipt
            .binary_counters()
            .expect("slowloris denial should expose binary counters"),
        "compat_http.transfer.semantic_truth_drift",
        0,
    );

    let disconnect = cancellation_receipt(
        &slowloris_server,
        "users.profile",
        worth_server::WorthServerStreamCancellationKind::ClientDisconnect,
    );
    assert_external_counter(
        disconnect.transfer_cleanup_evidence().external_counters(),
        "compat_http.transfer.disconnect_events",
        1,
    );
    assert_binary_counter(
        disconnect.transfer_cleanup_evidence().binary_counters(),
        "compat_http.transfer.semantic_truth_drift",
        0,
    );

    let prepared_upload = match cleanup_server.compat_http().prepare_upload(
        worth_testkit_server::shared_support_compat_http_phase_twelve_runtime::upload_input(
            &cleanup_server,
            staged_cleanup_upload(),
        ),
    ) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected upload preparation success, got {other:?}"),
    };
    let ingress_session = match cleanup_server
        .compat_http()
        .begin_binary_ingress(prepared_upload)
    {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected staged upload session, got {other:?}"),
    };
    let cleanup = cleanup_server
        .compat_http()
        .interrupt_binary_ingress(&ingress_session)
        .expect("staged upload cleanup should succeed");
    assert_external_counter(
        cleanup.transfer_cleanup_evidence().external_counters(),
        "compat_http.transfer.staged_cleanup_events",
        1,
    );
    assert_binary_counter(
        cleanup.transfer_cleanup_evidence().binary_counters(),
        "compat_http.transfer.semantic_truth_drift",
        0,
    );

    let range_server =
        worth_testkit_server::shared_support_compat_http_phase_eight_runtime::build_phase_eight_server_with_workspace_provider(
            worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
                worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            ),
        );
    let multi_range_denial = worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_denied(
        range_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_eight_runtime::prepared_request(
                    &range_server,
                    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-4,6-9")
                    .build()
                    .expect("multi-range request should validate"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(
                    b"phase-thirteen-download-body".to_vec(),
                ),
            ),
        ),
    );
    assert_denial_contains(
        &multi_range_denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "multi-range requests",
    );

    let resume_server =
        worth_testkit_server::shared_support_compat_http_phase_eight_runtime::build_phase_eight_server_with_workspace_provider(
            worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
                worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            ),
        );
    let baseline_body = b"phase-thirteen-download-body".to_vec();
    let first = worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_success(
        resume_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_eight_runtime::prepared_request(
                    &resume_server,
                    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=0-10")
                    .build()
                    .expect("initial resume request should validate"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(baseline_body.clone()),
            ),
        ),
    );
    let resume =
        worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_resume_success(
            resume_server
                .compat_http()
                .plan_binary_resume(first.session()),
        );
    let integrity_mismatch = worth_testkit_server::shared_support_compat_http_phase_eight_runtime::compat_download_denied(
        resume_server.compat_http().download(
            worth_server::WorthServerBinaryDownloadExecutionInput::new(
                worth_testkit_server::shared_support_compat_http_phase_eight_runtime::prepared_request(
                    &resume_server,
                    worth_testkit_server::shared_support_compat_http_phase_eight_runtime::download_input(
                        "tenant-a",
                        "workspace-42",
                        "branch-9",
                        "files.asset.download",
                    )
                    .with_header("range", "bytes=11-")
                    .build()
                    .expect("integrity mismatch request should validate"),
                ),
                "files.asset.download",
                worth_server::WorthServerBinaryDownloadRequest::new(
                    b"phase-thirteen-download-bodz".to_vec(),
                )
                .with_resume_request(
                    worth_server::WorthServerBinaryResumeRequest::resume_from(resume),
                ),
            ),
        ),
    );
    assert_denial_contains(
        &integrity_mismatch,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityDownloadRequestInvalid,
        "resume integrity digest mismatch",
    );

    let compression_server =
        worth_testkit_server::shared_support_compat_http_phase_six_runtime::build_phase_six_server_with_workspace_provider(
            worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledTestWorkspaceProvider::new(
                worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            ),
        );
    let compression_denial = worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_denied(
        compression_server.compat_http().upload(
            worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_execution_input(
                &compression_server,
                "tenant-a",
                "workspace-42",
                "branch-9",
                "files.compressed.upload",
                "boundary-compressed",
                compression_ratio_abuse_upload(),
            ),
        ),
    );
    assert_denial_contains(
        &compression_denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "decompression ratio cap",
    );
}

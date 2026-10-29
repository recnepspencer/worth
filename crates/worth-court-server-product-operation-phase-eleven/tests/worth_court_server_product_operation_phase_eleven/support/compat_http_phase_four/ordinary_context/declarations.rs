#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_four_assertions as compat_http_phase_four_assertions};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_four_runtime as compat_http_phase_four_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};
#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

pub(crate) use {worth_proof::TransitionOutcome};
pub(crate) use {worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerStreamCancellationKind, worth_server::WorthServerStreamFinishError, worth_server::WorthServerStreamingResponse};

pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_four_assertions::assert_cancellation_counters, worth_testkit_server::shared_support_compat_http_phase_four_assertions::assert_cancellation_kind, worth_testkit_server::shared_support_compat_http_phase_four_assertions::assert_export_counters, worth_testkit_server::shared_support_compat_http_phase_four_assertions::assert_read_artifact_parity, worth_testkit_server::shared_support_compat_http_phase_four_assertions::collect_stream_bytes};
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::build_phase_four_server, worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::build_phase_four_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::compat_read_input, worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::compat_stream_input, worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::default_stream_selection, worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::oversized_streaming_provider, worth_testkit_server::shared_support_compat_http_phase_four_runtime::request_support::streaming_response_success};

pub(crate) fn expect_stream(
    response: WorthServerStreamingResponse,
) -> worth_server::WorthServerCompatibilityStream {
    match response {
        worth_server::WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental stream, got {other:?}"),
    }
}

pub(crate) fn cancel_after_first_chunk(
    server: &worth_server::WorthServer,
    kind: WorthServerStreamCancellationKind,
) -> worth_server::WorthServerStreamCancellationReceipt {
    let response = streaming_response_success(server.compat_http().stream(
        compat_stream_input(server, "users.profile"),
        default_stream_selection(),
    ));
    let mut stream = expect_stream(response);
    let _ = stream
        .next_chunk()
        .expect("cancellation probe chunk should serialize")
        .expect("cancellation probe should emit a first chunk");
    match kind {
        worth_server::WorthServerStreamCancellationKind::ClientDisconnect => stream.abort_due_to_disconnect(),
        worth_server::WorthServerStreamCancellationKind::DownstreamBackpressure => {
            stream.abort_due_to_backpressure()
        }
        worth_server::WorthServerStreamCancellationKind::CallerCancelled => stream.cancel_by_caller(),
    }
}
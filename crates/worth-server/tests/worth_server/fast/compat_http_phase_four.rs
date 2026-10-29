#[allow(unused_imports)]
use crate::shared_support_compat_http_phase_four_assertions as compat_http_phase_four_assertions;
#[allow(unused_imports)]
use crate::shared_support_compat_http_phase_four_runtime as compat_http_phase_four_runtime;
#[allow(unused_imports)]
use crate::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use crate::shared_support_test_bootstrap_policy as test_bootstrap_policy;

use worth_proof::TransitionOutcome;
use worth_server::{
    WorthServerQueryHandoffDenialCode, WorthServerStreamCancellationKind,
    WorthServerStreamFinishError, WorthServerStreamingResponse,
};

use compat_http_phase_four_assertions::{
    assert_cancellation_counters, assert_cancellation_kind, assert_export_counters,
    assert_read_artifact_parity, collect_stream_bytes,
};
use compat_http_phase_four_runtime::request_support::{
    build_phase_four_server, build_phase_four_server_with_workspace_provider, compat_read_input,
    compat_stream_input, default_stream_selection, oversized_streaming_provider,
    streaming_response_success,
};

#[test]
fn compat_http_streaming_matches_buffered_read_on_canonical_artifacts() {
    let server = build_phase_four_server();
    let buffered_read = match server
        .compat_http()
        .read(compat_read_input(&server, "users.profile"))
    {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility read success, got {other:?}"),
    };
    let response = streaming_response_success(server.compat_http().stream(
        compat_stream_input(&server, "users.profile"),
        default_stream_selection(),
    ));
    let mut stream = match response {
        WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental streaming response, got {other:?}"),
    };
    let (streamed_bytes, streamed_chunks) =
        collect_stream_bytes(&mut stream).expect("stream bytes should serialize");
    let export = stream.finish().expect("stream completion should succeed");

    assert!(streamed_chunks >= 1);
    assert_read_artifact_parity(&buffered_read, export.read());
    assert_eq!(streamed_bytes, export.payload_bytes());
    assert_eq!(
        String::from_utf8(export.payload_bytes().to_vec()).expect("payload should be utf-8 json"),
        r#"[{"identity":{"id":"user-1"},"profile":{"display_name":"Ada Worth"}}]"#
    );
    assert_export_counters(
        &export,
        [
            streamed_chunks as u64,
            export.payload_bytes().len() as u64,
            0,
            1,
            0,
            0,
            0,
            0,
        ],
    );
}

#[test]
fn compat_http_streaming_does_not_change_semantics_when_chunk_boundaries_shift() {
    let server = build_phase_four_server();
    let small = streaming_response_success(server.compat_http().stream(
        compat_stream_input(&server, "users.profile"),
        worth_server::WorthServerStreamSelection::incremental().with_chunk_bytes(5),
    ));
    let large = streaming_response_success(server.compat_http().stream(
        compat_stream_input(&server, "users.profile"),
        worth_server::WorthServerStreamSelection::incremental().with_chunk_bytes(128),
    ));
    let mut small_stream = expect_stream(small);
    let mut large_stream = expect_stream(large);
    let (small_bytes, small_chunks) =
        collect_stream_bytes(&mut small_stream).expect("small chunks should serialize");
    let (large_bytes, large_chunks) =
        collect_stream_bytes(&mut large_stream).expect("large chunks should serialize");
    let small_export = small_stream.finish().expect("small stream should finish");
    let large_export = large_stream.finish().expect("large stream should finish");

    assert!(small_chunks > large_chunks);
    assert_eq!(small_bytes, large_bytes);
    assert_read_artifact_parity(small_export.read(), large_export.read());
    assert_eq!(small_export.payload_bytes(), large_export.payload_bytes());
}

#[test]
fn compat_http_streaming_keeps_first_chunk_honest_for_large_exports() {
    let server =
        build_phase_four_server_with_workspace_provider(oversized_streaming_provider(32, 96));
    let response = streaming_response_success(server.compat_http().stream(
        compat_stream_input(&server, "users.profile"),
        worth_server::WorthServerStreamSelection::incremental().with_chunk_bytes(23),
    ));
    let mut stream = expect_stream(response);
    let first = stream
        .next_chunk()
        .expect("first chunk should serialize")
        .expect("first chunk should exist");

    assert!(first.bytes().len() <= 23);
    assert_eq!(stream.estimated_payload_bytes(), None);
    assert_eq!(stream.emitted_chunks(), 1);
    assert_eq!(stream.emitted_bytes(), first.bytes().len());

    assert_eq!(
        stream.finish().err(),
        Some(WorthServerStreamFinishError::StreamNotFullyConsumed)
    );
}

#[test]
fn compat_http_streaming_large_export_completes_after_full_incremental_consumption() {
    let server =
        build_phase_four_server_with_workspace_provider(oversized_streaming_provider(32, 96));
    let response = streaming_response_success(server.compat_http().stream(
        compat_stream_input(&server, "users.profile"),
        worth_server::WorthServerStreamSelection::incremental().with_chunk_bytes(23),
    ));
    let mut stream = expect_stream(response);
    let (streamed_bytes, _) =
        collect_stream_bytes(&mut stream).expect("large stream bytes should serialize");
    let streamed_chunk_count = stream.emitted_chunks();
    let export = stream.finish().expect("large stream should finish");

    assert_eq!(streamed_bytes, export.payload_bytes());
    assert_export_counters(
        &export,
        [
            streamed_chunk_count as u64,
            export.payload_bytes().len() as u64,
            0,
            1,
            0,
            0,
            0,
            0,
        ],
    );
}

#[test]
fn compat_http_streaming_head_is_explicitly_buffered_with_zero_transfer() {
    let server = build_phase_four_server();
    let response = streaming_response_success(server.compat_http().stream(
        compat_http_phase_four_runtime::request_support::compat_stream_head_input(
            &server,
            "users.profile",
        ),
        default_stream_selection(),
    ));
    let export = match response {
        WorthServerStreamingResponse::Buffered(value) => value,
        other => panic!("expected buffered head export, got {other:?}"),
    };

    assert_eq!(export.payload_bytes(), b"");
    assert_export_counters(&export, [0, 0, 0, 0, 0, 0, 0, 0]);
}

#[test]
fn compat_http_streaming_localizes_disconnect_backpressure_and_caller_cancellation() {
    let server =
        build_phase_four_server_with_workspace_provider(oversized_streaming_provider(8, 64));

    let disconnect =
        cancel_after_first_chunk(&server, WorthServerStreamCancellationKind::ClientDisconnect);
    let backpressure = cancel_after_first_chunk(
        &server,
        WorthServerStreamCancellationKind::DownstreamBackpressure,
    );
    let caller =
        cancel_after_first_chunk(&server, WorthServerStreamCancellationKind::CallerCancelled);

    assert_cancellation_kind(
        &disconnect,
        WorthServerStreamCancellationKind::ClientDisconnect,
    );
    assert_cancellation_counters(
        &disconnect,
        [
            disconnect.chunks_emitted() as u64,
            disconnect.bytes_emitted() as u64,
            0,
            1,
            0,
            1,
            0,
            0,
        ],
    );
    assert_cancellation_kind(
        &backpressure,
        WorthServerStreamCancellationKind::DownstreamBackpressure,
    );
    assert_cancellation_counters(
        &backpressure,
        [
            backpressure.chunks_emitted() as u64,
            backpressure.bytes_emitted() as u64,
            0,
            1,
            1,
            0,
            0,
            0,
        ],
    );
    assert_cancellation_kind(&caller, WorthServerStreamCancellationKind::CallerCancelled);
    assert_cancellation_counters(
        &caller,
        [
            caller.chunks_emitted() as u64,
            caller.bytes_emitted() as u64,
            0,
            1,
            0,
            0,
            1,
            0,
        ],
    );
}

#[test]
fn compat_http_streaming_admits_background_export_when_sync_delivery_would_be_dishonest() {
    let server =
        build_phase_four_server_with_workspace_provider(oversized_streaming_provider(24, 128));
    let response = streaming_response_success(
        server.compat_http().stream(
            compat_stream_input(&server, "users.profile"),
            worth_server::WorthServerStreamSelection::incremental()
                .with_chunk_bytes(16)
                .with_background_export_threshold_bytes(200),
        ),
    );

    let background = match response {
        WorthServerStreamingResponse::BackgroundExport(value) => value,
        other => panic!("expected background export fallback, got {other:?}"),
    };

    assert!(background.estimated_payload_bytes() > 200);
    assert!(background
        .detail()
        .contains("exceeded synchronous threshold `200`"));
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.chunks_emitted"),
        Some(0)
    );
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.bytes_emitted"),
        Some(0)
    );
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.full_buffer_materializations"),
        Some(0)
    );
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.first_chunk_without_full_buffer"),
        Some(0)
    );
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.backpressure_events"),
        Some(0)
    );
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.disconnects"),
        Some(0)
    );
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.cancellations"),
        Some(0)
    );
    assert_eq!(
        background
            .performance_receipt()
            .counter("compat_http.streaming.background_export_fallbacks"),
        Some(1)
    );
}

#[test]
fn compat_http_streaming_requires_the_streaming_route_family() {
    let server = build_phase_four_server();
    let denial = match server.compat_http().stream(
        compat_read_input(&server, "users.profile"),
        default_stream_selection(),
    ) {
        TransitionOutcome::Denied(value) => value,
        other => panic!("expected compatibility streaming denial, got {other:?}"),
    };

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityStreamingRequestInvalid
    );
}

fn expect_stream(
    response: WorthServerStreamingResponse,
) -> worth_server::WorthServerCompatibilityStream {
    match response {
        WorthServerStreamingResponse::Stream(value) => value,
        other => panic!("expected incremental stream, got {other:?}"),
    }
}

fn cancel_after_first_chunk(
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
        WorthServerStreamCancellationKind::ClientDisconnect => stream.abort_due_to_disconnect(),
        WorthServerStreamCancellationKind::DownstreamBackpressure => {
            stream.abort_due_to_backpressure()
        }
        WorthServerStreamCancellationKind::CallerCancelled => stream.cancel_by_caller(),
    }
}

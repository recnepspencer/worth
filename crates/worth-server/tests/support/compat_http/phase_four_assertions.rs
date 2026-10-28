use worth_server::{
    WorthServerCompatibilityExport, WorthServerCompatibilityRead, WorthServerCompatibilityStream,
    WorthServerStreamCancellationKind, WorthServerStreamCancellationReceipt,
};

const STREAMING_COUNTER_NAMES: [&str; 8] = [
    "compat_http.streaming.chunks_emitted",
    "compat_http.streaming.bytes_emitted",
    "compat_http.streaming.full_buffer_materializations",
    "compat_http.streaming.first_chunk_without_full_buffer",
    "compat_http.streaming.backpressure_events",
    "compat_http.streaming.disconnects",
    "compat_http.streaming.cancellations",
    "compat_http.streaming.background_export_fallbacks",
];

pub(crate) fn collect_stream_bytes(
    stream: &mut WorthServerCompatibilityStream,
) -> Result<(Vec<u8>, usize), serde_json::Error> {
    let mut bytes = Vec::new();
    let mut chunks = 0usize;
    while let Some(chunk) = stream.next_chunk()? {
        chunks += 1;
        bytes.extend_from_slice(chunk.bytes());
    }
    Ok((bytes, chunks))
}

pub(crate) fn assert_read_artifact_parity(
    left: &WorthServerCompatibilityRead,
    right: &WorthServerCompatibilityRead,
) {
    assert_eq!(left.handoff_digest(), right.handoff_digest());
    assert_eq!(left.declaration_digest(), right.declaration_digest());
    assert_eq!(
        left.direct_context().basis_digest(),
        right.direct_context().basis_digest()
    );
    assert_eq!(
        left.validator().entity_tag(),
        right.validator().entity_tag()
    );
    assert_eq!(
        left.response_envelope().canonical_digest(),
        right.response_envelope().canonical_digest()
    );
    assert_eq!(
        left.cache_policy().canonical_digest(),
        right.cache_policy().canonical_digest()
    );
}

pub(crate) fn assert_export_counter(
    export: &WorthServerCompatibilityExport,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        export.performance_receipt().counter(name),
        Some(expected),
        "expected export counter `{name}` to equal `{expected}`"
    );
}

pub(crate) fn assert_export_counters(export: &WorthServerCompatibilityExport, expected: [u64; 8]) {
    for (name, expected_value) in STREAMING_COUNTER_NAMES.iter().zip(expected) {
        assert_export_counter(export, name, expected_value);
    }
}

pub(crate) fn assert_cancellation_counter(
    receipt: &WorthServerStreamCancellationReceipt,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        receipt.performance_receipt().counter(name),
        Some(expected),
        "expected cancellation counter `{name}` to equal `{expected}`"
    );
}

pub(crate) fn assert_cancellation_counters(
    receipt: &WorthServerStreamCancellationReceipt,
    expected: [u64; 8],
) {
    for (name, expected_value) in STREAMING_COUNTER_NAMES.iter().zip(expected) {
        assert_cancellation_counter(receipt, name, expected_value);
    }
}

pub(crate) fn assert_cancellation_kind(
    receipt: &WorthServerStreamCancellationReceipt,
    expected: WorthServerStreamCancellationKind,
) {
    assert_eq!(receipt.kind(), expected);
    assert!(!receipt.transport_completed());
    assert!(receipt.canonical_result_completed());
}

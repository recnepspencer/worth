use worth_server::{
    WorthServerBinaryCounterSet, WorthServerCompatibilityExport, WorthServerCompatibilityStream,
    WorthServerExternalCounterSet,
};

pub(crate) fn assert_external_counter(
    counters: &WorthServerExternalCounterSet,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        counters.counter(name),
        Some(expected),
        "expected external counter `{name}` to equal {expected}",
    );
}

pub(crate) fn assert_binary_counter(
    counters: &WorthServerBinaryCounterSet,
    name: &str,
    expected: u64,
) {
    assert_eq!(
        counters.counter(name),
        Some(expected),
        "expected binary counter `{name}` to equal {expected}",
    );
}

pub(crate) fn finish_stream(
    mut stream: WorthServerCompatibilityStream,
) -> WorthServerCompatibilityExport {
    while stream
        .next_chunk()
        .expect("stream chunk should serialize")
        .is_some()
    {}
    stream
        .finish()
        .expect("stream should finish after full consumption")
}

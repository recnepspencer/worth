use worth_server::{WorthServerBinaryDownload, WorthServerQueryHandoffDenial};

pub(crate) fn assert_download_denial(
    denial: &WorthServerQueryHandoffDenial,
    expected_code: worth_server::WorthServerQueryHandoffDenialCode,
    detail_fragment: &str,
) {
    assert_eq!(denial.code(), expected_code);
    assert!(
        denial.detail().contains(detail_fragment),
        "expected denial detail `{}` to contain `{detail_fragment}`",
        denial.detail()
    );
}

pub(crate) fn assert_counter(
    receipt: &worth_server::WorthServerBinaryEgressPerformanceReceipt,
    name: &str,
    expected: u64,
) {
    assert_eq!(receipt.counter(name), Some(expected));
}

pub(crate) fn assert_metadata_parity(
    left: &WorthServerBinaryDownload,
    right: &WorthServerBinaryDownload,
) {
    assert_eq!(
        left.read().canonical_digest(),
        right.read().canonical_digest()
    );
    assert_eq!(
        left.read().direct_context().canonical_digest(),
        right.read().direct_context().canonical_digest()
    );
    assert_eq!(
        left.read().response_envelope().canonical_digest(),
        right.read().response_envelope().canonical_digest()
    );
    assert_eq!(
        left.read().support_posture(),
        right.read().support_posture()
    );
}

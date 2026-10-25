use worth_server::{
    WorthServerBinaryDownload, WorthServerBinaryIntegrityDigest, WorthServerQueryHandoffDenial,
};

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

pub(crate) fn assert_integrity_matches(
    left: &WorthServerBinaryIntegrityDigest,
    right: &WorthServerBinaryIntegrityDigest,
) {
    assert_eq!(
        left.full_representation_digest(),
        right.full_representation_digest()
    );
    assert_eq!(left.validator_entity_tag(), right.validator_entity_tag());
    assert_eq!(left.total_bytes(), right.total_bytes());
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
        left.read().response_envelope().canonical_digest(),
        right.read().response_envelope().canonical_digest()
    );
}

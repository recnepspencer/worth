use worth_server::{
    WorthServerIngressPerformanceReceipt, WorthServerQueryHandoffDenial,
    WorthServerQueryHandoffDenialCode, WorthServerUploadCleanupReason,
    WorthServerUploadCleanupReceipt,
};

pub(crate) fn assert_cleanup_receipt(
    receipt: &WorthServerUploadCleanupReceipt,
    expected_reason: WorthServerUploadCleanupReason,
) {
    assert_eq!(receipt.reason(), expected_reason);
    assert!(receipt.truth_drift_free());
    assert!(!receipt.session_digest().is_empty());
    assert!(!receipt.tenant_id().is_empty());
    assert!(!receipt.workspace_digest().is_empty());
    assert!(!receipt.branch_digest().is_empty());
    assert!(receipt.canonical_digest().contains("truth_drift_free=true"));
}

pub(crate) fn assert_counter(
    receipt: &WorthServerIngressPerformanceReceipt,
    name: &str,
    expected: u64,
) {
    assert_eq!(receipt.counter(name), Some(expected), "counter `{name}`");
}

pub(crate) fn assert_ingress_counters(
    receipt: &WorthServerIngressPerformanceReceipt,
    expected_wire_bytes: u64,
    expected_authoritative_bytes: u64,
    expected_unknown_length_parts: u64,
    expected_compressed_parts: u64,
    expected_chunks_observed: u64,
) {
    assert_counter(receipt, "compat_http.upload.ingress_sessions_started", 1);
    assert_counter(receipt, "compat_http.upload.ingress_parts_processed", 1);
    assert_counter(
        receipt,
        "compat_http.upload.ingress_wire_bytes",
        expected_wire_bytes,
    );
    assert_counter(
        receipt,
        "compat_http.upload.ingress_authoritative_bytes",
        expected_authoritative_bytes,
    );
    assert_counter(
        receipt,
        "compat_http.upload.ingress_unknown_length_parts",
        expected_unknown_length_parts,
    );
    assert_counter(
        receipt,
        "compat_http.upload.ingress_compressed_parts",
        expected_compressed_parts,
    );
    assert_counter(
        receipt,
        "compat_http.upload.ingress_chunks_observed",
        expected_chunks_observed,
    );
    assert_counter(receipt, "compat_http.upload.cleanup_operations", 0);
    assert_counter(receipt, "compat_http.upload.cleanup_staged_bytes", 0);
}

pub(crate) fn assert_upload_denial(
    denial: &WorthServerQueryHandoffDenial,
    expected_code: WorthServerQueryHandoffDenialCode,
    required_detail: &str,
) {
    assert_eq!(denial.code(), expected_code);
    assert!(
        denial.detail().contains(required_detail),
        "expected denial detail to contain `{required_detail}`, got `{}`",
        denial.detail()
    );
}

pub(crate) fn stable_digest(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(bytes);
    format!("sha256:{}:{digest:x}", bytes.len())
}

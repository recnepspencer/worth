pub(crate) use serde_json::json;
pub(crate) use std::io::Write;
pub(crate) use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub(crate) use {flate2::write::GzEncoder, flate2::Compression};
pub(crate) use {
    worth_server::WorthServerQueryHandoffDenialCode,
    worth_server::WorthServerStreamCancellationKind, worth_server::WorthServerUploadChunk,
    worth_server::WorthServerUploadContentEncoding, worth_server::WorthServerUploadExpectation,
    worth_server::WorthServerUploadManifest, worth_server::WorthServerUploadPart,
};

pub(crate) use {
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_binary_counter,
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_denial_contains,
    crate::shared_support_compat_http_phase_thirteen_assertions::assert_external_counter,
    crate::shared_support_compat_http_phase_thirteen_runtime::cancellation_receipt,
    crate::shared_support_compat_http_phase_thirteen_runtime::phase_thirteen_server,
    worth_testkit_server::shared_support_compat_http_phase_eight_runtime as compat_http_phase_eight_runtime,
    worth_testkit_server::shared_support_compat_http_phase_six_runtime as compat_http_phase_six_runtime,
    worth_testkit_server::shared_support_compat_http_phase_twelve_runtime as compat_http_phase_twelve_runtime,
    worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime,
};

pub(crate) fn compression_ratio_abuse_upload() -> worth_server::WorthServerMultipartUpload {
    let authoritative_bytes = vec![3u8; 4_096];
    let compressed_bytes = gzip_bytes(&authoritative_bytes);
    let part = worth_server::WorthServerUploadPart::file("avatar")
        .with_content_type("application/gzip")
        .with_declared_length(authoritative_bytes.len() as u64)
        .with_content_encoding(worth_server::WorthServerUploadContentEncoding::Gzip)
        .with_body_bytes(authoritative_bytes)
        .with_wire_chunk(worth_server::WorthServerUploadChunk::new(compressed_bytes));
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": "compressed-phase-thirteen",
                    "title.value": "Compressed Phase Thirteen"
                }
            }
        }))
        .with_file_part("avatar"),
    )
    .with_expectation(worth_server::WorthServerUploadExpectation::continue_optional())
    .with_part(part)
}

pub(crate) fn staged_cleanup_upload() -> worth_server::WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": "phase-thirteen-cleanup",
                    "title.value": "Phase Thirteen Cleanup"
                }
            }
        }))
        .with_file_part("blob"),
    )
    .with_part(
        worth_server::WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(11)
            .with_body_bytes(b"hello-world".to_vec()),
    )
}

pub(crate) fn gzip_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(bytes)
        .expect("gzip encoder should accept payload bytes");
    encoder
        .finish()
        .expect("gzip encoder should finish payload")
}

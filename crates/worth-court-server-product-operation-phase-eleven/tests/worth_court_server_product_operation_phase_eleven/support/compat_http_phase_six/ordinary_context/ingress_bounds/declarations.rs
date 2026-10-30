pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub(crate) use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
pub(crate) use {flate2::write::GzEncoder, flate2::Compression};
pub(crate) use {
    worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerUploadChunk,
    worth_server::WorthServerUploadContentEncoding, worth_server::WorthServerUploadManifest,
    worth_server::WorthServerUploadPart, worth_server::WorthServerUploadTransferMode,
};

pub(crate) use crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_upload_denial;
pub(crate) use crate::compat_http_phase_six::ordinary_context::upload_fixtures::single_insert_body;
pub(crate) use worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider;
pub(crate) use {
    worth_testkit_server::shared_support_compat_http_phase_six_runtime::build_phase_six_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_denied,
    worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_execution_input,
};

pub(crate) fn drip_fed_upload() -> worth_server::WorthServerMultipartUpload {
    let part = (0..33).fold(
        worth_server::WorthServerUploadPart::file("avatar")
            .with_content_type("image/png")
            .with_declared_length(33)
            .with_transfer_mode(worth_server::WorthServerUploadTransferMode::UnknownLength)
            .with_body_bytes(vec![b'a'; 33]),
        |part, _| part.with_wire_chunk(worth_server::WorthServerUploadChunk::new(vec![1])),
    );
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(single_insert_body("drip"))
            .with_file_part("avatar"),
    )
    .with_part(part)
}

pub(crate) fn oversized_unknown_length_upload() -> worth_server::WorthServerMultipartUpload {
    let authoritative_bytes = vec![7u8; 1_024];
    let part = (0..9).fold(
        worth_server::WorthServerUploadPart::file("avatar")
            .with_content_type("application/octet-stream")
            .with_declared_length(authoritative_bytes.len() as u64)
            .with_transfer_mode(worth_server::WorthServerUploadTransferMode::UnknownLength)
            .with_content_encoding(worth_server::WorthServerUploadContentEncoding::Gzip)
            .with_body_bytes(authoritative_bytes),
        |part, _| {
            part.with_wire_chunk(worth_server::WorthServerUploadChunk::new(vec![
                9u8;
                2 * 1024
                    * 1024
            ]))
        },
    );
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(single_insert_body("unknown"))
            .with_file_part("avatar"),
    )
    .with_part(part)
}

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
        worth_server::WorthServerUploadManifest::new(single_insert_body("compressed"))
            .with_file_part("avatar"),
    )
    .with_part(part)
}

pub(crate) fn gzip_bytes(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;

    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder
        .write_all(bytes)
        .expect("gzip encoder should accept payload bytes");
    encoder
        .finish()
        .expect("gzip encoder should finish payload")
}

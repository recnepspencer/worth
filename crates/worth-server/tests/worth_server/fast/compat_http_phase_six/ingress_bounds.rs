use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use flate2::{write::GzEncoder, Compression};
use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerQueryHandoffDenialCode, WorthServerUploadChunk, WorthServerUploadContentEncoding,
    WorthServerUploadManifest, WorthServerUploadPart, WorthServerUploadTransferMode,
};

use super::compat_http_phase_six_assertions::assert_upload_denial;
use super::compat_http_phase_six_runtime::{
    build_phase_six_server_with_workspace_provider, compat_upload_denied,
    compat_upload_execution_input,
};
use super::query_handoff_runtime::ProfiledCountingTestWorkspaceProvider;
use super::upload_fixtures::single_insert_body;

#[test]
fn compat_http_chunked_and_compressed_ingress_enforces_exact_bounds() {
    let attempted_writes = Arc::new(AtomicUsize::new(0));
    let server =
        build_phase_six_server_with_workspace_provider(ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ));

    let drip_denial =
        compat_upload_denied(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.drip.upload",
            "boundary-drip",
            drip_fed_upload(),
        )));
    let unknown_length_denial =
        compat_upload_denied(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.unknown.upload",
            "boundary-unknown",
            oversized_unknown_length_upload(),
        )));
    let compression_denial =
        compat_upload_denied(server.compat_http().upload(compat_upload_execution_input(
            &server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.compressed.upload",
            "boundary-compressed",
            compression_ratio_abuse_upload(),
        )));

    assert_upload_denial(
        &drip_denial,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "chunk pacing cap",
    );
    assert_upload_denial(
        &unknown_length_denial,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "without a known content length",
    );
    assert_upload_denial(
        &compression_denial,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "decompression ratio cap",
    );
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "ingress hostility denials must occur before authoritative writes"
    );
}

fn drip_fed_upload() -> worth_server::WorthServerMultipartUpload {
    let part = (0..33).fold(
        WorthServerUploadPart::file("avatar")
            .with_content_type("image/png")
            .with_declared_length(33)
            .with_transfer_mode(WorthServerUploadTransferMode::UnknownLength)
            .with_body_bytes(vec![b'a'; 33]),
        |part, _| part.with_wire_chunk(WorthServerUploadChunk::new(vec![1])),
    );
    worth_server::WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(single_insert_body("drip")).with_file_part("avatar"),
    )
    .with_part(part)
}

fn oversized_unknown_length_upload() -> worth_server::WorthServerMultipartUpload {
    let authoritative_bytes = vec![7u8; 1_024];
    let part = (0..9).fold(
        WorthServerUploadPart::file("avatar")
            .with_content_type("application/octet-stream")
            .with_declared_length(authoritative_bytes.len() as u64)
            .with_transfer_mode(WorthServerUploadTransferMode::UnknownLength)
            .with_content_encoding(WorthServerUploadContentEncoding::Gzip)
            .with_body_bytes(authoritative_bytes),
        |part, _| part.with_wire_chunk(WorthServerUploadChunk::new(vec![9u8; 2 * 1024 * 1024])),
    );
    worth_server::WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(single_insert_body("unknown")).with_file_part("avatar"),
    )
    .with_part(part)
}

fn compression_ratio_abuse_upload() -> worth_server::WorthServerMultipartUpload {
    let authoritative_bytes = vec![3u8; 4_096];
    let compressed_bytes = gzip_bytes(&authoritative_bytes);
    let part = WorthServerUploadPart::file("avatar")
        .with_content_type("application/gzip")
        .with_declared_length(authoritative_bytes.len() as u64)
        .with_content_encoding(WorthServerUploadContentEncoding::Gzip)
        .with_body_bytes(authoritative_bytes)
        .with_wire_chunk(WorthServerUploadChunk::new(compressed_bytes));
    worth_server::WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(single_insert_body("compressed")).with_file_part("avatar"),
    )
    .with_part(part)
}

fn gzip_bytes(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(bytes)
        .expect("gzip encoder should accept payload bytes");
    encoder
        .finish()
        .expect("gzip encoder should finish payload")
}

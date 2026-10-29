#[allow(unused_imports)] use crate::compat_http_phase_six::ordinary_context::ingress_bounds::*;
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

use {flate2::write::GzEncoder, flate2::Compression};
use {worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
use {worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerUploadChunk, worth_server::WorthServerUploadContentEncoding, worth_server::WorthServerUploadManifest, worth_server::WorthServerUploadPart, worth_server::WorthServerUploadTransferMode};

use {crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_upload_denial};
use {worth_testkit_server::shared_support_compat_http_phase_six_runtime::build_phase_six_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_denied, worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_execution_input};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider};
use {crate::compat_http_phase_six::ordinary_context::upload_fixtures::single_insert_body};

#[test]
fn compat_http_chunked_and_compressed_ingress_enforces_exact_bounds() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server =
        build_phase_six_server_with_workspace_provider(worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
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
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "chunk pacing cap",
    );
    assert_upload_denial(
        &unknown_length_denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "without a known content length",
    );
    assert_upload_denial(
        &compression_denial,
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "decompression ratio cap",
    );
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "ingress hostility denials must occur before authoritative writes"
    );
}

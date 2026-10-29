use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_query::facade::runtime::WorthQueryRuntimeSupportProfile;
use worth_server::{
    WorthServerQueryHandoffDenialCode, WorthServerUploadManifest, WorthServerUploadPart,
};

use super::compat_http_phase_six_assertions::{
    assert_ingress_counters, assert_upload_denial, stable_digest,
};
use super::compat_http_phase_six_runtime::{
    build_phase_six_server_with_workspace_provider, compat_upload_denied,
    compat_upload_execution_input, compat_upload_success,
};
use super::query_handoff_runtime::ProfiledCountingTestWorkspaceProvider;
use super::upload_fixtures::single_insert_body;

#[test]
fn compat_http_upload_integrity_preserves_exact_digests_and_blocks_mismatch_before_commit() {
    let success_writes = Arc::new(AtomicUsize::new(0));
    let success_server =
        build_phase_six_server_with_workspace_provider(ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            success_writes.clone(),
        ));
    let success_upload = integrity_honest_upload();
    let expected_manifest_digest =
        stable_digest(success_upload.manifest().integrity_basis().as_bytes());
    let expected_avatar_digest = stable_digest(b"avatar-phase-six");
    let success = compat_upload_success(success_server.compat_http().upload(
        compat_upload_execution_input(
            &success_server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.integrity.upload",
            "boundary-integrity",
            success_upload,
        ),
    ));

    assert_eq!(
        success.ingress_integrity().manifest_digest(),
        expected_manifest_digest
    );
    assert_eq!(
        success.ingress_integrity().part_digest("avatar"),
        Some(expected_avatar_digest.as_str())
    );
    assert_ingress_counters(success.ingress_performance(), 16, 16, 0, 0, 1);
    assert_eq!(success_writes.load(Ordering::Relaxed), 1);

    let failed_writes = Arc::new(AtomicUsize::new(0));
    let failed_server =
        build_phase_six_server_with_workspace_provider(ProfiledCountingTestWorkspaceProvider::new(
            WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            failed_writes.clone(),
        ));
    let denial = compat_upload_denied(failed_server.compat_http().upload(
        compat_upload_execution_input(
            &failed_server,
            "tenant-a",
            "workspace-42",
            "branch-9",
            "files.integrity.upload",
            "boundary-integrity-denied",
            integrity_mismatched_upload(),
        ),
    ));

    assert_upload_denial(
        &denial,
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid,
        "integrity digest mismatch",
    );
    assert_eq!(
        failed_writes.load(Ordering::Relaxed),
        0,
        "integrity mismatch must deny before metadata truth commits"
    );
}

fn integrity_honest_upload() -> worth_server::WorthServerMultipartUpload {
    let manifest =
        WorthServerUploadManifest::new(single_insert_body("integrity")).with_file_part("avatar");
    let manifest_digest = stable_digest(manifest.integrity_basis().as_bytes());
    let avatar_bytes = b"avatar-phase-six".to_vec();
    worth_server::WorthServerMultipartUpload::new(manifest.with_integrity_digest(manifest_digest))
        .with_part(
            WorthServerUploadPart::file("avatar")
                .with_content_type("image/png")
                .with_declared_length(avatar_bytes.len() as u64)
                .with_body_bytes(avatar_bytes.clone())
                .with_integrity_digest(stable_digest(&avatar_bytes)),
        )
}

fn integrity_mismatched_upload() -> worth_server::WorthServerMultipartUpload {
    let avatar_bytes = b"avatar-phase-six".to_vec();
    worth_server::WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(single_insert_body("integrity-bad"))
            .with_file_part("avatar")
            .with_integrity_digest("wrong-manifest-digest"),
    )
    .with_part(
        WorthServerUploadPart::file("avatar")
            .with_content_type("image/png")
            .with_declared_length(avatar_bytes.len() as u64)
            .with_body_bytes(avatar_bytes)
            .with_integrity_digest("wrong-part-digest"),
    )
}

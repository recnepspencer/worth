pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub(crate) use {worth_query::facade::runtime::WorthQueryRuntimeSupportProfile};
pub(crate) use {worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerUploadManifest, worth_server::WorthServerUploadPart};

pub(crate) use {crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_ingress_counters, crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::assert_upload_denial, crate::compat_http_phase_six::ordinary_context::compat_http_phase_six_assertions::stable_digest};
pub(crate) use {worth_testkit_server::shared_support_compat_http_phase_six_runtime::build_phase_six_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_denied, worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_execution_input, worth_testkit_server::shared_support_compat_http_phase_six_runtime::compat_upload_success};
pub(crate) use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider};
pub(crate) use {crate::compat_http_phase_six::ordinary_context::upload_fixtures::single_insert_body};

pub(crate) fn integrity_honest_upload() -> worth_server::WorthServerMultipartUpload {
    let manifest =
        worth_server::WorthServerUploadManifest::new(single_insert_body("integrity")).with_file_part("avatar");
    let manifest_digest = stable_digest(manifest.integrity_basis().as_bytes());
    let avatar_bytes = b"avatar-phase-six".to_vec();
    worth_server::WorthServerMultipartUpload::new(manifest.with_integrity_digest(manifest_digest))
        .with_part(
            worth_server::WorthServerUploadPart::file("avatar")
                .with_content_type("image/png")
                .with_declared_length(avatar_bytes.len() as u64)
                .with_body_bytes(avatar_bytes.clone())
                .with_integrity_digest(stable_digest(&avatar_bytes)),
        )
}

pub(crate) fn integrity_mismatched_upload() -> worth_server::WorthServerMultipartUpload {
    let avatar_bytes = b"avatar-phase-six".to_vec();
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(single_insert_body("integrity-bad"))
            .with_file_part("avatar")
            .with_integrity_digest("wrong-manifest-digest"),
    )
    .with_part(
        worth_server::WorthServerUploadPart::file("avatar")
            .with_content_type("image/png")
            .with_declared_length(avatar_bytes.len() as u64)
            .with_body_bytes(avatar_bytes)
            .with_integrity_digest("wrong-part-digest"),
    )
}
use serde_json::json;
use worth_server::{
    WorthServerMultipartUpload, WorthServerQueryHandoffDenialCode, WorthServerUploadManifest,
    WorthServerUploadPart,
};

use super::compat_http_phase_five_runtime::{
    build_phase_five_server, compat_upload_denied, compat_upload_execution_input, manifest_for,
    prepared_request, single_insert_body, upload_order_alpha,
};
use super::upload_input::upload_input_with_content_type;

#[test]
fn compat_http_upload_rejects_malformed_part_graphs_and_unsupported_content_shapes() {
    let server = build_phase_five_server();
    let missing_part = compat_upload_denied(
        server.compat_http().upload(compat_upload_execution_input(
            &server,
            "files.avatar.upload",
            "boundary-a",
            WorthServerMultipartUpload::new(manifest_for("task-1")).with_part(
                WorthServerUploadPart::file("avatar")
                    .with_content_type("image/png")
                    .with_declared_length(128),
            ),
        )),
    );
    let missing_metadata = compat_upload_denied(
        server.compat_http().upload(compat_upload_execution_input(
            &server,
            "files.avatar.upload",
            "boundary-metadata",
            WorthServerMultipartUpload::new(
                WorthServerUploadManifest::new(json!({}))
                    .with_file_part("avatar")
                    .with_file_part("thumbnail"),
            )
            .with_part(
                WorthServerUploadPart::file("avatar")
                    .with_content_type("image/png")
                    .with_declared_length(128),
            )
            .with_part(
                WorthServerUploadPart::file("thumbnail")
                    .with_content_type("image/webp")
                    .with_declared_length(64),
            ),
        )),
    );
    let duplicate_part_identity = compat_upload_denied(
        server.compat_http().upload(compat_upload_execution_input(
            &server,
            "files.avatar.upload",
            "boundary-duplicate",
            WorthServerMultipartUpload::new(
                WorthServerUploadManifest::new(single_insert_body("task-1"))
                    .with_file_part("avatar")
                    .with_file_part("avatar"),
            )
            .with_part(
                WorthServerUploadPart::file("avatar")
                    .with_content_type("image/png")
                    .with_declared_length(128),
            ),
        )),
    );
    let oversized_part = compat_upload_denied(
        server.compat_http().upload(compat_upload_execution_input(
            &server,
            "files.avatar.upload",
            "boundary-b",
            WorthServerMultipartUpload::new(manifest_for("task-1"))
                .with_part(
                    WorthServerUploadPart::file("avatar")
                        .with_content_type("image/png")
                        .with_declared_length(128),
                )
                .with_part(
                    WorthServerUploadPart::file("thumbnail")
                        .with_content_type("image/webp")
                        .with_declared_length(9 * 1024 * 1024),
                ),
        )),
    );
    let wrong_content_type = compat_upload_denied(
        server.compat_http().upload(
            worth_server::WorthServerCompatibilityUploadExecutionInput::new(
                prepared_request(
                    &server,
                    upload_input_with_content_type("files.avatar.upload", "application/json")
                        .build()
                        .expect("wrong-content-type upload input should validate structurally"),
                ),
                "files.avatar.upload",
                upload_order_alpha("task-1"),
            ),
        ),
    );

    assert_eq!(
        missing_part.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(missing_part
        .detail()
        .contains("part graph did not match the declared manifest file-part set"));
    assert_eq!(
        missing_metadata.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityMutationRequestInvalid
    );
    assert!(missing_metadata
        .detail()
        .contains("must define `command` or `commands`"));
    assert_eq!(
        duplicate_part_identity.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(duplicate_part_identity
        .detail()
        .contains("manifest file part names may not repeat"));
    assert_eq!(
        oversized_part.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(oversized_part
        .detail()
        .contains("exceeds the phase-five early-admission cap"));
    assert_eq!(
        wrong_content_type.code(),
        WorthServerQueryHandoffDenialCode::CompatibilityUploadRequestInvalid
    );
    assert!(wrong_content_type
        .detail()
        .contains("expected multipart/form-data"));
}

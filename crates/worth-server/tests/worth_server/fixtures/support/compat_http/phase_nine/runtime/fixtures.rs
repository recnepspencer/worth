use serde_json::json;
use worth_server::{WorthServerMultipartUpload, WorthServerUploadManifest, WorthServerUploadPart};

pub(crate) fn canonical_upload(identity: &str) -> WorthServerMultipartUpload {
    WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": {
                    "identity.id": identity,
                    "title.value": format!("Title for {identity}")
                }
            }
        }))
        .with_file_part("blob"),
    )
    .with_part(
        WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(11)
            .with_body_bytes(b"hello-world".to_vec()),
    )
}

pub(crate) fn malformed_upload() -> WorthServerMultipartUpload {
    WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": ""
            }
        }))
        .with_file_part("blob"),
    )
    .with_part(
        WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(4)
            .with_body_bytes(b"nope".to_vec()),
    )
}

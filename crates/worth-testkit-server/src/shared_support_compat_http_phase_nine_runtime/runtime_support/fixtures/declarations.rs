pub use {serde_json::json};
pub use {worth_server::WorthServerMultipartUpload, worth_server::WorthServerUploadManifest, worth_server::WorthServerUploadPart};

pub fn canonical_upload(identity: &str) -> WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
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
        worth_server::WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(11)
            .with_body_bytes(b"hello-world".to_vec()),
    )
}

pub fn malformed_upload() -> WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": ""
            }
        }))
        .with_file_part("blob"),
    )
    .with_part(
        worth_server::WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(4)
            .with_body_bytes(b"nope".to_vec()),
    )
}
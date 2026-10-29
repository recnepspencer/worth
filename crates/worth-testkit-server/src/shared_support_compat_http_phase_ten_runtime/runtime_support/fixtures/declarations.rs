pub use {serde_json::json, serde_json::Map, serde_json::Value};
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

pub fn reordered_canonical_upload(identity: &str) -> WorthServerMultipartUpload {
    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "collection": "Task",
                "aspects": {
                    "title.value": format!("Title for {identity}"),
                    "identity.id": identity
                },
                "family": "insert"
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

pub fn ambiguous_metadata_upload() -> WorthServerMultipartUpload {
    let mut aspects = serde_json::Map::new();
    aspects.insert("title.value".to_string(), serde_json::Value::String("safe".to_string()));
    aspects.insert(
        " Title.Value ".to_string(),
        serde_json::Value::String("unsafe".to_string()),
    );

    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": serde_json::Value::Object(aspects)
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

pub fn non_ascii_metadata_upload() -> WorthServerMultipartUpload {
    let mut aspects = serde_json::Map::new();
    aspects.insert(
        "t\u{00ed}tle.value".to_string(),
        serde_json::Value::String("unsafe".to_string()),
    );

    worth_server::WorthServerMultipartUpload::new(
        worth_server::WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": serde_json::Value::Object(aspects)
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
use serde_json::{json, Map, Value};
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

pub(crate) fn reordered_canonical_upload(identity: &str) -> WorthServerMultipartUpload {
    WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(json!({
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
        WorthServerUploadPart::file("blob")
            .with_content_type("application/octet-stream")
            .with_declared_length(11)
            .with_body_bytes(b"hello-world".to_vec()),
    )
}

pub(crate) fn ambiguous_metadata_upload() -> WorthServerMultipartUpload {
    let mut aspects = Map::new();
    aspects.insert("title.value".to_string(), Value::String("safe".to_string()));
    aspects.insert(
        " Title.Value ".to_string(),
        Value::String("unsafe".to_string()),
    );

    WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": Value::Object(aspects)
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

pub(crate) fn non_ascii_metadata_upload() -> WorthServerMultipartUpload {
    let mut aspects = Map::new();
    aspects.insert(
        "t\u{00ed}tle.value".to_string(),
        Value::String("unsafe".to_string()),
    );

    WorthServerMultipartUpload::new(
        WorthServerUploadManifest::new(json!({
            "command": {
                "family": "insert",
                "collection": "Task",
                "aspects": Value::Object(aspects)
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

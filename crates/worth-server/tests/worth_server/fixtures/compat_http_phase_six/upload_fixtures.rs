use serde_json::json;

pub(super) fn single_insert_body(identity: &str) -> serde_json::Value {
    json!({
        "command": {
            "family": "insert",
            "collection": "Task",
            "aspects": {
                "identity.id": identity,
                "title.value": format!("Title for {identity}")
            }
        }
    })
}

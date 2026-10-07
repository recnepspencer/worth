use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerOperationInputEnvelope {
    declared_schema_identity: Option<String>,
    payload_identity: String,
    payload_kind: &'static str,
    payload_size_bytes: usize,
    canonical_digest: String,
}

impl WorthServerOperationInputEnvelope {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::{
            option_string, string,
        };
        option_string(&self.declared_schema_identity)
            .saturating_add(string(&self.payload_identity))
            .saturating_add(string(&self.canonical_digest))
    }

    pub fn json(
        declared_schema_identity: impl Into<String>,
        payload: &Value,
    ) -> WorthServerOperationInputEnvelope {
        let declared_schema_identity = declared_schema_identity.into();
        let canonical_payload = canonicalize_json(payload);
        let payload_size_bytes = canonical_payload.len();
        let payload_identity = format!("json:{canonical_payload}");
        let canonical_digest = format!(
            "worth-server-operation-input-envelope-v1|kind=json|schema={}|payload={payload_identity}|size={payload_size_bytes}",
            declared_schema_identity.trim()
        );
        Self {
            declared_schema_identity: Some(declared_schema_identity),
            payload_identity,
            payload_kind: "json",
            payload_size_bytes,
            canonical_digest,
        }
    }

    pub fn opaque_digest(
        payload_kind: &'static str,
        declared_schema_identity: Option<impl Into<String>>,
        payload_identity: impl Into<String>,
        payload_size_bytes: usize,
    ) -> Self {
        let declared_schema_identity = declared_schema_identity.map(Into::into);
        let payload_identity = payload_identity.into();
        let canonical_digest = format!(
            "worth-server-operation-input-envelope-v1|kind={payload_kind}|schema={}|payload={payload_identity}|size={payload_size_bytes}",
            declared_schema_identity.as_deref().unwrap_or("none")
        );
        Self {
            declared_schema_identity,
            payload_identity,
            payload_kind,
            payload_size_bytes,
            canonical_digest,
        }
    }

    pub fn declared_schema_identity(&self) -> Option<&str> {
        self.declared_schema_identity.as_deref()
    }

    pub fn payload_identity(&self) -> &str {
        &self.payload_identity
    }

    pub fn payload_kind(&self) -> &'static str {
        self.payload_kind
    }

    pub fn payload_size_bytes(&self) -> usize {
        self.payload_size_bytes
    }

    pub fn canonical_digest(&self) -> &str {
        &self.canonical_digest
    }
}

fn canonicalize_json(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => format!("{value:?}"),
        Value::Array(values) => {
            let rows = values.iter().map(canonicalize_json).collect::<Vec<_>>();
            format!("[{}]", rows.join(","))
        }
        Value::Object(entries) => {
            let mut rows = entries.iter().collect::<Vec<_>>();
            rows.sort_by(|left, right| left.0.cmp(right.0));
            let rows = rows
                .into_iter()
                .map(|(name, value)| format!("{name:?}:{}", canonicalize_json(value)))
                .collect::<Vec<_>>();
            format!("{{{}}}", rows.join(","))
        }
    }
}

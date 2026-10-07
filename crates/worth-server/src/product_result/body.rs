use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerProductResultBody {
    value: Value,
    canonical_bytes: Vec<u8>,
}

impl WorthServerProductResultBody {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        crate::product_adapter::execution_pipeline::read_batch_accounting::json(&self.value)
            .saturating_add(u64::try_from(self.canonical_bytes.capacity()).unwrap_or(u64::MAX))
    }

    pub(crate) fn canonical_json(value: Value) -> Result<Self, serde_json::Error> {
        let (value, canonical_bytes) = super::canonicalize_json(value)?;
        Ok(Self {
            value,
            canonical_bytes,
        })
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }

    pub fn byte_len(&self) -> usize {
        self.canonical_bytes.len()
    }
}

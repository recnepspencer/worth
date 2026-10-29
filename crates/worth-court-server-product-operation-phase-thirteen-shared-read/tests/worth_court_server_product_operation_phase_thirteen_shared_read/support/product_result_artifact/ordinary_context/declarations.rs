pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

pub(crate) use {serde::ser::SerializeSeq, serde::Serialize, serde::Serializer};
pub(crate) use {serde_json::json};
pub(crate) use {worth_server::WorthServerProductResultArtifact, worth_server::WorthServerProductResultArtifactErrorCode, worth_server::WorthServerProductResultContract, worth_server::WorthServerProductResultValue};

#[allow(unused_imports)]
pub(crate) use {worth_testkit_server::shared_support_product_result_schema_bound_json as schema_bound_json};

pub(crate) use {worth_testkit_server::shared_support_product_result_schema_bound_json::SchemaBoundJsonResult};

pub(crate) fn publish(
    contract: &WorthServerProductResultContract,
    body: serde_json::Value,
) -> WorthServerProductResultArtifact {
    worth_server::WorthServerProductResultArtifact::publish_json(
        contract,
        &worth_testkit_server::shared_support_product_result_schema_bound_json::SchemaBoundJsonResult::v1("product.connection.result.v1", body),
    )
    .expect("schema-bound result should publish")
}

pub(crate) struct CountingOversizedResult {
    pub(crate) item_count: usize,
    pub(crate) visited_items: Arc<AtomicUsize>,
}

impl Serialize for CountingOversizedResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.item_count))?;
        for _ in 0..self.item_count {
            self.visited_items.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            sequence.serialize_element("oversized-result-item")?;
        }
        sequence.end()
    }
}

impl WorthServerProductResultValue for CountingOversizedResult {
    fn result_schema_identity(&self) -> &str {
        "product.connection.result.v1"
    }

    fn result_schema_version(&self) -> u32 {
        1
    }
}
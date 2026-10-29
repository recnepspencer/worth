use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use serde::{ser::SerializeSeq, Serialize, Serializer};
use serde_json::json;
use worth_server::{
    WorthServerProductResultArtifact, WorthServerProductResultArtifactErrorCode,
    WorthServerProductResultContract, WorthServerProductResultValue,
};

#[allow(unused_imports)]
use crate::shared_support_product_result_schema_bound_json as schema_bound_json;

use schema_bound_json::SchemaBoundJsonResult;

#[test]
fn canonical_json_order_and_semantic_change_have_exact_identity_behavior() {
    let contract =
        WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 1024)
            .expect("result contract should validate");
    let left = publish(
        &contract,
        json!({ "connection": { "state": "ready", "id": "host-7" }, "version": 1 }),
    );
    let reordered = publish(
        &contract,
        json!({ "version": 1, "connection": { "id": "host-7", "state": "ready" } }),
    );
    let changed = publish(
        &contract,
        json!({ "version": 1, "connection": { "id": "host-7", "state": "failed" } }),
    );

    assert_eq!(
        left.body().canonical_bytes(),
        reordered.body().canonical_bytes()
    );
    assert_eq!(left.body_digest(), reordered.body_digest());
    assert_eq!(left.artifact_digest(), reordered.artifact_digest());
    assert_ne!(left.body_digest(), changed.body_digest());
    assert_ne!(left.artifact_digest(), changed.artifact_digest());
    assert_eq!(left.artifact_digest().len(), 64);
}

#[test]
fn typed_result_schema_must_match_the_declared_contract_before_publication() {
    let contract =
        WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 1024)
            .expect("result contract should validate");
    let wrong_schema = SchemaBoundJsonResult::v1(
        "product.deployment.result.v1",
        json!({ "connection_id": "host-7" }),
    );
    let error = WorthServerProductResultArtifact::publish_json(&contract, &wrong_schema)
        .expect_err("typed schema mismatch must fail before artifact construction");

    assert_eq!(
        error.code(),
        WorthServerProductResultArtifactErrorCode::SchemaContractMismatch
    );
}

#[test]
fn declared_inline_budget_rejects_oversized_result_bodies() {
    let contract =
        WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 16)
            .expect("result contract should validate");
    let result = SchemaBoundJsonResult::v1(
        "product.connection.result.v1",
        json!({ "body": "this result is intentionally too large" }),
    );
    let error = WorthServerProductResultArtifact::publish_json(&contract, &result)
        .expect_err("oversized inline body must fail closed");

    assert_eq!(
        error.code(),
        WorthServerProductResultArtifactErrorCode::InlineBudgetExceeded
    );
}

#[test]
fn inline_budget_stops_serialization_before_unbounded_materialization() {
    let contract =
        WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 64)
            .expect("result contract should validate");
    let visited_items = Arc::new(AtomicUsize::new(0));
    let result = CountingOversizedResult {
        item_count: 1_000_000,
        visited_items: visited_items.clone(),
    };

    let error = WorthServerProductResultArtifact::publish_json(&contract, &result)
        .expect_err("bounded publication must stop the serializer at the inline limit");

    assert_eq!(
        error.code(),
        WorthServerProductResultArtifactErrorCode::InlineBudgetExceeded
    );
    assert!(
        visited_items.load(Ordering::Relaxed) < 100,
        "the serializer must not walk the complete oversized value"
    );
}

#[test]
fn canonical_contract_digest_frames_delimiter_bearing_schema_identity() {
    let left = WorthServerProductResultContract::canonical_json("schema|version=2", 1, 1024)
        .expect("delimiter-bearing identity should remain representable");
    let right = WorthServerProductResultContract::canonical_json("schema", 2, 1024)
        .expect("neighboring schema should validate");

    assert_ne!(left.canonical_digest(), right.canonical_digest());
    assert_eq!(left.canonical_digest().len(), 64);
    assert_eq!(right.canonical_digest().len(), 64);
}

fn publish(
    contract: &WorthServerProductResultContract,
    body: serde_json::Value,
) -> WorthServerProductResultArtifact {
    WorthServerProductResultArtifact::publish_json(
        contract,
        &SchemaBoundJsonResult::v1("product.connection.result.v1", body),
    )
    .expect("schema-bound result should publish")
}

struct CountingOversizedResult {
    item_count: usize,
    visited_items: Arc<AtomicUsize>,
}

impl Serialize for CountingOversizedResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.item_count))?;
        for _ in 0..self.item_count {
            self.visited_items.fetch_add(1, Ordering::Relaxed);
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

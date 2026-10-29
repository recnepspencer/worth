#[allow(unused_imports)] use crate::product_result_artifact::ordinary_context::*;
use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Arc};

use {serde::ser::SerializeSeq, serde::Serialize, serde::Serializer};
use {serde_json::json};
use {worth_server::WorthServerProductResultArtifact, worth_server::WorthServerProductResultArtifactErrorCode, worth_server::WorthServerProductResultContract, worth_server::WorthServerProductResultValue};

#[allow(unused_imports)]
use {worth_testkit_server::shared_support_product_result_schema_bound_json as schema_bound_json};

use {worth_testkit_server::shared_support_product_result_schema_bound_json::SchemaBoundJsonResult};

#[test]
fn canonical_json_order_and_semantic_change_have_exact_identity_behavior() {
    let contract =
        worth_server::WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 1024)
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
        worth_server::WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 1024)
            .expect("result contract should validate");
    let wrong_schema = worth_testkit_server::shared_support_product_result_schema_bound_json::SchemaBoundJsonResult::v1(
        "product.deployment.result.v1",
        json!({ "connection_id": "host-7" }),
    );
    let error = worth_server::WorthServerProductResultArtifact::publish_json(&contract, &wrong_schema)
        .expect_err("typed schema mismatch must fail before artifact construction");

    assert_eq!(
        error.code(),
        worth_server::WorthServerProductResultArtifactErrorCode::SchemaContractMismatch
    );
}

#[test]
fn declared_inline_budget_rejects_oversized_result_bodies() {
    let contract =
        worth_server::WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 16)
            .expect("result contract should validate");
    let result = worth_testkit_server::shared_support_product_result_schema_bound_json::SchemaBoundJsonResult::v1(
        "product.connection.result.v1",
        json!({ "body": "this result is intentionally too large" }),
    );
    let error = worth_server::WorthServerProductResultArtifact::publish_json(&contract, &result)
        .expect_err("oversized inline body must fail closed");

    assert_eq!(
        error.code(),
        worth_server::WorthServerProductResultArtifactErrorCode::InlineBudgetExceeded
    );
}

#[test]
fn inline_budget_stops_serialization_before_unbounded_materialization() {
    let contract =
        worth_server::WorthServerProductResultContract::canonical_json("product.connection.result.v1", 1, 64)
            .expect("result contract should validate");
    let visited_items = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let result = CountingOversizedResult {
        item_count: 1_000_000,
        visited_items: visited_items.clone(),
    };

    let error = worth_server::WorthServerProductResultArtifact::publish_json(&contract, &result)
        .expect_err("bounded publication must stop the serializer at the inline limit");

    assert_eq!(
        error.code(),
        worth_server::WorthServerProductResultArtifactErrorCode::InlineBudgetExceeded
    );
    assert!(
        visited_items.load(std::sync::atomic::Ordering::Relaxed) < 100,
        "the serializer must not walk the complete oversized value"
    );
}

#[test]
fn canonical_contract_digest_frames_delimiter_bearing_schema_identity() {
    let left = worth_server::WorthServerProductResultContract::canonical_json("schema|version=2", 1, 1024)
        .expect("delimiter-bearing identity should remain representable");
    let right = worth_server::WorthServerProductResultContract::canonical_json("schema", 2, 1024)
        .expect("neighboring schema should validate");

    assert_ne!(left.canonical_digest(), right.canonical_digest());
    assert_eq!(left.canonical_digest().len(), 64);
    assert_eq!(right.canonical_digest().len(), 64);
}

//! Real validation meaning is independent of its preparation placement.
use super::*;
use crate::transactions::data::RelationalCommitSemanticResult;

#[test]
fn original_validation_meaning_excludes_live_preparation_posture_and_refuses_its_retired_fields() {
    let host = crate::test_execution_policy::serial_request();
    let lease = test_execution_lease();
    let mut serial_validation = None;
    for execution in [
        ExecutionRequest::serial(&host),
        ExecutionRequest::leased(&lease),
    ] {
        let runtime = durable_runtime_with_test_schema();
        let committed =
            create_entity_outcome(&execution, &runtime, "posture-independent-validation");
        assert!(committed
            .validation()
            .invariant_executions()
            .iter()
            .any(|result| result.metadata().preparation_strategy().is_some()));
        let semantic = committed.semantic_result();
        assert!(!semantic.validation().is_empty());
        let bytes = rmp_serde::to_vec_named(semantic.validation()).unwrap();
        assert_eq!(
            &bytes,
            serial_validation.get_or_insert_with(|| bytes.clone())
        );

        let original = serde_json::to_value(semantic).unwrap();
        for metadata in original["validation"].as_array().unwrap() {
            assert!(metadata["metadata"].get("preparation_strategy").is_none());
            assert!(metadata["metadata"].get("preparation_failures").is_none());
        }
        for (field, value) in [
            ("preparation_strategy", serde_json::Value::Null),
            ("preparation_failures", serde_json::json!([])),
        ] {
            let mut retired = original.clone();
            retired["validation"][0]["metadata"][field] = value;
            assert!(serde_json::from_value::<RelationalCommitSemanticResult>(retired).is_err());
        }
        release_test_commit_snapshot(&runtime, &committed);
    }
}

//! Real validation meaning is independent of its preparation placement.
use super::{
    create_entity_outcome, durable_runtime, release_test_commit_snapshot, TestDurableMedium,
};
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionRequest, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::transactions::RelationalCommitSemanticResult;

#[test]
fn original_validation_meaning_excludes_live_preparation_posture_and_refuses_its_retired_fields() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(8).unwrap(),
        charged_memory_bytes: Some(512 * 1024 * 1024),
    })
    .unwrap();
    let lease = authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), None, Some(1_000_000)),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let mut serial_validation = None;
    for execution in [
        ExecutionRequest::serial(&host),
        ExecutionRequest::leased(&lease),
    ] {
        let medium = TestDurableMedium::new();
        let runtime = durable_runtime(&medium);
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

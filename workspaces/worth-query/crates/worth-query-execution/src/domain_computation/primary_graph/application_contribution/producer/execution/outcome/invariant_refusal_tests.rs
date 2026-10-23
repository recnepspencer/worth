use super::*;
use crate::domain_computation::{
    WorthQueryCustomInvariantDenial, WorthQueryInvariantExecutionFailure,
};
use worth_relational::facade::runtime::{
    CustomInvariantRefusal, CustomInvariantRuleId, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion,
};

#[test]
fn producer_denial_and_retained_clone_keep_the_original_invariant_payload() {
    let evidence = WorthQueryCustomInvariantDenial::Violation {
        identity: CustomInvariantSemanticIdentity {
            rule_id: CustomInvariantRuleId::new("producer.integrity"),
            semantic_version: CustomInvariantSemanticVersion::new(1, 0),
        },
        refusal: CustomInvariantRefusal::DeclaredValue {
            value_identity: "producer.refusal.v1".into(),
            bytes: b"{\"reason\":17}".to_vec(),
        },
    };
    let commit = graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
        graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
        WorthQueryInvariantExecutionFailure::custom_invariant(evidence.clone(), "domain violation"),
    );
    let denial = commit_receipt(
        "producer",
        WorthQueryApplicationCommitOutcome::Denied(commit),
    )
    .unwrap_err();
    assert_eq!(denial.custom_invariant_denial(), Some(&evidence));
    assert_eq!(denial.clone().custom_invariant_denial(), Some(&evidence));
    assert_eq!(
        denial.custom_invariant_denial().unwrap().refusal_bytes(),
        Some(b"{\"reason\":17}".as_slice())
    );
}

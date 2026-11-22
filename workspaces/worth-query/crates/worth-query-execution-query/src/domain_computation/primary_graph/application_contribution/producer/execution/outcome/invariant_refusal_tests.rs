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

#[test]
fn a_producer_requirement_refusal_keeps_its_receipt_basis_on_the_demand_denial() {
    use crate::domain_computation::WorthQueryInvariantExecutionDenialKind as Kind;
    use worth_relational::facade::mvcc::CustomInvariantNonApplicability;
    use worth_relational::facade::runtime::InvariantDecisionKind;
    let admission =
        graph::WorthQueryGeneratedOutputInvariantAdmissionDenial::RequiredInvariantDidNotPass {
            verdict: InvariantDecisionKind::NotApplicable,
            non_applicability: Some(CustomInvariantNonApplicability::NoApplicableTouch),
        };
    let commit = graph::WorthQueryApplicationCommitDenial::invariant_execution_denied(
        graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
        WorthQueryInvariantExecutionFailure::new(
            Kind::ProducerRequirementNotAdmitted(admission),
            "producer-required invariant did not admit the publication",
        ),
    );
    let denial = commit_receipt(
        "producer",
        WorthQueryApplicationCommitOutcome::Denied(commit),
    )
    .unwrap_err();
    assert_eq!(denial.producer_requirement_denial(), Some(admission));
    assert_eq!(
        denial.clone().producer_requirement_denial(),
        Some(admission)
    );
    assert_eq!(denial.custom_invariant_denial(), None);
}

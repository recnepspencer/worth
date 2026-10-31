//! Actual installed comparator verdicts survive the public Bridge handoff.
use crate::conditional_evaluations::{
    source_free_builder, ParityComparator, Requested, VersionCompute,
};
use std::sync::Arc;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::{
    diagnostics::{EvaluationVerdict, SuppressionReason},
    Aspect, NodeEvaluationResult, SignalConditionalArtifactReuse, SignalConditionalDecisionClass,
    SignalConditionalVersionComparator,
};
mod causal_retention;
mod exact_pair;
mod history;

#[test]
fn computed_comparator_reason_survives_owner_release_without_relabeling_reuse() {
    let mut builder = source_free_builder();
    let parts = BridgeConditionalContractParts {
        identity: Arc::from("computed-output"),
        dependency_count: 0,
        condition_dependency_ordinals: Vec::new(),
        condition: BridgeConditionalCondition::OnDemand,
        dependency_comparator: SignalConditionalVersionComparator::Exact,
        output_comparator: SignalConditionalVersionComparator::RuntimeResolved,
        artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
    };
    let lowering = builder
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: BridgeConditionalContract::new(parts),
            location: BridgeConditionalLocation::operation("computed-output"),
            dependencies: Vec::new(),
            providers: BridgeConditionalProviderSet::new()
                .trigger(Requested)
                .compute(VersionCompute)
                .output_comparator(ParityComparator),
        })
        .unwrap();
    let owner = builder.seal().unwrap();
    let baseline = owner
        .conditional_lifecycle_probe()
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes();
    let serial = worth_execution::SerialRequest::from_policy(
        &worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
            .request_policy(),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let session = owner
        .admit_conditional_evaluation(
            worth_runtime_bridge::facade::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                &basis,
            ),
            execution,
        )
        .unwrap();
    let mut retained = Vec::new();
    for (attempt, mut value, expected_class, expected_verdict, expected_versions) in [
        (
            1,
            3_u64,
            SignalConditionalDecisionClass::ComputedChanged,
            EvaluationVerdict::Recomputed,
            (0, 3),
        ),
        (
            2,
            5,
            SignalConditionalDecisionClass::ComputedRevertedClean,
            EvaluationVerdict::Suppressed {
                reason: SuppressionReason::ComparatorMatch,
            },
            (3, 3),
        ),
        (
            3,
            6,
            SignalConditionalDecisionClass::ComputedChanged,
            EvaluationVerdict::Recomputed,
            (3, 6),
        ),
    ] {
        let evidence = owner
            .execute_admitted_conditional(
                execution,
                &session,
                BridgeConditionalExecutionRequest {
                    lowering: &lowering,
                    query_binding_identity: "computed-output-query",
                    query_capability_identity: 1,
                    snapshot_identity: "source-free",
                    truth_branch_identity: None,
                    bridge_snapshot_identity: None,
                    execution_identity: "computed-output",
                    attempt,
                },
                &mut value,
            )
            .unwrap();
        let causal = evidence.retain_causal_observation();
        assert_eq!(causal.signal_class(), expected_class);
        assert_eq!(causal.signal_output_versions(), expected_versions);
        assert_eq!(
            causal.computed_evaluation().unwrap().verdict,
            expected_verdict
        );
        assert!(
            std::ptr::eq(
                causal.computed_evaluation().unwrap(),
                evidence.signal().computed_evaluation().unwrap()
            ),
            "Bridge borrows the genuine owner report"
        );
        assert_eq!(evidence.signal().counters().compute_contacts, 1);
        drop(evidence);
        assert_eq!(
            causal.computed_evaluation().unwrap().verdict,
            expected_verdict
        );
        retained.push(causal);
    }
    assert!(!retained[0].same_observation(&retained[1]));
    assert_ne!(
        retained[0].signal_projection().label(),
        retained[1].signal_projection().label()
    );
    drop(session);
    drop(retained);
    assert_eq!(
        owner
            .conditional_lifecycle_probe()
            .bridge_conditional_retention()
            .unwrap()
            .retained_bytes(),
        baseline
    );
}

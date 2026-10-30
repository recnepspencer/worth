//! Actual installed comparator verdicts survive the read-only Bridge handoff.
use super::*;
use crate::facade::{
    BridgeConditionalComparatorProvider, BridgeConditionalCondition,
    BridgeConditionalContractParts, BridgeConditionalExecutionRequest,
    BridgeConditionalTriggerProvider, BridgeOwnedConditionalInstallationRequest,
};
use worth_signal::facade::{
    diagnostics::{EvaluationVerdict, SuppressionReason},
    SignalConditionalArtifactReuse, SignalConditionalDecisionClass,
    SignalConditionalVersionComparator,
};

struct VersionCompute;
struct ParityComparator;
struct Requested;
mod exact_pair;
macro_rules! stateless_semantics {
    ($Provider:ty, $identity:literal) => {
        impl BridgeConditionalProviderSemantics for $Provider {
            type SemanticContract = &'static str;
            fn semantic_contract(&self) -> Self::SemanticContract {
                $identity
            }
            fn retained_heap_bytes(
                &self,
                _: &Self::SemanticContract,
            ) -> Result<
                crate::facade::BridgeConditionalProviderHeapRetention,
                crate::facade::BridgeConditionalProviderRetentionOverflow,
            > {
                Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
            }
        }
    };
}
stateless_semantics!(VersionCompute, "actual-version-compute-v1");
stateless_semantics!(ParityComparator, "parity-equivalence-v1");
stateless_semantics!(Requested, "explicit-output-evaluation-v1");
impl BridgeConditionalTriggerProvider for Requested {
    fn requested(&self) -> bool {
        true
    }
}
impl BridgeConditionalComputeProvider for VersionCompute {
    fn compute(&self, context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        let value = context
            .downcast_ref::<u64>()
            .ok_or("missing computed output")?;
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(Aspect::new(0), *value)]),
        ))
    }
}
impl BridgeConditionalComparatorProvider for ParityComparator {
    fn has_meaningful_change(&self, _: Aspect, cached: u64, current: u64) -> Result<bool, String> {
        Ok(cached % 2 != current % 2)
    }
}

#[test]
fn computed_comparator_reason_survives_owner_release_without_relabeling_reuse() {
    let mut builder = owned_runtime_builder(runtime(exact_mapping(), Vec::new())).unwrap();
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
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let basis = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let session = owner
        .admit_conditional_evaluation(
            crate::facade::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
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

//! Exact pairs get a private admission, never an ambient warm baseline.
use super::*;
use crate::facade::{
    BridgeConditionalSignalBasisBinding, BridgeExactOutputComparison, BridgeOutputComparisonDenial,
    BridgeOutputComparisonStage,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Fault {
    None,
    WrongBaseline,
    CandidateFailure,
}
struct Compute(Fault);
impl BridgeConditionalProviderSemantics for Compute {
    type SemanticContract = Fault;
    fn semantic_contract(&self) -> Fault {
        self.0
    }
    fn retained_heap_bytes(
        &self,
        _: &Fault,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}
impl BridgeConditionalComputeProvider for Compute {
    fn compute(&self, context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        let mut value = *context
            .downcast_ref::<u64>()
            .ok_or("missing actual version")?;
        match self.0 {
            Fault::WrongBaseline => {
                assert_ne!(
                    value, 43,
                    "a refused baseline must not execute its candidate"
                );
                value += 1;
            }
            Fault::CandidateFailure if value == 43 => {
                return Err("actual candidate failure".to_owned())
            }
            _ => {}
        }
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(Aspect::new(0), value)]),
        ))
    }
}

fn install(
    fault: Fault,
) -> (
    crate::facade::BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    let mut builder = owned_runtime_builder(runtime(exact_mapping(), Vec::new())).unwrap();
    let lowering = builder
        .install_owned_conditional(BridgeOwnedConditionalInstallationRequest {
            contract: BridgeConditionalContract::new(BridgeConditionalContractParts {
                identity: Arc::from("exact-output"),
                dependency_count: 0,
                condition_dependency_ordinals: Vec::new(),
                condition: BridgeConditionalCondition::OnDemand,
                dependency_comparator: SignalConditionalVersionComparator::Exact,
                output_comparator: SignalConditionalVersionComparator::Exact,
                artifact_reuse: SignalConditionalArtifactReuse::NotReusable,
            }),
            location: BridgeConditionalLocation::operation("exact-output"),
            dependencies: Vec::new(),
            providers: BridgeConditionalProviderSet::new()
                .trigger(Requested)
                .compute(Compute(fault)),
        })
        .unwrap();
    (builder.seal().unwrap(), lowering)
}
fn request(
    lowering: &Arc<BridgeInstalledConditionalLowering>,
) -> BridgeConditionalExecutionRequest<'_> {
    BridgeConditionalExecutionRequest {
        lowering,
        query_binding_identity: "same-output-binding",
        query_capability_identity: 1,
        snapshot_identity: "same-descriptive-label",
        truth_branch_identity: None,
        bridge_snapshot_identity: None,
        execution_identity: "same-comparison-label",
        attempt: 2,
    }
}
fn compare(
    owner: &crate::facade::BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    versions: (u64, u64),
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<BridgeExactOutputComparison, BridgeOutputComparisonDenial> {
    let basis = owner
        .admit_conditional_signal_basis(lowering, owner.admitted_signal_basis())
        .unwrap();
    owner.compare_exact_output(execution, &basis, request(lowering), versions.0, versions.1)
}
fn retained(owner: &crate::facade::BridgeSealedRuntimeAssembly) -> u64 {
    owner
        .conditional_lifecycle_probe()
        .bridge_conditional_retention()
        .unwrap()
        .retained_bytes()
}

#[test]
fn exact_pair_initializes_the_actual_before_value_and_isolates_equal_labeled_occurrences() {
    let (owner, lowering) = install(Fault::None);
    let baseline = retained(&owner);
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let mut originals = Vec::new();
    for (before, after, expected) in [
        (
            42,
            42,
            SignalConditionalDecisionClass::ComputedRevertedClean,
        ),
        (3, 4, SignalConditionalDecisionClass::ComputedChanged),
        (0, 0, SignalConditionalDecisionClass::ComputedRevertedClean),
        (
            42,
            42,
            SignalConditionalDecisionClass::ComputedRevertedClean,
        ),
    ] {
        let comparison = compare(&owner, &lowering, (before, after), execution).unwrap();
        assert_eq!(
            comparison.baseline().signal_output_versions(),
            (0, before),
            "each private slot initializes the supplied before, including nonzero values"
        );
        assert_eq!(
            comparison.candidate().signal_output_versions(),
            (before, after)
        );
        assert_eq!(comparison.candidate().signal_class(), expected);
        assert_eq!(
            comparison
                .candidate()
                .computed_evaluation()
                .unwrap()
                .verdict,
            if before == after {
                EvaluationVerdict::Suppressed {
                    reason: SuppressionReason::ComparatorMatch,
                }
            } else {
                EvaluationVerdict::Recomputed
            }
        );
        assert!(!comparison
            .baseline()
            .same_observation(comparison.candidate()));
        assert_ne!(
            comparison.baseline().signal_projection().label(),
            comparison.candidate().signal_projection().label()
        );
        originals.push(comparison);
    }
    assert!(
        !originals[0].same_comparison(&originals[3]),
        "equal payload, lowering and request labels cannot supply occurrence identity"
    );
    let shared = originals[0].clone();
    assert!(shared.same_comparison(&originals[0]));
    drop(originals);
    assert_eq!(shared.candidate().signal_output_versions(), (42, 42));
    drop(shared);
    assert_eq!(retained(&owner), baseline);
}

#[test]
fn exact_pair_refuses_wrong_baseline_and_preserves_candidate_failure_without_a_pair() {
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    for (fault, stage) in [
        (Fault::WrongBaseline, BridgeOutputComparisonStage::Baseline),
        (
            Fault::CandidateFailure,
            BridgeOutputComparisonStage::Candidate,
        ),
    ] {
        let (owner, lowering) = install(fault);
        let baseline = retained(&owner);
        let denial = compare(&owner, &lowering, (42, 43), execution).unwrap_err();
        assert_eq!(denial.stage(), stage);
        match fault {
            Fault::WrongBaseline => assert_eq!(
                denial.cause().kind(),
                crate::facade::BridgeConditionalDenialKind::SignalContractMismatch
            ),
            Fault::CandidateFailure => assert_eq!(
                denial.cause().kind(),
                crate::facade::BridgeConditionalDenialKind::SignalExecution(
                    crate::facade::BridgeSignalDenial::Error(
                        worth_signal::facade::SignalError::invalid_input(
                            "actual candidate failure"
                        )
                    )
                )
            ),
            Fault::None => unreachable!(),
        }
        drop(denial);
        assert_eq!(
            retained(&owner),
            baseline,
            "partial private execution retains no comparison or reservation"
        );
        if fault == Fault::CandidateFailure {
            let retry = compare(&owner, &lowering, (42, 42), execution).unwrap();
            assert_eq!(retry.candidate().signal_output_versions(), (42, 42));
            drop(retry);
            assert_eq!(retained(&owner), baseline);
        }
    }
}

#[test]
fn exact_pair_rejects_foreign_owner_and_preserves_cancelled_request() {
    let (owner, lowering) = install(Fault::None);
    let (foreign, _) = install(Fault::None);
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let basis: BridgeConditionalSignalBasisBinding = owner
        .admit_conditional_signal_basis(&lowering, owner.admitted_signal_basis())
        .unwrap();
    let baseline = retained(&foreign);
    let denial = foreign
        .compare_exact_output(execution, &basis, request(&lowering), 42, 42)
        .unwrap_err();
    assert_eq!(denial.stage(), BridgeOutputComparisonStage::Admission);
    assert_eq!(
        denial.cause().kind(),
        crate::facade::BridgeConditionalDenialKind::StaleLowering
    );
    assert_eq!(retained(&foreign), baseline);
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let cancelled = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(cancellation.token(), None);
    let denied = owner
        .compare_exact_output(
            worth_execution::ExecutionRequest::serial(&cancelled),
            &basis,
            request(&lowering),
            42,
            42,
        )
        .unwrap_err();
    assert_eq!(
        denied.cause().kind(),
        crate::facade::BridgeConditionalDenialKind::ExecutionDenied(
            crate::facade::BridgeExecutionDenial::Cancelled
        )
    );
}

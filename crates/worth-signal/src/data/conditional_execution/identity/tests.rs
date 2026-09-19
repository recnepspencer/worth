use worth_proof::TransitionOutcome;

use crate::data::aspect::{Aspect, AspectMask, SignalAspectLoweringOwner};
use crate::data::graph::SignalGraph;

use super::super::SignalConditionalDecisionClass;
use super::super::{
    SignalConditionalArtifactReuse, SignalConditionalCondition,
    SignalConditionalContractDefinition, SignalConditionalVersionComparator,
    SignalDeltaThresholdContract, SignalThresholdBoundary, SignalThresholdComparisonDomain,
    SignalThresholdValueFamily,
};
fn decision_projection_basis(
    contract: &super::InstalledSignalConditionalContract,
    snapshot: &str,
    execution: &str,
    attempt: u64,
    class: SignalConditionalDecisionClass,
    dependencies: &[super::SignalConditionalDependencyVersion],
) -> String {
    super::decision_projection_basis(
        contract,
        snapshot,
        execution,
        attempt,
        class,
        dependencies,
        &mut crate::data::retained_storage::RetainedStoragePreparation::new(usize::MAX),
    )
    .unwrap()
}

#[test]
fn identical_owner_material_produces_stable_operational_identity() {
    let (_graph, contract) = installed(definition(SignalConditionalVersionComparator::Exact));

    let first = decision_projection_basis(
        &contract,
        "snapshot-a",
        "execution-a",
        1,
        SignalConditionalDecisionClass::ComputedChanged,
        &[],
    );
    let second = decision_projection_basis(
        &contract,
        "snapshot-a",
        "execution-a",
        1,
        SignalConditionalDecisionClass::ComputedChanged,
        &[],
    );

    assert_eq!(first, second);
    assert!(first.starts_with("signal-conditional-decision-v1"));
    assert!(!first.contains("ComputedChanged"));
    assert!(!first.contains("AspectFilter"));
}

#[test]
fn operational_coordinates_and_typed_contract_parameters_remain_distinct() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let TransitionOutcome::Success(exact_capability) = graph.admit_installed_node(node) else {
        panic!("fresh node admits");
    };
    let exact = graph
        .install_conditional_contract(
            &owner,
            exact_capability,
            definition(SignalConditionalVersionComparator::Exact),
        )
        .expect("the owner installs exact comparator meaning");
    let TransitionOutcome::Success(capability) = graph.admit_installed_node(node) else {
        panic!("the installed node remains live");
    };
    let tolerance = graph
        .install_conditional_contract(
            &owner,
            capability,
            definition(SignalConditionalVersionComparator::Tolerance(7)),
        )
        .expect("the owner may reinstall typed conditional meaning");
    let threshold_a = reinstall(
        &mut graph,
        &owner,
        node,
        threshold_definition(10, "millisecond"),
    );
    let threshold_b = reinstall(
        &mut graph,
        &owner,
        node,
        threshold_definition(11, "millisecond"),
    );

    let identity = |contract, snapshot, execution, attempt, class| {
        decision_projection_basis(contract, snapshot, execution, attempt, class, &[])
    };
    let baseline = identity(
        &exact,
        "snapshot-a",
        "execution-a",
        1,
        SignalConditionalDecisionClass::ComputedChanged,
    );
    assert_ne!(
        baseline,
        identity(
            &exact,
            "snapshot-b",
            "execution-a",
            1,
            SignalConditionalDecisionClass::ComputedChanged
        )
    );
    assert_ne!(
        baseline,
        identity(
            &exact,
            "snapshot-a",
            "execution-b",
            1,
            SignalConditionalDecisionClass::ComputedChanged
        )
    );
    assert_ne!(
        baseline,
        identity(
            &exact,
            "snapshot-a",
            "execution-a",
            2,
            SignalConditionalDecisionClass::ComputedChanged
        )
    );
    assert_ne!(
        baseline,
        identity(
            &exact,
            "snapshot-a",
            "execution-a",
            1,
            SignalConditionalDecisionClass::ComputedRevertedClean
        )
    );
    assert_ne!(
        baseline,
        identity(
            &tolerance,
            "snapshot-a",
            "execution-a",
            1,
            SignalConditionalDecisionClass::ComputedChanged
        )
    );
    assert_ne!(
        identity(
            &threshold_a,
            "snapshot-a",
            "execution-a",
            1,
            SignalConditionalDecisionClass::ComputedChanged
        ),
        identity(
            &threshold_b,
            "snapshot-a",
            "execution-a",
            1,
            SignalConditionalDecisionClass::ComputedChanged
        )
    );
}

fn reinstall(
    graph: &mut SignalGraph,
    owner: &SignalAspectLoweringOwner,
    node: crate::data::handle::NodeId,
    definition: SignalConditionalContractDefinition,
) -> super::super::InstalledSignalConditionalContract {
    let TransitionOutcome::Success(capability) = graph.admit_installed_node(node) else {
        panic!("the installed node remains live");
    };
    graph
        .install_conditional_contract(owner, capability, definition)
        .expect("the owner may reinstall typed conditional meaning")
}

fn installed(
    definition: SignalConditionalContractDefinition,
) -> (
    SignalGraph,
    super::super::InstalledSignalConditionalContract,
) {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let owner = SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let TransitionOutcome::Success(capability) = graph.admit_installed_node(node) else {
        panic!("fresh node admits");
    };
    let contract = graph
        .install_conditional_contract(&owner, capability, definition)
        .unwrap();
    (graph, contract)
}

fn definition(
    comparator: SignalConditionalVersionComparator,
) -> SignalConditionalContractDefinition {
    SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::AspectFilter(AspectMask::from_aspect(Aspect::new(
            3,
        ))),
        dependency_aspects: AspectMask::from_aspect(Aspect::new(1)),
        trigger_aspects: AspectMask::from_aspect(Aspect::new(2)),
        dependency_comparator: comparator,
        output_comparator: SignalConditionalVersionComparator::Exact,
        artifact_reuse: SignalConditionalArtifactReuse::DependencyAndOutputEquivalent,
    }
}

fn threshold_definition(value: u64, unit: &str) -> SignalConditionalContractDefinition {
    SignalConditionalContractDefinition {
        condition: SignalConditionalCondition::DeltaThreshold(SignalDeltaThresholdContract::new(
            worth_foundational::facade::AspectValue::UInt64(value),
            unit,
            SignalThresholdValueFamily::Integer,
            SignalThresholdComparisonDomain::AbsoluteDifference,
            SignalThresholdBoundary::Inclusive,
        )),
        ..definition(SignalConditionalVersionComparator::Exact)
    }
}

#[test]
fn projection_admission_covers_long_text_before_rendering_and_keeps_installed_meaning() {
    use crate::data::error::SignalError;
    use crate::data::output::PartitionSubscription;
    use crate::data::retained_storage::RetainedStoragePreparation;
    let (_graph, contract) = installed(threshold_definition(u64::MAX, &"unit-λ|=:".repeat(400)));
    let dependencies = [super::SignalConditionalDependencyVersion {
        node: contract.node(),
        aspect: Aspect::new(3),
        version: u64::MAX,
        scope: Some(PartitionSubscription::partition_and_detail(
            "p|λ".repeat(800),
            "d:=λ".repeat(900),
        )),
    }];
    let snapshot = "snapshot-λ|=:".repeat(300);
    let execution = "execution-λ|=:".repeat(200);
    let mut full = RetainedStoragePreparation::new(1_000_000);
    let material = super::decision_projection_basis(
        &contract,
        &snapshot,
        &execution,
        u64::MAX,
        SignalConditionalDecisionClass::ComputedChanged,
        &dependencies,
        &mut full,
    )
    .unwrap();
    assert!(material.ends_with(&super::contract_projection_basis(&contract)));
    assert!(material.contains(&format!("snapshot={}:{}", snapshot.len(), snapshot)));
    let consumed = full.visits();
    for limit in [consumed - 1, consumed] {
        let mut work = RetainedStoragePreparation::new(limit);
        let result = super::decision_projection_basis(
            &contract,
            &snapshot,
            &execution,
            u64::MAX,
            SignalConditionalDecisionClass::ComputedChanged,
            &dependencies,
            &mut work,
        );
        if limit == consumed {
            assert_eq!(result.unwrap(), material);
        } else {
            assert_eq!(
                result,
                Err(SignalError::ConditionalEvaluationWorkExhausted {
                    maximum_visits: limit
                })
            );
            assert_eq!(work.visits(), dependencies.len() + 1);
        }
    }
}

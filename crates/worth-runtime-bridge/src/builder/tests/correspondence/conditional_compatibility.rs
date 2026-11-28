use std::sync::Arc;

use worth_signal::facade::{Aspect, NodeEvaluationResult, PartitionToken, SignalGraph};

use super::semantic_dependencies::{
    always_eligible_contract, conditional_contract, dependency, freshly_installed_dependency,
};
use super::{exact_mapping, registration, runtime, BridgeAspectRegistrationId};
use crate::facade::{
    BridgeConditionalComputeProvider, BridgeConditionalContinuityMismatch,
    BridgeConditionalContract, BridgeConditionalExecutionAffinityMismatch,
    BridgeConditionalInstallationRequest, BridgeConditionalLocation,
    BridgeConditionalProviderSemantics, BridgeConditionalProviderSet,
    BridgeConditionalRuntimeBuilder, BridgeInstalledConditionalLowering,
    BridgeSealedRuntimeAssembly, BridgeSignalAspectTargetDeclaration,
};

mod certification;
mod evaluation_budget;
mod managed_time;
mod owned_installation;
mod provider_semantics;
mod retained_decision;
mod retained_source;
mod sealed_delivery;
mod source_admission;
mod source_packet_affinity;

pub(super) struct Compute(pub(super) u64);

impl BridgeConditionalProviderSemantics for Compute {
    type SemanticContract = u64;

    fn semantic_contract(&self) -> Self::SemanticContract {
        self.0
    }

    fn retained_heap_bytes(
        &self,
        _semantic_contract: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}

impl BridgeConditionalComputeProvider for Compute {
    fn compute(&self, _context: &mut dyn std::any::Any) -> Result<NodeEvaluationResult, String> {
        Ok(NodeEvaluationResult::from_version(
            worth_signal::facade::AspectVersion::from_updates([(
                worth_signal::facade::Aspect::new(0),
                self.0,
            )]),
        ))
    }
}

pub(super) fn install(
    contract: BridgeConditionalContract,
    partition: &str,
) -> (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    install_with(
        contract,
        partition,
        BridgeConditionalProviderSet::new().compute(Compute(1)),
    )
}

fn install_with(
    contract: BridgeConditionalContract,
    partition: &str,
    providers: BridgeConditionalProviderSet,
) -> (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    install_with_target_partitions(contract, &[partition], providers)
}

fn install_with_target_partitions(
    contract: BridgeConditionalContract,
    partitions: &[&str],
    providers: BridgeConditionalProviderSet,
) -> (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
) {
    let (mut builder, request) = installation_fixture(contract, partitions, providers);
    let lowering = builder
        .install(request)
        .expect("owner-bound conditional lowering installs");
    let owner = builder
        .seal()
        .expect("installed conditional operations seal");
    (owner, lowering)
}

fn installation_fixture(
    contract: BridgeConditionalContract,
    partitions: &[&str],
    providers: BridgeConditionalProviderSet,
) -> (
    BridgeConditionalRuntimeBuilder,
    BridgeConditionalInstallationRequest,
) {
    installation_fixture_with_baseline(contract, partitions, providers, &[])
}

fn installation_fixture_with_baseline(
    contract: BridgeConditionalContract,
    partitions: &[&str],
    providers: BridgeConditionalProviderSet,
    baseline_labels: &[&str],
) -> (
    BridgeConditionalRuntimeBuilder,
    BridgeConditionalInstallationRequest,
) {
    installation_fixture_with_runtime(
        contract,
        partitions,
        providers,
        baseline_labels,
        |baseline| runtime(exact_mapping(), baseline),
    )
}

fn installation_fixture_with_runtime(
    contract: BridgeConditionalContract,
    partitions: &[&str],
    providers: BridgeConditionalProviderSet,
    baseline_labels: &[&str],
    build_runtime: impl FnOnce(
        Vec<crate::facade::BridgeSemanticCorrespondenceRegistration>,
    ) -> crate::facade::RuntimeBridge,
) -> (
    BridgeConditionalRuntimeBuilder,
    BridgeConditionalInstallationRequest,
) {
    installation_fixture_with_runtime_and_budget(
        contract,
        partitions,
        providers,
        baseline_labels,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
        build_runtime,
    )
}

pub(super) fn installation_fixture_with_budget(
    contract: BridgeConditionalContract,
    partitions: &[&str],
    providers: BridgeConditionalProviderSet,
    budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
) -> (
    BridgeConditionalRuntimeBuilder,
    BridgeConditionalInstallationRequest,
) {
    installation_fixture_with_runtime_and_budget(
        contract,
        partitions,
        providers,
        &[],
        budget,
        |baseline| runtime(exact_mapping(), baseline),
    )
}

fn installation_fixture_with_runtime_and_budget(
    contract: BridgeConditionalContract,
    partitions: &[&str],
    providers: BridgeConditionalProviderSet,
    baseline_labels: &[&str],
    budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    build_runtime: impl FnOnce(
        Vec<crate::facade::BridgeSemanticCorrespondenceRegistration>,
    ) -> crate::facade::RuntimeBridge,
) -> (
    BridgeConditionalRuntimeBuilder,
    BridgeConditionalInstallationRequest,
) {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let targets = partitions
        .iter()
        .enumerate()
        .map(|(index, partition)| {
            let worth_proof::TransitionOutcome::Success(node_capability) =
                graph.admit_installed_node(node)
            else {
                panic!("fresh Signal node admits");
            };
            if partitions.len() == 1 {
                BridgeSignalAspectTargetDeclaration::allocate(
                    BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
                    PartitionToken::new(*partition),
                    node_capability,
                )
            } else {
                let worth_proof::TransitionOutcome::Success(aspect_capability) =
                    graph.admit_installed_aspect(node, Aspect::new(index as u8))
                else {
                    panic!("fresh Signal aspect admits");
                };
                BridgeSignalAspectTargetDeclaration::exact(
                    BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
                    PartitionToken::new(*partition),
                    node_capability,
                    aspect_capability,
                )
                .expect("node and aspect capabilities share the graph owner")
            }
        })
        .collect();
    let request_dependency = if baseline_labels.is_empty() {
        freshly_installed_dependency("query:one")
    } else {
        dependency("query:one")
    };
    let request_registration = registration(request_dependency, targets);
    let baseline = baseline_labels
        .iter()
        .map(|label| {
            let baseline_node = graph.node().build();
            let worth_proof::TransitionOutcome::Success(node_capability) =
                graph.admit_installed_node(baseline_node)
            else {
                panic!("baseline Signal node admits");
            };
            registration(
                dependency(label),
                vec![BridgeSignalAspectTargetDeclaration::allocate(
                    BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
                    PartitionToken::new("bridge-baseline"),
                    node_capability,
                )],
            )
        })
        .collect();
    let owner =
        BridgeConditionalRuntimeBuilder::new(build_runtime(baseline), Box::new(graph), budget)
            .expect("Bridge owns the fresh Signal runtime");
    (
        owner,
        BridgeConditionalInstallationRequest {
            contract,
            location: BridgeConditionalLocation::operation("query:one"),
            registrations: vec![request_registration],
            providers,
        },
    )
}

pub(super) fn owned_runtime_builder(
    bridge: crate::facade::RuntimeBridge,
) -> Result<BridgeConditionalRuntimeBuilder, crate::facade::BridgeConditionalDenial> {
    BridgeConditionalRuntimeBuilder::with_owned_signal_graph(
        bridge,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
    )
}

#[test]
fn promoted_temporal_basis_checks_wake_membership_and_clock_domain() {
    use crate::facade::{
        BridgeTemporalBasisDenial, BridgeTemporalSignalBasis, BridgeTemporalSignalBasisDenial,
        BridgeTemporalTruthViewBasis,
    };
    use crate::truth_identity_fixtures::{
        truth_branch_fixture, truth_commit_fixture, truth_snapshot_fixture,
    };
    use std::num::NonZeroUsize;
    use worth_proof::{ConditionalSourceObservationOwner, TransitionOutcome};
    use worth_signal::facade::{
        ClockAdvanceOrdinal, ClockAdvanceRequest, ClockDomain, ClockTick,
        SignalAspectLoweringOwner, SignalRuntime, TemporalCondition, TemporalWakeId,
    };
    let owner = SignalAspectLoweringOwner::fresh();
    let mut graph = SignalGraph::new();
    graph.claim_aspect_lowering_owner(&owner).unwrap();
    let mut signal = SignalRuntime::build_for::<()>(graph);
    let basis = signal
        .observe_signal_branch_basis(signal.current_branch())
        .unwrap();
    signal.owner_component_services().unwrap();
    let source = ConditionalSourceObservationOwner::fresh();
    let service = signal
        .issue_conditional_execution_service(&basis, &owner, &source.authority())
        .unwrap();
    let maximum = NonZeroUsize::new(1).unwrap();
    let partition = service.admit_temporal_partition(maximum).unwrap();
    let tick = ClockTick::new(3);
    let domain = ClockDomain::MonotonicExecution;
    let scheduled = partition
        .schedule_temporal_wake(TemporalCondition::at_or_after(tick), tick)
        .unwrap();
    let (_, promotion) = partition
        .advance_clock_and_promote_due_temporal_wakes_ready_bounded(
            ClockAdvanceRequest::new(domain, tick),
            maximum,
        )
        .unwrap();

    let bridge = runtime(exact_mapping(), Vec::new());
    let branch = truth_branch_fixture("temporal-promotion-branch");
    let truth = BridgeTemporalTruthViewBasis::authoritative(
        branch.clone(),
        truth_commit_fixture("temporal-promotion-commit"),
        truth_snapshot_fixture("temporal-promotion-snapshot"),
    );
    let clock = BridgeTemporalSignalBasis::new(
        branch.clone(),
        domain,
        tick,
        ClockAdvanceOrdinal::new(1),
        None,
    );
    assert!(matches!(
        bridge.admit_promoted_temporal_basis(
            truth.clone(),
            clock.clone(),
            &promotion,
            TemporalWakeId::new(scheduled.id().get() + 1),
        ),
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::MissingWakeEvidence)
    ));
    let wrong_clock = BridgeTemporalSignalBasis::new(
        branch,
        ClockDomain::WallClock,
        tick,
        ClockAdvanceOrdinal::new(1),
        None,
    );
    assert!(matches!(
        bridge.admit_promoted_temporal_basis(
            truth.clone(),
            wrong_clock,
            &promotion,
            scheduled.id()
        ),
        TransitionOutcome::Denied(BridgeTemporalBasisDenial::SignalBasisDenied(
            BridgeTemporalSignalBasisDenial::WrongClockDomain {
                domain: ClockDomain::WallClock
            }
        ))
    ));
    let outcome = bridge.admit_promoted_temporal_basis(truth, clock, &promotion, scheduled.id());
    let TransitionOutcome::Success(admitted) = outcome else {
        panic!("the promoted wake should admit: {outcome:?}");
    };
    let evidence = admitted.wake_evidence().evidence();
    assert_eq!(evidence.wake_id(), scheduled.id());
    assert_eq!(evidence.wake_tick(), tick);
    assert_eq!(
        evidence.wake_ready_ordinal(),
        promotion.promotion().ready_wakes()[0].ready_ordinal()
    );
}

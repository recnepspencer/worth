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

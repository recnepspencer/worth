use super::BridgeSemanticClaimOrigin;
use std::collections::BTreeSet;
use worth_signal::facade::{InstalledSignalGraphCapability, SignalGraph};

use crate::facade::RuntimeBridge;

use super::admission::CorrespondenceAdmissionOutcome;
use super::{
    BridgeCorrespondenceBasis, BridgeCorrespondenceDenialKind, BridgeCorrespondenceRebindRequired,
    BridgeInstalledSemanticCorrespondence, BridgeSemanticDependencyCandidate,
    BridgeSignalAspectTargetDeclaration, CorrespondenceAdmissionCounters,
};

pub(super) struct ResolvedCorrespondence {
    pub(super) recipe: super::installed_witness::CorrespondenceResolvedRecipe,
    pub(super) declarations: Vec<BridgeSignalAspectTargetDeclaration>,
    pub(super) signal_graph: InstalledSignalGraphCapability,
    pub(super) owner: BridgeSemanticClaimOrigin,
    pub(super) counters: CorrespondenceAdmissionCounters,
}

pub(super) fn resolve(
    runtime: &RuntimeBridge,
    dependency: BridgeSemanticDependencyCandidate,
    graph: &SignalGraph,
) -> Result<ResolvedCorrespondence, CorrespondenceAdmissionOutcome> {
    let owner = runtime
        .semantic_dependency_registry
        .claim_origin(&dependency);
    let unresolved = BridgeInstalledSemanticCorrespondence::begin(dependency);
    let counters = CorrespondenceAdmissionCounters {
        semantic_dependency_lookups: 1,
        ..CorrespondenceAdmissionCounters::default()
    };
    let Some(declarations) = runtime
        .semantic_dependency_registry
        .resolve(unresolved.payload())
    else {
        return Err(super::admission::denied(
            BridgeCorrespondenceDenialKind::PortableDependencyNotInstalled,
            counters,
        ));
    };
    resolve_declarations(runtime, unresolved, declarations, graph, counters, owner)
}

pub(super) fn resolve_registration(
    runtime: &RuntimeBridge,
    registration: &super::BridgeSemanticCorrespondenceRegistration,
    graph: &SignalGraph,
) -> Result<ResolvedCorrespondence, CorrespondenceAdmissionOutcome> {
    let owner = registration
        .recorded_claim
        .map(BridgeSemanticClaimOrigin::Recorded)
        .unwrap_or_else(|| {
            runtime
                .semantic_dependency_registry
                .claim_origin(&registration.dependency)
        });
    let unresolved = BridgeInstalledSemanticCorrespondence::begin(registration.dependency.clone());
    let counters = CorrespondenceAdmissionCounters {
        provided_registration_reads: 1,
        ..CorrespondenceAdmissionCounters::default()
    };
    resolve_declarations(
        runtime,
        unresolved,
        registration.targets.clone(),
        graph,
        counters,
        owner,
    )
}

fn resolve_declarations(
    runtime: &RuntimeBridge,
    unresolved: worth_proof::Recipe<worth_proof::Unresolved, BridgeSemanticDependencyCandidate>,
    declarations: Vec<BridgeSignalAspectTargetDeclaration>,
    graph: &SignalGraph,
    mut counters: CorrespondenceAdmissionCounters,
    owner: BridgeSemanticClaimOrigin,
) -> Result<ResolvedCorrespondence, CorrespondenceAdmissionOutcome> {
    counters.registered_targets_materialized = declarations.len();
    let signal_graph = graph.installed_graph_capability();
    let graph_basis = declarations[0].graph_instance_id();
    if graph_basis != signal_graph.graph_instance_id() {
        return Err(worth_proof::TransitionOutcome::RebindRequired(
            BridgeCorrespondenceRebindRequired::SignalGraphGeneration,
        ));
    }
    counters.source_profile_cache_reads = 1;
    let Some(authoritative_source_profile) = runtime.authoritative_source_profile.clone() else {
        return Err(super::admission::denied(
            BridgeCorrespondenceDenialKind::AuthoritativeSourceMismatch,
            counters,
        ));
    };
    let basis = BridgeCorrespondenceBasis {
        source_installation_identity: unresolved.payload().source_installation_identity.clone(),
        source_basis: unresolved.payload().source_basis.clone(),
        source_runtime_authority: unresolved.payload().source_runtime_authority,
        source_installation_generation: unresolved.payload().source_installation_generation,
        source_authority_binding_identity: unresolved
            .payload()
            .source_authority_binding_identity
            .clone(),
        declared_graph_role: unresolved.payload().declared_graph_role.clone(),
        graph_participation_identity: unresolved.payload().graph_participation_identity.clone(),
        graph_adapter_identity: unresolved.payload().graph_adapter_identity.clone(),
        authoritative_source_profile: Some(authoritative_source_profile),
        bridge_runtime_key: runtime.signal_runtime_key,
        signal_graph_instance_id: signal_graph.graph_instance_id(),
        signal_partitions: declarations
            .iter()
            .map(|declaration| declaration.partition.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
    };
    let recipe = BridgeInstalledSemanticCorrespondence::resolve(unresolved, basis);
    Ok(ResolvedCorrespondence {
        recipe,
        declarations,
        signal_graph,
        owner,
        counters,
    })
}

use std::collections::{BTreeMap, BTreeSet};

use crate::runtime::replacement::query_binding::{
    evidence_accumulator::WorthUiQueryBindingEvidenceAccumulator, WorthUiQueryBindingIdentity,
    WorthUiQueryBindingUiRequirements,
};
use crate::runtime::WorthUiCandidateDependencyMetadata;
use crate::source::{
    WorthUiArtifact, WorthUiArtifactDependencyDeriver, WorthUiArtifactDependencyGraph,
    WorthUiArtifactDigestor, WorthUiArtifactEquivalenceBasis, WorthUiArtifactNode,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthUiQueryBindingEvidence {
    identity: WorthUiQueryBindingIdentity,
    ui_requirements: WorthUiQueryBindingUiRequirements,
    installed_reference: Option<worth_ui_query_binding::WorthUiInstalledQueryBindingReference>,
    settled: Option<worth_ui_query_binding::WorthUiExactSettledSnapshotEvidence>,
    exact_live_resource: Option<worth_ui_query_binding::WorthUiExactOperationLiveResourceEvidence>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct WorthUiQueryBindingEvidenceIndex {
    bindings: BTreeMap<String, WorthUiQueryBindingEvidence>,
}

impl WorthUiQueryBindingEvidence {
    pub(super) fn new(
        identity: WorthUiQueryBindingIdentity,
        ui_requirements: WorthUiQueryBindingUiRequirements,
    ) -> Self {
        Self {
            identity,
            ui_requirements,
            installed_reference: None,
            settled: None,
            exact_live_resource: None,
        }
    }

    pub(crate) fn identity(&self) -> &WorthUiQueryBindingIdentity {
        &self.identity
    }

    pub(crate) fn ui_requirements(&self) -> &WorthUiQueryBindingUiRequirements {
        &self.ui_requirements
    }

    pub(crate) fn installed_reference(
        &self,
    ) -> Option<&worth_ui_query_binding::WorthUiInstalledQueryBindingReference> {
        self.installed_reference.as_ref()
    }

    pub(crate) fn settled(
        &self,
    ) -> Option<&worth_ui_query_binding::WorthUiExactSettledSnapshotEvidence> {
        self.settled.as_ref()
    }

    pub(crate) fn exact_live_resource(
        &self,
    ) -> Option<&worth_ui_query_binding::WorthUiExactOperationLiveResourceEvidence> {
        self.exact_live_resource.as_ref()
    }

    fn attach_query_authority(
        &mut self,
        plan: &worth_ui_query_binding::WorthUiQueryBindingPlan,
        binding: &worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    ) {
        let identity = worth_ui_query_binding::WorthUiQueryViewIdentity::new(
            self.identity.query_view_identity().as_str(),
        )
        .expect("admitted Query view identity remains valid");
        self.installed_reference = plan.resolve_definition(&identity, self.identity.result_shape());
        self.settled = self.installed_reference.as_ref().and_then(|reference| {
            binding
                .exact_settled_snapshot_evidence_for(reference)
                .ok()
                .flatten()
        });
        self.exact_live_resource = self.installed_reference.as_ref().and_then(|reference| {
            binding
                .exact_operation_live_resource_evidence_for(reference)
                .ok()
                .flatten()
        });
    }
}

impl WorthUiQueryBindingEvidenceIndex {
    pub(crate) fn from_active_artifact_for_bindings(
        active: &crate::runtime::active::WorthUiActiveArtifact,
        binding_ids: &BTreeSet<String>,
        plan: &worth_ui_query_binding::WorthUiQueryBindingPlan,
        binding: &worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    ) -> Self {
        Self::from_artifact_and_graph_for_bindings(
            active.artifact(),
            active.dependency_graph(),
            binding_ids,
            plan,
            binding,
        )
    }

    pub(crate) fn from_artifact_and_graph_for_bindings(
        artifact: &WorthUiArtifact,
        graph: &WorthUiArtifactDependencyGraph,
        binding_ids: &BTreeSet<String>,
        plan: &worth_ui_query_binding::WorthUiQueryBindingPlan,
        binding: &worth_ui_query_binding::WorthUiRuntimeQueryBinding,
    ) -> Self {
        let bindings = binding_ids
            .iter()
            .filter_map(|binding_id| {
                let mut accumulator = WorthUiQueryBindingEvidenceAccumulator::default();
                record_artifact_link_for_binding(artifact, binding_id, &mut accumulator);
                for hook in graph.runtime_hooks_for_query_binding(binding_id) {
                    accumulator.record_runtime_hook(hook);
                }
                accumulator.finish(binding_id).map(|mut evidence| {
                    evidence.attach_query_authority(plan, binding);
                    (binding_id.clone(), evidence)
                })
            })
            .collect();
        Self { bindings }
    }

    pub(crate) fn from_active_artifact(artifact: &WorthUiArtifact) -> Self {
        let report = WorthUiArtifactDependencyDeriver::derive_with_report(artifact);
        let metadata = WorthUiCandidateDependencyMetadata::from_derived_report(
            WorthUiArtifactDigestor::digest(artifact, WorthUiArtifactEquivalenceBasis::semantic()),
            report,
        );
        Self::from_artifact_graph_without_query_authority(
            artifact,
            metadata.invalidation_basis().dependency_graph(),
        )
    }

    fn from_artifact_graph_without_query_authority(
        artifact: &WorthUiArtifact,
        graph: &WorthUiArtifactDependencyGraph,
    ) -> Self {
        let mut accumulators = BTreeMap::<String, WorthUiQueryBindingEvidenceAccumulator>::new();
        record_artifact_links(artifact, &mut accumulators);
        record_runtime_hooks(graph, &mut accumulators);
        Self {
            bindings: accumulators
                .into_iter()
                .filter_map(|(binding_id, accumulator)| {
                    accumulator
                        .finish(&binding_id)
                        .map(|evidence| (binding_id, evidence))
                })
                .collect(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.bindings.len()
    }

    pub(crate) fn get(&self, view_binding_id: &str) -> Option<&WorthUiQueryBindingEvidence> {
        self.bindings.get(view_binding_id)
    }

    pub(crate) fn binding_ids(&self) -> BTreeSet<String> {
        self.bindings.keys().cloned().collect()
    }

    pub(crate) fn entries(
        &self,
    ) -> impl Iterator<
        Item = (
            &WorthUiQueryBindingIdentity,
            &WorthUiQueryBindingUiRequirements,
        ),
    > {
        self.bindings
            .values()
            .map(|evidence| (evidence.identity(), evidence.ui_requirements()))
    }
}

fn record_artifact_link_for_binding(
    artifact: &WorthUiArtifact,
    binding_id: &str,
    accumulator: &mut WorthUiQueryBindingEvidenceAccumulator,
) {
    let Some(node) = artifact.node_for_identity_basis(binding_id) else {
        return;
    };
    match node {
        WorthUiArtifactNode::Binding(binding) => {
            accumulator.record_bound_view_binding(binding.view_binding_reference());
        }
        WorthUiArtifactNode::Surface(surface) => {
            if let Some(view_binding) = surface.semantics().view_binding() {
                accumulator.record_bound_view_binding(view_binding);
            }
        }
        WorthUiArtifactNode::Import(_)
        | WorthUiArtifactNode::Component(_)
        | WorthUiArtifactNode::Token(_) => {}
    }
}

fn record_runtime_hooks(
    graph: &WorthUiArtifactDependencyGraph,
    accumulators: &mut BTreeMap<String, WorthUiQueryBindingEvidenceAccumulator>,
) {
    for hooks in graph.runtime_hooks().values() {
        for hook in hooks {
            let binding_id = hook.view_binding_id().as_str().to_owned();
            let accumulator = accumulators.entry(binding_id).or_default();
            accumulator.record_runtime_hook(hook);
        }
    }
}

fn record_artifact_links(
    artifact: &WorthUiArtifact,
    accumulators: &mut BTreeMap<String, WorthUiQueryBindingEvidenceAccumulator>,
) {
    for module_id in artifact.module_ids() {
        let Some(module) = artifact.module(module_id) else {
            continue;
        };
        for node in module.nodes() {
            match node {
                WorthUiArtifactNode::Binding(binding) => {
                    let view_binding = binding.view_binding_reference();
                    accumulators
                        .entry(view_binding.view_binding().id().as_str().to_owned())
                        .or_default()
                        .record_bound_view_binding(view_binding);
                }
                WorthUiArtifactNode::Surface(surface) => {
                    if let Some(view_binding) = surface.semantics().view_binding() {
                        accumulators
                            .entry(view_binding.view_binding().id().as_str().to_owned())
                            .or_default()
                            .record_bound_view_binding(view_binding);
                    }
                }
                WorthUiArtifactNode::Import(_)
                | WorthUiArtifactNode::Component(_)
                | WorthUiArtifactNode::Token(_) => {}
            }
        }
    }
}

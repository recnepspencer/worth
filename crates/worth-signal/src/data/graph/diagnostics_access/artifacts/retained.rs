use crate::data::error::SignalError;
use crate::data::graph::signal_graph::SignalGraph;
use crate::data::handle::NodeId;
use crate::diagnostics::facts::{ExplanationFact, ProvenanceFact};
use crate::diagnostics::policy::DiagnosticsAvailability;
use crate::logic::explain::{
    CausalDisposition, CausalLink, CausalLinkKind, NodeExplanation, ScopeProvenance,
    ScopeProvenanceKind,
};

impl SignalGraph {
    pub(super) fn attach_rewiring_topology_links(
        &self,
        explanation: &mut NodeExplanation,
        rewiring: &crate::logic::explain::RewiringSummary,
    ) {
        explanation
            .causal_links
            .reserve(rewiring.removed.len() + rewiring.added.len());
        for dependency in &rewiring.removed {
            explanation.causal_links.push(CausalLink {
                source: Some(dependency.source),
                aspect: Some(dependency.aspect),
                disposition: CausalDisposition::Topology,
                kind: CausalLinkKind::DependencyRemoved,
                scope: ScopeProvenance {
                    source_scope: dependency.subscription.clone(),
                    validation_scope: dependency.subscription.clone(),
                    kind: ScopeProvenanceKind::Direct,
                    note: Some("dependency rewired away from current topology".to_string()),
                },
                cached_version: None,
                current_version: None,
                comparator: None,
                reason: None,
                note: Some("rewiring removed this dependency during apply".to_string()),
            });
        }

        for dependency in &rewiring.added {
            explanation.causal_links.push(CausalLink {
                source: Some(dependency.source),
                aspect: Some(dependency.aspect),
                disposition: CausalDisposition::Topology,
                kind: CausalLinkKind::DependencyAdded,
                scope: ScopeProvenance {
                    source_scope: dependency.subscription.clone(),
                    validation_scope: dependency.subscription.clone(),
                    kind: ScopeProvenanceKind::Direct,
                    note: Some(
                        "dependency entered the active topology during rewiring".to_string(),
                    ),
                },
                cached_version: None,
                current_version: self
                    .node_version_for_scope(
                        dependency.source,
                        dependency.aspect,
                        dependency.subscription.as_ref(),
                    )
                    .ok(),
                comparator: None,
                reason: None,
                note: Some("rewiring added this dependency during apply".to_string()),
            });
        }
    }

    pub(crate) fn record_operational_diagnostic_facts(
        &mut self,
        node: NodeId,
        rewiring: Option<crate::logic::explain::RewiringSummary>,
    ) -> Result<(), SignalError> {
        if !self.captures_observation_surface(
            crate::logic::transaction::SignalObservationSurface::DescriptiveFacts,
        ) {
            return Ok(());
        }
        let policy = self.installed_runtime_policy();
        if !policy.retains_explanation_facts() && !policy.retains_provenance_facts() {
            return Ok(());
        }
        let Some(runtime) = self.node_runtime_artifact_state(node)? else {
            return Ok(());
        };
        let contract = self.get_contract(node)?.clone();
        let condition = self.node_eval_config(node)?.condition.clone();
        let state = self.get_state(node)?;
        let cold_artifact = self.node_cold_artifact_record(node)?;
        let execution_trace = self.node_execution_trace_stamp(node)?;
        let causality = self.causality_of(node)?;
        let mut compact_explanation = ExplanationFact::compact_explanation_from_runtime_projection(
            node,
            state,
            contract.semantics.reads,
            contract.semantics.produces,
            contract.semantics.partition_scope.clone(),
            contract.semantics.required_context,
            condition,
            runtime,
            cold_artifact,
            execution_trace,
            causality,
            rewiring.clone(),
        );
        if let Some(rewiring) = compact_explanation.rewiring.clone() {
            self.attach_rewiring_topology_links(&mut compact_explanation, &rewiring);
        }
        compact_explanation.materialization_mode = DiagnosticsAvailability::RetainedAvailable;
        let explanation_fact = policy.retains_explanation_facts().then(|| {
            let mut fact = ExplanationFact::from_explanation(&compact_explanation);
            fact.compact_projection = true;
            fact
        });
        let provenance_fact = policy
            .retains_provenance_facts()
            .then(|| ProvenanceFact::from_explanation(&compact_explanation));
        let diagnostics = self.diagnostics_state_mut();
        if let Some(fact) = explanation_fact {
            diagnostics.record_explanation_fact(fact);
        }
        if let Some(fact) = provenance_fact {
            diagnostics.record_provenance_fact(fact);
        }
        Ok(())
    }

    pub(crate) fn explanation_fact(&self, node: NodeId) -> Option<&ExplanationFact> {
        self.observation.diagnostics.explanation_facts().get(&node)
    }

    pub(crate) fn provenance_fact(&self, node: NodeId) -> Option<&ProvenanceFact> {
        self.observation.diagnostics.provenance_facts().get(&node)
    }
}

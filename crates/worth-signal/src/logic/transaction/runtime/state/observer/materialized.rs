use crate::data::error::SignalError;
use crate::data::graph::GraphMaterializer;
use crate::data::handle::NodeId;
use crate::diagnostics::facts::ProvenanceFact;
use crate::logic::explain::NodeExplanation;

use super::RuntimeMaterializer;

impl<'a, D, I, E, Ctx, T> RuntimeMaterializer<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn graph(&self) -> GraphMaterializer<'a> {
        self.runtime.assert_construction_graph_access();
        self.runtime.graph.observe().materialize()
    }

    pub fn retained_explanation_artifact(&self, node: NodeId) -> Option<NodeExplanation> {
        self.graph().retained_explanation_artifact(node)
    }

    pub fn materialize_historical_artifact_record(
        &self,
        node: NodeId,
    ) -> Result<Option<crate::data::trace::HistoricalArtifactRecord>, SignalError> {
        self.graph().materialize_historical_artifact_record(node)
    }

    pub fn materialize_trace_summary(
        &self,
        node: NodeId,
    ) -> Result<Option<crate::data::trace::TraceSummary>, SignalError> {
        self.graph().materialize_trace_summary(node)
    }

    pub fn reconstruct_explanation_artifact(
        &self,
        node: NodeId,
    ) -> Result<NodeExplanation, SignalError> {
        self.graph().reconstruct_explanation_artifact(node)
    }

    pub fn materialize_explanation_artifact(
        &self,
        node: NodeId,
    ) -> Result<
        (
            Option<NodeExplanation>,
            crate::diagnostics::policy::DiagnosticsAvailability,
        ),
        SignalError,
    > {
        self.graph().materialize_explanation_artifact(node)
    }

    pub fn retained_provenance_artifact(&self, node: NodeId) -> Option<ProvenanceFact> {
        self.graph().retained_provenance_artifact(node)
    }

    pub fn reconstruct_provenance_artifact(
        &self,
        node: NodeId,
    ) -> Result<ProvenanceFact, SignalError> {
        self.graph().reconstruct_provenance_artifact(node)
    }

    pub fn materialize_provenance_artifact(
        &self,
        node: NodeId,
    ) -> Result<
        (
            Option<ProvenanceFact>,
            crate::diagnostics::policy::DiagnosticsAvailability,
        ),
        SignalError,
    > {
        self.graph().materialize_provenance_artifact(node)
    }
}

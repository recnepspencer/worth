use super::GraphObserver;
use crate::data::handle::NodeId;
use crate::diagnostics::lineage::{
    LineageArtifactId, RetainedLineageView, SynthesizedLineageChain,
};

impl<'a> GraphObserver<'a> {
    pub fn lineage_records(&self) -> RetainedLineageView<'a> {
        let records = self.graph.observation.diagnostics.lineage_records();
        RetainedLineageView::new(records, 0, records.len())
    }

    pub fn lineage_for_node(&self, node: NodeId) -> RetainedLineageView<'a> {
        self.graph
            .diagnostics_state()
            .lineage_records_for_node(node)
            .map(|records| RetainedLineageView::new(records, 0, records.len()))
            .unwrap_or_else(RetainedLineageView::empty)
    }

    pub fn lineage_for_artifact(&self, artifact_id: LineageArtifactId) -> RetainedLineageView<'a> {
        self.graph
            .diagnostics_state()
            .lineage_records_for_artifact(artifact_id)
            .map(|records| RetainedLineageView::new(records, 0, records.len()))
            .unwrap_or_else(RetainedLineageView::empty)
    }

    pub fn current_lineage_artifact(&self, node: NodeId) -> Option<LineageArtifactId> {
        self.graph.node_lineage_artifact_id(node).ok().flatten()
    }

    pub fn lineage_chain_for_artifact(
        &self,
        artifact_id: LineageArtifactId,
    ) -> SynthesizedLineageChain {
        let mut chain = Vec::new();
        let mut current = Some(artifact_id);
        let mut visited = std::collections::BTreeSet::new();
        while let Some(artifact_id) = current {
            if !visited.insert(artifact_id) {
                break;
            }
            let artifact_records = self
                .graph
                .diagnostics_state()
                .lineage_records_for_artifact(artifact_id)
                .map(|records| records.iter().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            if artifact_records.is_empty() {
                break;
            }
            current = artifact_records.iter().find_map(|record| {
                record
                    .parent_artifact_id()
                    .filter(|parent| *parent != artifact_id)
            });
            chain.extend(artifact_records);
        }
        SynthesizedLineageChain::new(chain)
    }

    pub fn lineage_chain_for_node(&self, node: NodeId) -> SynthesizedLineageChain {
        self.current_lineage_artifact(node)
            .map(|artifact_id| self.lineage_chain_for_artifact(artifact_id))
            .unwrap_or_default()
    }
}

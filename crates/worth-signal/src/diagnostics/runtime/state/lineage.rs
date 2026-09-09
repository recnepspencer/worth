use super::DiagnosticHistory;

use crate::data::handle::NodeId;
use crate::diagnostics::lineage::{LineageArtifactId, LineageRecord};

use super::DiagnosticsState;

impl DiagnosticsState {
    pub fn lineage_records(&self) -> &DiagnosticHistory<LineageRecord> {
        &self.lineage_records
    }

    pub fn lineage_records_for_artifact(
        &self,
        artifact_id: LineageArtifactId,
    ) -> Option<&DiagnosticHistory<LineageRecord>> {
        self.lineage_records_by_artifact.get(&artifact_id)
    }

    pub fn lineage_records_for_node(
        &self,
        node: NodeId,
    ) -> Option<&DiagnosticHistory<LineageRecord>> {
        self.lineage_records_by_node.get(&node)
    }

    pub fn allocate_lineage_artifact_id(&mut self) -> LineageArtifactId {
        let artifact_id = LineageArtifactId(self.next_lineage_artifact_id);
        self.next_lineage_artifact_id += 1;
        artifact_id
    }

    pub fn allocate_lineage_sequence(&mut self) -> u64 {
        let sequence = self.next_lineage_sequence;
        self.next_lineage_sequence += 1;
        sequence
    }

    pub fn lineage_allocator_state(&self) -> (u64, u64) {
        (self.next_lineage_artifact_id, self.next_lineage_sequence)
    }

    pub fn synchronize_lineage_allocator(
        &mut self,
        next_lineage_artifact_id: u64,
        next_lineage_sequence: u64,
    ) {
        self.next_lineage_artifact_id = self.next_lineage_artifact_id.max(next_lineage_artifact_id);
        self.next_lineage_sequence = self.next_lineage_sequence.max(next_lineage_sequence);
    }

    pub fn record_lineage_record(&mut self, record: LineageRecord) {
        if let Some(node) = record.node() {
            self.lineage_records_by_node
                .entry(node)
                .or_default()
                .push_back(record.clone())
                .expect("diagnostic history exhausted its private position space");
        }
        if let Some(artifact_id) = record.subject_artifact_id() {
            self.lineage_records_by_artifact
                .entry(artifact_id)
                .or_default()
                .push_back(record.clone())
                .expect("diagnostic history exhausted its private position space");
        }
        self.lineage_records
            .push_back(record)
            .expect("diagnostic history exhausted its private position space");
        let limit = self.installed_retention_budget.history_limit.max(1) * 32;
        while self.lineage_records.len() > limit {
            if let Some(record) = self.lineage_records.pop_front() {
                self.remove_lineage_record_from_index(&record);
            }
        }
    }
}

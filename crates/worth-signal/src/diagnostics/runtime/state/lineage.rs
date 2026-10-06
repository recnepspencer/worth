use super::DiagnosticHistory;

use crate::data::handle::NodeId;
use crate::data::persistent_ord_map::{PersistentOrdMap, RetainedMapMutationOutcome};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement, RetainedStoragePreparation as Work,
};
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
        let mut work = Work::new(usize::MAX);
        if let Some(node) = record.node() {
            append_index(&mut self.lineage_records_by_node, node, &record, &mut work);
        }
        if let Some(artifact_id) = record.subject_artifact_id() {
            append_index(
                &mut self.lineage_records_by_artifact,
                artifact_id,
                &record,
                &mut work,
            );
        }
        self.lineage_records
            .append_with_retained_capacity(record, Charge::from_bytes(u64::MAX), &mut work)
            .expect("diagnostic history exhausted its private position space");
        let limit = self.installed_retention_budget.history_limit.max(1) * 32;
        while self.lineage_records.len() > limit {
            if let Some(record) = self
                .lineage_records
                .evict_front_with_retained_charge(&mut work)
                .expect("ordinary lineage eviction accounting")
            {
                if let Some(node) = record.node() {
                    evict_index(&mut self.lineage_records_by_node, &node, &mut work);
                }
                if let Some(artifact) = record.subject_artifact_id() {
                    evict_index(&mut self.lineage_records_by_artifact, &artifact, &mut work);
                }
            }
        }
    }
}

fn append_index<K: Clone + Ord + RetainedStorageMeasurement>(
    index: &mut PersistentOrdMap<K, DiagnosticHistory<LineageRecord>>,
    key: K,
    record: &LineageRecord,
    work: &mut Work,
) {
    let mut history = index.get(&key).cloned().unwrap_or_default();
    history
        .append_with_retained_capacity(record.clone(), Charge::from_bytes(u64::MAX), work)
        .expect("ordinary lineage index position exhausted");
    expect_accounted(index.insert_with_retained_charge(key, history, work));
}

fn evict_index<K: Clone + Ord + RetainedStorageMeasurement>(
    index: &mut PersistentOrdMap<K, DiagnosticHistory<LineageRecord>>,
    key: &K,
    work: &mut Work,
) {
    let mut history = index.get(key).cloned().expect("lineage index has frame");
    history
        .evict_front_with_retained_charge(work)
        .expect("ordinary lineage index eviction accounting")
        .expect("lineage index has oldest frame");
    if history.is_empty() {
        expect_accounted(index.remove_with_retained_charge(key, work));
    } else {
        expect_accounted(index.insert_with_retained_charge(key.clone(), history, work));
    }
}

fn expect_accounted<R: std::fmt::Debug>(
    outcome: Result<
        RetainedMapMutationOutcome<R>,
        crate::data::persistent_ord_map::RetainedMapMutationDenial,
    >,
) {
    match outcome.expect("ordinary lineage map was prepared") {
        RetainedMapMutationOutcome::Accounted { .. } => {}
        RetainedMapMutationOutcome::Unaccounted { denial, .. } => {
            panic!("ordinary lineage map charge failed: {denial:?}")
        }
    }
}

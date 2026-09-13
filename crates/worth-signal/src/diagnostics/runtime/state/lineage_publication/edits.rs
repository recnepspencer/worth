use super::*;
use crate::diagnostics::state::DiagnosticHistoryEditDenial;

impl LineagePublication {
    pub(super) fn append(
        &mut self,
        record: LineageRecord,
        maximum: Charge,
        work: &mut Work,
    ) -> Result<(), Denial> {
        if let Some(node) = record.node() {
            append_index(&mut self.nodes, node, &record, maximum, work)?;
        }
        if let Some(artifact) = record.subject_artifact_id() {
            append_index(&mut self.artifacts, artifact, &record, maximum, work)?;
        }
        self.records.admit_edit_work(work)?;
        self.records
            .append_with_retained_capacity(record, maximum, work)
            .map_err(|(_, denial)| map_history(denial))
    }

    pub(super) fn evict_to(&mut self, limit: usize, work: &mut Work) -> Result<(), Denial> {
        while self.records.len() > limit {
            self.records.admit_edit_work(work)?;
            let record = self
                .records
                .evict_front_with_retained_charge(work)
                .map_err(map_history)?
                .ok_or(Denial::Unprepared)?;
            if let Some(node) = record.node() {
                evict_index(&mut self.nodes, &node, &record, work)?;
            }
            if let Some(artifact) = record.subject_artifact_id() {
                evict_index(&mut self.artifacts, &artifact, &record, work)?;
            }
        }
        Ok(())
    }
}

fn append_index<K: Clone + Ord + RetainedStorageMeasurement>(
    index: &mut PersistentOrdMap<K, DiagnosticHistory<LineageRecord>>,
    key: K,
    record: &LineageRecord,
    maximum: Charge,
    work: &mut Work,
) -> Result<(), Denial> {
    work.reserve_visits(index.lookup_steps())?;
    let mut history = index.get(&key).cloned().unwrap_or_default();
    record.admit_copy_work(work)?;
    history.admit_edit_work(work)?;
    history
        .append_with_retained_capacity(record.clone(), maximum, work)
        .map_err(|(_, denial)| map_history(denial))?;
    admit_index_edit(index, work)?;
    accounted(index.insert_with_retained_charge(key, history, work))?;
    Ok(())
}

fn evict_index<K: Clone + Ord + RetainedStorageMeasurement>(
    index: &mut PersistentOrdMap<K, DiagnosticHistory<LineageRecord>>,
    key: &K,
    record: &LineageRecord,
    work: &mut Work,
) -> Result<(), Denial> {
    work.reserve_visits(index.lookup_steps())?;
    let mut history = index.get(key).cloned().ok_or(Denial::Unprepared)?;
    history.admit_edit_work(work)?;
    // Canonical global eviction must remove the oldest matching index frame.
    let front = history.front().ok_or(Denial::Unprepared)?;
    front.admit_comparison_work(record, work)?;
    if front != record {
        return Err(Denial::Unprepared);
    }
    history
        .evict_front_with_retained_charge(work)
        .map_err(map_history)?;
    admit_index_edit(index, work)?;
    if history.is_empty() {
        accounted(index.remove_with_retained_charge(key, work))?;
    } else {
        accounted(index.insert_with_retained_charge(key.clone(), history, work))?;
    }
    Ok(())
}

fn map_history(denial: DiagnosticHistoryEditDenial) -> Denial {
    match denial {
        DiagnosticHistoryEditDenial::Accounting(denial) => Denial::Accounting(denial),
        _ => Denial::Unprepared,
    }
}

/// Only NodeId and LineageArtifactId instantiate this private index editor.
/// Their comparisons/copies are fixed-width. Each mutation includes both
/// accounting neighborhoods, membership queries, retirement/readmission, and
/// COW/split/merge paths. Thirty-two combined lookup passes bound those paths;
/// selected history payload traversal is separately charged by map accounting.
fn admit_index_edit<K: Clone + Ord>(
    index: &PersistentOrdMap<K, DiagnosticHistory<LineageRecord>>,
    work: &mut Work,
) -> Result<(), Denial> {
    work.reserve_visits(
        index
            .lookup_steps()
            .checked_mul(32)
            .ok_or(RetainedStoragePreparationDenial::ChargeOverflow)?,
    )?;
    Ok(())
}

//! Remove one row's posting memberships without retaining intermediate trees.
use super::super::admission::PreparedTreeEdits;
use super::{
    debit, index_capacity, Arc, CompanionPreflightStop, FactPosting, FactPostingKey, MarkState,
    OrdSet, RecordedSettlementIdentity, RetainedIndexAdmission, SettlementMarks,
};

pub(super) fn remove(
    state: &mut MarkState,
    identity: &Arc<RecordedSettlementIdentity>,
    row: &SettlementMarks,
    admission: &mut impl RetainedIndexAdmission,
) -> Result<(), CompanionPreflightStop> {
    let mut map_edits = PreparedTreeEdits::<Arc<FactPostingKey>, OrdSet<FactPosting>>::new();
    for (key, ordinals) in &row.posting_ordinals {
        admission.work(1)?;
        admission.key_read(key, state.postings.len())?;
        let Some(mut postings) = state.postings.get(key).cloned() else {
            continue;
        };
        let mut set_edits = PreparedTreeEdits::<FactPosting, ()>::new();
        for ordinal in ordinals {
            let posting = FactPosting {
                settlement: Arc::clone(identity),
                ordinal: *ordinal,
            };
            admission.work(1)?;
            set_edits.remove(postings.len(), admission)?;
            if postings.remove(&posting).is_some() {
                debit(&mut state.posting_count, 1)?;
            }
        }
        if postings.is_empty() {
            map_edits.key_remove(key, state.postings.len(), admission)?;
            state.postings.remove(key);
            let key_bytes = key
                .owned_payload_capacity_bytes()
                .and_then(|n| n.checked_add(index_capacity::arc_bytes::<FactPostingKey>()?))
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
            debit(&mut state.key_payload_bytes, key_bytes)?;
            // The removed bucket and its edited set are dropped here. Its old
            // image remains owned by the predecessor; no new set survives.
        } else {
            set_edits.retain(postings.len(), admission)?;
            map_edits.key_insert(key, state.postings.len(), admission)?;
            state.postings.insert(Arc::clone(key), postings);
        }
    }
    map_edits.retain(state.postings.len(), admission)
}

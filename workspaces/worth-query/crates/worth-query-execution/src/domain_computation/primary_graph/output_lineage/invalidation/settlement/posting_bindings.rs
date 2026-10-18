//! Versioned exact settlement membership in the one physical reverse index.
use super::super::admission::PreparedTreeEdits;
use super::*;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) struct PostingBindings
{
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) projection:
        Arc<PostingProjection>,
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) settlements:
        OrdSet<Arc<RecordedSettlementIdentity>>,
}

pub(super) fn bind(
    state: &mut MarkState,
    projection: &Arc<PostingProjection>,
    identity: &Arc<RecordedSettlementIdentity>,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();
    admission.ordered_read(state.posting_bindings.len())?;
    let mut binding = if let Some(existing) = state.posting_bindings.get(&projection.id) {
        if !Arc::ptr_eq(&existing.projection, projection) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        (**existing).clone()
    } else {
        install_physical(state, projection, admission)?;
        admission.record_index_bytes(
            index_capacity::retained_map_bytes::<Arc<RecordedSettlementIdentity>, ()>(0)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        PostingBindings {
            projection: Arc::clone(projection),
            settlements: OrdSet::new(),
        }
    };
    admission.index_edit::<Arc<RecordedSettlementIdentity>, ()>(binding.settlements.len())?;
    binding.settlements.insert(Arc::clone(identity));
    admission.index_bytes(
        index_capacity::arc_bytes::<PostingBindings>()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
    )?;
    admission.index_edit::<u64, Arc<PostingBindings>>(state.posting_bindings.len())?;
    state
        .posting_bindings
        .insert(projection.id, Arc::new(binding));
    Ok(())
}

fn install_physical(
    state: &mut MarkState,
    projection: &PostingProjection,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    let mut map_edits = PreparedTreeEdits::<Arc<FactPostingKey>, OrdSet<u64>>::new();
    for key in projection.ordinals.keys() {
        crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();
        admission.key_read(key, state.postings.len())?;
        let (stored, mut groups, new_key) = match state.postings.get_key_value(key) {
            Some((stored, groups)) => (Arc::clone(stored), groups.clone(), false),
            None => (Arc::clone(key), OrdSet::new(), true),
        };
        let mut edits = PreparedTreeEdits::<u64, ()>::new();
        edits.insert(groups.len(), admission)?;
        groups.insert(projection.id);
        edits.retain(groups.len(), admission)?;
        map_edits.key_insert(&stored, state.postings.len(), admission)?;
        state.postings.insert(stored, groups);
        state.posting_count = state
            .posting_count
            .checked_add(1)
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        if new_key {
            state.key_payload_bytes = state
                .key_payload_bytes
                .checked_add(super::postings::key_payload_bytes(key)?)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        }
    }
    map_edits.retain(state.postings.len(), admission)
}

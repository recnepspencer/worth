//! Versioned exact settlement membership in the one physical reverse index.
use super::super::admission::PreparedTreeEdits;
use super::*;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) enum PostingBindings
{
    Settlements(SettlementPostingBindings),
    QueryRead(super::super::query_coverage::QueryReadBinding),
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) struct SettlementPostingBindings
{
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) projection:
        Arc<PostingProjection>,
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) settlements:
        OrdSet<Arc<RecordedSettlementIdentity>>,
}
impl PostingBindings {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn settlements(
        &self,
    ) -> Option<&SettlementPostingBindings> {
        match self {
            Self::Settlements(binding) => Some(binding),
            Self::QueryRead(_) => None,
        }
    }
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
        let existing = existing
            .settlements()
            .ok_or(CompanionPreflightStop::SelectedSourceMismatch)?;
        if !Arc::ptr_eq(&existing.projection, projection) {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        }
        existing.clone()
    } else {
        install_physical(state, projection.id, &projection.ordinals, admission)?;
        admission.record_index_bytes(
            index_capacity::retained_map_bytes::<Arc<RecordedSettlementIdentity>, ()>(0)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        SettlementPostingBindings {
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
    state.posting_bindings.insert(
        projection.id,
        Arc::new(PostingBindings::Settlements(binding)),
    );
    Ok(())
}

pub(in super::super) fn install_physical(
    state: &mut MarkState,
    id: u64,
    keys: &im::OrdMap<Arc<FactPostingKey>, OrdSet<usize>>,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), CompanionPreflightStop> {
    let mut map_edits = PreparedTreeEdits::<Arc<FactPostingKey>, OrdSet<u64>>::new();
    for key in keys.keys() {
        crate::domain_computation::primary_graph::fact_verification_observation::posting_registration();
        admission.key_read(key, state.postings.len())?;
        let (stored, mut groups, new_key) = match state.postings.get_key_value(key) {
            Some((stored, groups)) => (Arc::clone(stored), groups.clone(), false),
            None => (Arc::clone(key), OrdSet::new(), true),
        };
        let mut edits = PreparedTreeEdits::<u64, ()>::new();
        edits.insert(groups.len(), admission)?;
        groups.insert(id);
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

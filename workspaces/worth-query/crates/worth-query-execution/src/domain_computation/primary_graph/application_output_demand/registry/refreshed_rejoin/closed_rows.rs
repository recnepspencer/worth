use super::super::*;
impl DemandRegistryState {
    pub(in super::super) fn prune_closed_note(
        &mut self,
        queued: super::super::cleanup::closed_notes::QueuedClosedRow,
        candidates: Vec<WorthQueryOutputDemandKey>,
    ) -> (
        Vec<super::super::DemandRecord>,
        Vec<std::sync::Arc<WorthQueryOutputDemandKey>>,
    ) {
        let note = self.take_closed_note(queued);
        note.prune(|state| {
            for key in &candidates {
                if let Some(mut row) = state.records.get_mut(key)
                    && row.terminal()
                    && row.interests == 0
                    && row.framework_required_count == 0
                    && row.prepared_prerequisite_claims == 0
                {
                    row.release_obligations();
                }
            }
            // The cascade can end the last stale owner of another row in this
            // occurrence. Its member is debt of this same paid visit too.
            let retired = state.retire_closed_rows(candidates.clone());
            for key in &candidates {
                state.remove_required_member_if_released(key);
            }
            retired
        })
    }
    pub(in super::super) fn disposition(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> Option<super::super::IdleRowDisposition> {
        let awaited = super::awaited_by_stale_owner(&self.records, key);
        self.records
            .get(key)
            .map(|row| row.idle_disposition(awaited))
    }
    pub(in super::super) fn release_newest_member(
        &mut self,
        key: &WorthQueryOutputDemandKey,
    ) -> super::super::required_members::DetachedRequiredMember {
        let newest = super::newest_of_occurrence(&self.records, key);
        newest.map_or_else(Default::default, |newest| {
            self.remove_required_member_if_released_detached(&newest)
        })
    }
}

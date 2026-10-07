use super::super::RecordPublicationDirector;

impl RecordPublicationDirector {
    /// Called after the normal rewrite guard has installed displaced-source
    /// retention. The live destination is no longer excess candidate space.
    pub(in crate::physical_runtime::record_serving::publication::director) fn release_copy_candidate_after_source_retained(
        &self,
    ) {
        let slot = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let Some(obligation) = slot.as_ref() else {
            return;
        };
        let mut state = obligation.lock().unwrap_or_else(|e| e.into_inner());
        if state.published_root.is_some() {
            drop(state.physical_growth.take());
        }
    }
}

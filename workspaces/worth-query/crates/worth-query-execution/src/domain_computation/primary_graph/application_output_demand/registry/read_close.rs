//! Refuse orphaning this interest's unfinished native publication.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputReadCloseDenial;
impl WorthQueryOutputDemandInterest {
    pub(in crate::domain_computation::primary_graph) fn require_read_closeable(
        &self,
    ) -> Result<(), WorthQueryOutputReadCloseDenial> {
        let state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = state.records.get(&self.key) else {
            return Ok(());
        };
        if record.interests == 1
            && record.held_successor.is_none()
            && matches!(&record.state, DemandState::Output(output)
                if matches!(output.checkpoint, Some(WorthQueryOutputCheckpoint::Published { .. } | WorthQueryOutputCheckpoint::Delivered { .. })))
        {
            return Err(WorthQueryOutputReadCloseDenial::NativePublicationRetained);
        }
        Ok(())
    }
}

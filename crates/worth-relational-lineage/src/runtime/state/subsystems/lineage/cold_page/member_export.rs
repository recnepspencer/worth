//! Export checks frames through the same cell that owns point-demand damage.
use super::*;
use worth_relational_storage::internal_port::storage::data::member_frame;

impl<D: ColdPageDescriptor, P> ColdPageCell<D, P> {
    pub(in super::super) fn export_member(
        &self,
        source: impl FnOnce() -> Result<Arc<dyn RelationalDurableColdSource>, Cause>,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let ColdPageState::Damaged(failure) = &*state {
            return Err(failure.cause);
        }
        let source = source()?;
        let result = self
            .read_frame(source.as_ref(), limit.min(D::BYTE_LIMIT))
            .and_then(|bytes| {
                member_frame::decode(&bytes, self.descriptor.member(), D::KIND)?;
                Ok(bytes)
            });
        if let Err(cause) = result
            && cause.is_proven_damage()
        {
            *state = ColdPageState::Damaged(self.descriptor.storage_failure(cause));
        }
        result
    }
}

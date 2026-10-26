use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

impl RelationalDurableColdSource for super::TestDurableMedium {
    fn read_member(
        &self,
        _locator: RelationalDurableMemberLocator,
        _limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        // This medium currently files embedded checkpoints, with no external members.
        Err(Cause::SourceUnavailable)
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        let journal = self.journal();
        let bytes = journal.bytes_at(position).ok_or(Cause::SourceUnavailable)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.to_vec())
    }
}

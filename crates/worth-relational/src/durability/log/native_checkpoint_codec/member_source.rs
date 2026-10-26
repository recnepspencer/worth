use std::collections::BTreeMap;
use std::sync::Arc;

use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

/// Inline artifacts implement the same read port as Store. This keeps the
/// member verification and loading path independent of artifact placement.
#[derive(Debug)]
pub(super) struct InlineCheckpointSource {
    pub(super) members: BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    pub(super) envelopes: BTreeMap<PatchStreamPosition, Arc<[u8]>>,
}

impl RelationalDurableColdSource for InlineCheckpointSource {
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        bounded(self.members.get(&locator), limit)
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        bounded(self.envelopes.get(&position), limit)
    }
}

fn bounded(bytes: Option<&Arc<[u8]>>, limit: u64) -> Result<Vec<u8>, Cause> {
    let bytes = bytes.ok_or(Cause::SourceUnavailable)?;
    if bytes.len() as u64 > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(bytes.len())
        .map_err(|_| Cause::MemoryExhausted)?;
    result.extend_from_slice(bytes);
    Ok(result)
}

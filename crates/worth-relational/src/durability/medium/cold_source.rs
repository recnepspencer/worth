use serde::{Deserialize, Serialize};

use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause;

/// Content identity of a checkpoint member. The home resolves physical
/// placement through its currently selected checkpoint's member table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationalDurableMemberLocator {
    pub sha256: [u8; 32],
    pub length: u64,
}

/// Read-only committed storage, injected before recovery readmission. A fork
/// may share this port without acquiring any publication authority.
pub trait RelationalDurableColdSource: Send + Sync + std::fmt::Debug {
    /// Retain selected placement without reading member payloads. The provider
    /// must preserve it across checkpoint replacement until the last owner
    /// drops. This makes no claim about payload availability or integrity.
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<super::RelationalDurableMemberCustody, RelationalStorageUnavailableCause>;

    /// Return member bytes only after enforcing the caller's byte limit.
    /// Relational independently checks the frame, content identity and schema.
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause>;

    /// Return the canonical artifact at exactly this position. Missing durable
    /// content refuses; implementations must never return an empty substitute.
    fn read_commit(
        &self,
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause>;
}

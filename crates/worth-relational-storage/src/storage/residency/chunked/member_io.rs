//! The durable member ports chunk producers write to and cold readers read
//! from, and bounded whole-member loading under the in-flight fill ledger.
use std::sync::Arc;

use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::{WorkingSetAccountant, WorkingSetCharge};

/// Receives sealed, content-addressed members.
pub trait ChunkMemberSink {
    fn put(&mut self, locator: RelationalDurableMemberLocator, bytes: Vec<u8>)
        -> Result<(), Cause>;
}

/// Reads committed members by byte range; never returns a whole member.
pub trait ChunkMemberSource {
    /// Copies bytes of `member` starting at `offset` into `target`. Returns
    /// zero only at the end of the member.
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause>;

    /// Data-pointer identity of an underlying `RelationalDurableColdSource`, if any.
    fn cold_source_identity(&self) -> Option<usize> {
        None
    }
}

/// A member's bytes together with the fill charge that funds them.
pub struct LoadedMember {
    pub bytes: Vec<u8>,
    pub fill: WorkingSetCharge,
}

/// Loads one bounded member (a page or a chunk image). Its encoded bytes are
/// charged to `max_in_flight_fill_bytes` before they are allocated, so
/// concurrent cold fills cannot exceed the in-flight ceiling.
pub fn load_member(
    accountant: &Arc<WorkingSetAccountant>,
    source: &dyn ChunkMemberSource,
    member: RelationalDurableMemberLocator,
) -> Result<LoadedMember, Cause> {
    let fill = accountant
        .reserve_fill(member.length)
        .map_err(|_| Cause::MemoryExhausted)?;
    let length = usize::try_from(member.length).map_err(|_| Cause::MemoryExhausted)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| Cause::MemoryExhausted)?;
    bytes.resize(length, 0);
    let mut filled = 0;
    while filled < length {
        match source.read_range(member, filled as u64, &mut bytes[filled..])? {
            0 => return Err(Cause::MissingMember),
            read => filled += read,
        }
    }
    Ok(LoadedMember { bytes, fill })
}

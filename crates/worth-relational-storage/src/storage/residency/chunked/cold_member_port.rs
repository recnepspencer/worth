//! Production ports between chunked/paged members and a checkpoint's durable
//! medium: a reader over the selected cold source and a collecting sink for
//! capture.
use std::sync::Arc;

use super::member_io::{ChunkMemberSink, ChunkMemberSource};
use crate::durability::medium::cold_source::RelationalDurableColdSource;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

/// Reads paged members from the checkpoint's selected cold source. A page is a
/// small bounded member, so a range read reads it whole and copies the range.
#[derive(Debug, Clone)]
pub struct ColdMemberSource {
    source: Arc<dyn RelationalDurableColdSource>,
    member_limit: u64,
}

impl ColdMemberSource {
    pub fn new(source: Arc<dyn RelationalDurableColdSource>, member_limit: u64) -> Self {
        Self {
            source,
            member_limit,
        }
    }
}

impl ChunkMemberSource for ColdMemberSource {
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause> {
        let bytes = self.source.read_member(member, self.member_limit)?;
        let start = usize::try_from(offset).map_err(|_| Cause::Damaged)?;
        let tail = bytes.get(start..).ok_or(Cause::Damaged)?;
        let count = tail.len().min(target.len());
        target[..count].copy_from_slice(&tail[..count]);
        Ok(count)
    }

    fn cold_source_identity(&self) -> Option<usize> {
        Some((self.source.as_ref() as *const dyn RelationalDurableColdSource).cast::<()>() as usize)
    }
}

/// Collects sealed members during capture, in write order, without duplicates.
#[derive(Debug, Default)]
pub struct CollectedMembers {
    pub members: Vec<(RelationalDurableMemberLocator, Vec<u8>)>,
}

impl ChunkMemberSink for CollectedMembers {
    fn put(
        &mut self,
        locator: RelationalDurableMemberLocator,
        bytes: Vec<u8>,
    ) -> Result<(), Cause> {
        if !self.members.iter().any(|(known, _)| *known == locator) {
            self.members.push((locator, bytes));
        }
        Ok(())
    }
}

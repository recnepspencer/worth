//! Production ports between chunked/paged members and a checkpoint's durable
//! medium: a reader over the selected cold source and a collecting sink for
//! capture.
use std::sync::{Arc, Mutex};

use super::member_io::{ChunkMemberSink, ChunkMemberSource};
use crate::durability::medium::cold_source::RelationalDurableColdSource;
use crate::durability::medium::member_custody::native_slot::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
};
use crate::durability::medium::member_custody::native_source_slot::NativeSourceMemberState;
use crate::durability::medium::member_custody::retirement::{
    RelationalDurableMemberRetirementObservation as Observation,
    RelationalDurableMemberRetirementRefusal as Refusal,
};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

#[derive(Debug)]
enum SourceCustody {
    Direct(Arc<dyn RelationalDurableColdSource>),
    Managed(Mutex<NativeSourceMemberState>),
}

/// Reads paged members from the checkpoint's selected cold source. A page is a
/// small bounded member, so a range read reads it whole and copies the range.
#[derive(Debug)]
pub struct ColdMemberSource {
    custody: SourceCustody,
    member_limit: u64,
}

impl ColdMemberSource {
    pub fn new(source: Arc<dyn RelationalDurableColdSource>, member_limit: u64) -> Self {
        Self {
            custody: SourceCustody::Direct(source),
            member_limit,
        }
    }

    pub fn with_custody(physical: NativeSourceMemberState, member_limit: u64) -> Self {
        Self {
            custody: SourceCustody::Managed(Mutex::new(physical)),
            member_limit,
        }
    }

    fn active_source(&self) -> Result<Arc<dyn RelationalDurableColdSource>, Cause> {
        match &self.custody {
            SourceCustody::Direct(source) => Ok(Arc::clone(source)),
            SourceCustody::Managed(state) => state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .source()
                .cloned()
                .ok_or(Cause::SourceUnavailable),
        }
    }

    pub(crate) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let token = match &self.custody {
            SourceCustody::Direct(direct) => {
                if !std::ptr::addr_eq(direct.as_ref(), source) {
                    return Err(Denial::Refused(Refusal::ForeignSource));
                }
                None
            }
            SourceCustody::Managed(state) => {
                let guard = state.try_lock().map_err(|_| Denial::Busy)?;
                guard.preflight_snapshot(source)?
            }
        };
        if let Some(token) = token {
            token
                .preflight_retirement(source)
                .map_err(Denial::Refused)?;
        }
        Ok(())
    }

    pub(crate) fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        match &self.custody {
            SourceCustody::Direct(_) => Ok(true),
            SourceCustody::Managed(state) => {
                let mut guard = state.try_lock().map_err(|_| Denial::Busy)?;
                guard.loan(handoff)
            }
        }
    }

    pub(crate) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        match &self.custody {
            SourceCustody::Direct(_) => Err(Denial::Refused(Refusal::ForeignCustody)),
            SourceCustody::Managed(state) => {
                let mut guard = state.try_lock().map_err(|_| Denial::Busy)?;
                guard.return_handoff(handoff, source)
            }
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
        let source = self.active_source()?;
        let bytes = source.read_member(member, self.member_limit)?;
        let start = usize::try_from(offset).map_err(|_| Cause::Damaged)?;
        let tail = bytes.get(start..).ok_or(Cause::Damaged)?;
        let count = tail.len().min(target.len());
        target[..count].copy_from_slice(&tail[..count]);
        Ok(count)
    }
}

/// Confines chunk-member reads to an admitted placement's member roster so a
/// tampered continuation or directory page cannot fetch an unlisted locator
/// from a shared backing source.
#[derive(Debug)]
pub struct ConfinedMemberSource<S> {
    inner: S,
    allowed: Vec<RelationalDurableMemberLocator>,
}

impl<S> ConfinedMemberSource<S> {
    pub fn new(inner: S, mut allowed: Vec<RelationalDurableMemberLocator>) -> Self {
        allowed.sort_unstable();
        allowed.dedup();
        Self { inner, allowed }
    }
}

impl<S: ChunkMemberSource> ChunkMemberSource for ConfinedMemberSource<S> {
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause> {
        if self.allowed.binary_search(&member).is_err() {
            return Err(Cause::Damaged);
        }
        self.inner.read_range(member, offset, target)
    }
}

/// Collects sealed members during capture, in write order, without duplicates.
#[derive(Debug, Default)]
pub struct CollectedMembers {
    pub members: Vec<(RelationalDurableMemberLocator, Vec<u8>)>,
    seen: std::collections::BTreeSet<RelationalDurableMemberLocator>,
}

impl CollectedMembers {
    pub fn contains(&self, locator: &RelationalDurableMemberLocator) -> bool {
        self.seen.contains(locator)
    }
}

impl ChunkMemberSink for CollectedMembers {
    fn put(
        &mut self,
        locator: RelationalDurableMemberLocator,
        bytes: Vec<u8>,
    ) -> Result<(), Cause> {
        if self.seen.insert(locator) {
            self.members.push((locator, bytes));
        }
        Ok(())
    }
}

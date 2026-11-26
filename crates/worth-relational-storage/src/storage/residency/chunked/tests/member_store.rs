//! An in-memory committed member store and the fixtures the courts share.
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};

use super::super::layout::ChunkLayout;
use super::super::member_io::{ChunkMemberSink, ChunkMemberSource};
use super::super::sealing::{seal_chunk_image, ChunkSealRequest, SealedChunk};
use super::super::slot_projection::{AdjacencyEdge, SlotProjection, SlotVersion};
use super::super::slot_range::{SlotDomain, SlotRange};
use crate::config::data::ResidencyBudget;
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::identity::data::{KindId, PartitionId, RelationId};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::residency::working_set::WorkingSetAccountant;

/// Content-addressed members; `put` refuses bytes that do not match their locator.
#[derive(Default)]
pub(super) struct MemberStore {
    members: Mutex<HashMap<RelationalDurableMemberLocator, Arc<Vec<u8>>>>,
}

impl MemberStore {
    pub(super) fn replace(&self, locator: RelationalDurableMemberLocator, bytes: Vec<u8>) {
        self.members
            .lock()
            .unwrap()
            .insert(locator, Arc::new(bytes));
    }

    pub(super) fn bytes(&self, locator: RelationalDurableMemberLocator) -> Arc<Vec<u8>> {
        Arc::clone(&self.members.lock().unwrap()[&locator])
    }

    pub(super) fn put_raw(&self, bytes: Vec<u8>) -> RelationalDurableMemberLocator {
        let locator = RelationalDurableMemberLocator {
            sha256: Sha256::digest(&bytes).into(),
            length: bytes.len() as u64,
        };
        self.replace(locator, bytes);
        locator
    }
}

impl ChunkMemberSink for &MemberStore {
    fn put(
        &mut self,
        locator: RelationalDurableMemberLocator,
        bytes: Vec<u8>,
    ) -> Result<(), Cause> {
        if <[u8; 32]>::from(Sha256::digest(&bytes)) != locator.sha256 {
            return Err(Cause::IntegrityMismatch);
        }
        self.replace(locator, bytes);
        Ok(())
    }
}

impl ChunkMemberSource for MemberStore {
    fn read_range(
        &self,
        member: RelationalDurableMemberLocator,
        offset: u64,
        target: &mut [u8],
    ) -> Result<usize, Cause> {
        let bytes = self
            .members
            .lock()
            .unwrap()
            .get(&member)
            .cloned()
            .ok_or(Cause::MissingMember)?;
        let start = usize::try_from(offset)
            .map_err(|_| Cause::Damaged)?
            .min(bytes.len());
        let count = target.len().min(bytes.len() - start);
        target[..count].copy_from_slice(&bytes[start..start + count]);
        Ok(count)
    }
}

/// A small chunk geometry so scale courts stay fast.
pub(super) const LAYOUT: ChunkLayout = ChunkLayout {
    chunk_slots: 64,
    slot_inline_budget: 256,
    continuation_page_bytes: 1024,
    directory_page_entries: 8,
    max_chunk_image_bytes: 1024 * 1024,
};

pub(super) fn accountant(
    pool_bytes: u64,
    fill_bytes: u64,
    window: u32,
) -> Arc<WorkingSetAccountant> {
    WorkingSetAccountant::new(ResidencyBudget {
        head_working_set_bytes: pool_bytes,
        historical_working_set_bytes: pool_bytes,
        max_in_flight_fill_bytes: fill_bytes,
        streaming_window_bytes: window,
    })
}

pub(super) fn version(kind: u32, marker: u64) -> SlotVersion {
    SlotVersion {
        kind: KindId::new(kind),
        payload: marker.to_be_bytes().repeat(4),
    }
}

pub(super) fn edge(relation_kind: u32, slot: u64) -> AdjacencyEdge {
    AdjacencyEdge {
        relation_kind: KindId::new(relation_kind),
        relation: RelationId::new(PartitionId::main(), slot, 1),
    }
}

/// One slot as plain data, replayable into a fresh projection.
#[derive(Clone)]
pub(super) struct SlotFixture {
    pub(super) slot: u64,
    pub(super) head: Option<SlotVersion>,
    pub(super) history: Vec<SlotVersion>,
    pub(super) adjacency: Vec<AdjacencyEdge>,
}

impl SlotFixture {
    pub(super) fn projection(&self) -> SlotProjection<'_> {
        SlotProjection {
            slot: self.slot,
            head: self.head.clone(),
            history: Box::new(self.history.iter().cloned()),
            adjacency: Box::new(self.adjacency.iter().copied()),
        }
    }
}

pub(super) fn seal(
    store: &MemberStore,
    domain: SlotDomain,
    domain_slots: u64,
    slot_range: SlotRange,
    slots: &[SlotFixture],
) -> SealedChunk {
    seal_chunk_image(
        ChunkSealRequest {
            layout: LAYOUT,
            domain,
            domain_slots,
            slot_range,
            slots: Box::new(slots.iter().map(SlotFixture::projection)),
            claimed: None,
        },
        &mut &*store,
    )
    .expect("an honest producer seals")
}

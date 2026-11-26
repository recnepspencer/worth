//! `seal_region_chunks`: the live producer of a region leaf's chunk placement.
//!
//! Each slot domain of the captured partition is cut into the fixed `REG8`
//! partition of `[0, domain)`, where `domain` is one past the highest
//! occupied logical slot, and every chunk is sealed by the single producer
//! owner [`seal_chunk_image`]. A slot's canonical head is its current record
//! kind with a bounded payload: the record generation and the digest of its
//! current portable row (lifecycle, creation and retirement versions, current
//! metadata and extra fields). Entity slots stream their current outgoing
//! adjacency as `(relation kind, relation)` edges; oversized adjacency spills
//! into continuation pages. Record history stays in the region's layered
//! history members: under head-only residency the full canonical history is
//! not resident at capture, so a chunk commits to no history versions and its
//! history summary is empty.
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::cold_member_port::CollectedMembers;
use super::member_io::ChunkMemberSink;
use super::region_placement::{RegionChunkPlacement, REGION_CHUNK_LAYOUT};
use super::sealing::{seal_chunk_image, ChunkSealRefusal, ChunkSealRequest};
use super::semantic_entry::root_semantic_merkle;
use super::slot_projection::{AdjacencyEdge, SlotProjection, SlotVersion};
use super::slot_range::{fixed_partition, SlotDomain};
use crate::durability::medium::RelationalDurableMemberLocator;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::{AdjacencyKindBasis, SparseAdjacencyTable};
use crate::storage::residency::region_capture::complete::arena::WireKind;
use crate::storage::substrate::RecordArena;

/// A sealed placement and the new members its chunks reference.
#[derive(Debug)]
pub struct SealedRegionChunks {
    pub placement: RegionChunkPlacement,
    pub members: Vec<(RelationalDurableMemberLocator, Vec<u8>)>,
}

#[derive(Debug)]
pub enum RegionChunkCaptureError<E> {
    /// The capture's own admission stopped.
    Stop(E),
    Unavailable(Cause),
}

impl<E> From<E> for RegionChunkCaptureError<E> {
    fn from(stop: E) -> Self {
        Self::Stop(stop)
    }
}

/// Seals `partition`'s chunks. `charge(units, bytes)` admits work units per
/// chunk, occupied slot and outgoing adjacency edge before sealing, and admits
/// each new member's bytes incrementally before storing it.
pub fn seal_region_chunks<E>(
    partition: &PartitionState,
    charge: &mut dyn FnMut(u64, u64) -> Result<(), E>,
) -> Result<SealedRegionChunks, RegionChunkCaptureError<E>> {
    let mut chunks = Vec::new();
    let mut pass = DomainPass {
        sink: ChargingSink {
            collected: CollectedMembers::default(),
            charge,
            stop: None,
        },
        chunks: &mut chunks,
    };
    let entity_domain = pass.seal(
        SlotDomain::Entity,
        &partition.entity_arena,
        Some(&partition.adjacency),
    )?;
    let relation_domain = pass.seal(SlotDomain::Relation, &partition.relation_arena, None)?;
    let sink = pass.sink.collected;
    let entries = chunks
        .iter()
        .map(|(semantic, _)| semantic.clone())
        .collect::<Vec<_>>();
    let mut members = sink
        .members
        .iter()
        .map(|(locator, _)| *locator)
        .collect::<Vec<_>>();
    members.sort_unstable();
    Ok(SealedRegionChunks {
        placement: RegionChunkPlacement {
            entity_domain,
            relation_domain,
            root_semantic_merkle: root_semantic_merkle(&entries),
            chunks,
            members,
        },
        members: sink.members,
    })
}

/// The storage cause a sealing refusal reports to a live capture.
pub fn seal_refusal_cause(refusal: ChunkSealRefusal) -> Cause {
    match refusal {
        ChunkSealRefusal::OversizeChunk { length, limit } => Cause::LimitExceeded { length, limit },
        ChunkSealRefusal::Member(cause) => cause,
        _ => Cause::Damaged,
    }
}

struct ChargingSink<'p, E> {
    collected: CollectedMembers,
    charge: &'p mut dyn FnMut(u64, u64) -> Result<(), E>,
    stop: Option<E>,
}

impl<E> ChunkMemberSink for ChargingSink<'_, E> {
    fn put(
        &mut self,
        locator: RelationalDurableMemberLocator,
        bytes: Vec<u8>,
    ) -> Result<(), Cause> {
        if self.collected.contains(&locator) {
            return Ok(());
        }
        if let Err(stop) = (self.charge)(0, bytes.len() as u64) {
            self.stop = Some(stop);
            return Err(Cause::MemoryExhausted);
        }
        self.collected.put(locator, bytes)
    }
}

struct DomainPass<'p, E> {
    sink: ChargingSink<'p, E>,
    chunks: &'p mut Vec<(
        super::semantic_entry::ChunkSemanticEntry,
        super::locator_binding::ChunkLocatorBinding,
    )>,
}

impl<E> DomainPass<'_, E> {
    /// Seals one domain's fixed chunks and returns the domain bound.
    fn seal<K: WireKind>(
        &mut self,
        domain: SlotDomain,
        arena: &RecordArena<K>,
        adjacency: Option<&SparseAdjacencyTable>,
    ) -> Result<u64, RegionChunkCaptureError<E>> {
        let domain_slots = match arena.occupied_slots_iter().last() {
            None => 0,
            Some(slot) => u64::try_from(slot)
                .ok()
                .and_then(|s| s.checked_add(1))
                .ok_or(RegionChunkCaptureError::Unavailable(Cause::Damaged))?,
        };
        let mut occupied = arena
            .occupied_slots_iter()
            .map(|slot| slot as u64)
            .peekable();
        for slot_range in fixed_partition(domain_slots, REGION_CHUNK_LAYOUT.chunk_slots) {
            let mut slots = Vec::new();
            while let Some(slot) = occupied.next_if(|slot| slot_range.contains(*slot)) {
                slots.push(slot);
            }
            let mut work_units = (slots.len() as u64).saturating_add(1);
            if let Some(table) = adjacency {
                for &slot in &slots {
                    if let Some(set) = usize::try_from(slot).ok().and_then(|i| table.get(i)) {
                        for _ in set.kind_memberships(AdjacencyKindBasis::Current) {
                            work_units = work_units.saturating_add(1);
                            if work_units >= 1024 {
                                (self.sink.charge)(work_units, 0)
                                    .map_err(RegionChunkCaptureError::Stop)?;
                                work_units = 0;
                            }
                        }
                    }
                }
            }
            if work_units > 0 {
                (self.sink.charge)(work_units, 0).map_err(RegionChunkCaptureError::Stop)?;
            }
            let heads = slots
                .iter()
                .map(|slot| head_version(arena, *slot))
                .collect::<Result<Vec<_>, _>>()
                .map_err(RegionChunkCaptureError::Unavailable)?;
            let projections = slots.iter().zip(heads).map(|(&slot, head)| SlotProjection {
                slot,
                head,
                history: Box::new(std::iter::empty()),
                adjacency: Box::new(
                    adjacency
                        .and_then(|table| table.get(slot as usize))
                        .into_iter()
                        .flat_map(|set| set.kind_memberships(AdjacencyKindBasis::Current))
                        .map(|(relation_kind, relation)| AdjacencyEdge {
                            relation_kind,
                            relation,
                        }),
                ),
            });
            let sealed = match seal_chunk_image(
                ChunkSealRequest {
                    layout: REGION_CHUNK_LAYOUT,
                    domain,
                    domain_slots,
                    slot_range,
                    slots: Box::new(projections),
                    claimed: None,
                },
                &mut self.sink,
            ) {
                Ok(sealed) => sealed,
                Err(refusal) => {
                    if let Some(stop) = self.sink.stop.take() {
                        return Err(RegionChunkCaptureError::Stop(stop));
                    }
                    return Err(RegionChunkCaptureError::Unavailable(seal_refusal_cause(
                        refusal,
                    )));
                }
            };
            self.chunks.push((sealed.semantic, sealed.binding));
        }
        Ok(domain_slots)
    }
}

/// The slot's current record version, or `None` for a row without a kind.
fn head_version<K: WireKind>(
    arena: &RecordArena<K>,
    slot: u64,
) -> Result<Option<SlotVersion>, Cause> {
    let physical = arena
        .physical_index(usize::try_from(slot).map_err(|_| Cause::Damaged)?)
        .ok_or(Cause::Damaged)?;
    let rows = &arena.metadata_history[physical];
    let (Some(kind), Some(last)) = (arena.kind_ids[physical], rows.len().checked_sub(1)) else {
        return Ok(None);
    };
    let row = CurrentRow::<K> {
        lifecycle: &arena.lifecycle[physical],
        created_at: &arena.created_at[physical],
        retired_at: &arena.retired_at[physical],
        meta: Meta::<K>(&rows[last]),
        extra: Extra::<K>(&arena.extra[physical]),
    };
    let mut digest = DigestWriter(Sha256::new());
    rmp_serde::encode::write(&mut digest, &row).map_err(|_| Cause::Damaged)?;
    let mut payload = arena.generations[physical].to_be_bytes().to_vec();
    payload.extend_from_slice(&digest.0.finalize());
    Ok(Some(SlotVersion { kind, payload }))
}

#[derive(Serialize)]
#[serde(bound = "")]
struct CurrentRow<'a, K: WireKind> {
    lifecycle: &'a crate::storage::data::RecordLifecycleState,
    created_at: &'a crate::identity::data::VersionId,
    retired_at: &'a Option<crate::identity::data::VersionId>,
    meta: Meta<'a, K>,
    extra: Extra<'a, K>,
}

struct Meta<'a, K: WireKind>(&'a K::Meta);
impl<K: WireKind> Serialize for Meta<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        K::serialize_meta(self.0, serializer)
    }
}

struct Extra<'a, K: WireKind>(&'a K::Extra);
impl<K: WireKind> Serialize for Extra<'_, K> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        K::serialize_extra(self.0, serializer)
    }
}

struct DigestWriter(Sha256);
impl std::io::Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

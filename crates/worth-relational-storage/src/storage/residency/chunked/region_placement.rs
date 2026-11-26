//! The chunked placement a `CHK24` region leaf records.
//!
//! A placement carries each chunk's consolidation-invariant semantic entry,
//! the checkpoint-authenticated locator binding of its members, the
//! `root_semantic_merkle` over the ordered entries, and the roster of every
//! member the bindings reach (chunk images, continuation pages and
//! continuation-directory pages), so checkpoint custody pins them all without
//! reading any member. Admission proves exact fixed-partition coverage of the
//! leaf's slot domains, root agreement and roster bounds before any reader
//! trusts a negative chunk summary.
use serde::{Deserialize, Serialize};

use super::canonical_read::ChunkResidency;
use super::chunk_directory::{ChunkCoverageRefusal, RegionChunkDirectory};
use super::consolidation::{
    consolidate_history_chunk, flatten_history_image, ChunkVerificationRefusal,
};
use super::layout::ChunkLayout;
use super::locator_binding::ChunkLocatorBinding;
use super::member_io::{ChunkMemberSink, ChunkMemberSource};
use super::semantic_entry::{root_semantic_merkle, ChunkSemanticEntry};
use crate::durability::medium::RelationalDurableMemberLocator;

/// The one chunk geometry of `REG8` region leaves.
pub const REGION_CHUNK_LAYOUT: ChunkLayout = ChunkLayout::REG8;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionChunkPlacement {
    /// One past the highest occupied logical entity slot (zero when none).
    pub entity_domain: u64,
    /// One past the highest occupied logical relation slot (zero when none).
    pub relation_domain: u64,
    pub root_semantic_merkle: [u8; 32],
    /// Entity chunks ascending, then relation chunks ascending.
    pub chunks: Vec<(ChunkSemanticEntry, ChunkLocatorBinding)>,
    /// Every member the bindings reach, strictly ascending.
    pub members: Vec<RelationalDurableMemberLocator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkPlacementRefusal {
    Coverage(ChunkCoverageRefusal),
    /// The recorded root is not the Merkle root of the recorded entries.
    RootMismatch,
    /// The roster is unsorted or duplicated, omits a bound member, holds an
    /// oversized member, or holds more members than the bindings can reach.
    Roster,
}

impl RegionChunkPlacement {
    /// The placement of a region with no occupied slot.
    pub fn empty() -> Self {
        Self {
            entity_domain: 0,
            relation_domain: 0,
            root_semantic_merkle: root_semantic_merkle(&[]),
            chunks: Vec::new(),
            members: Vec::new(),
        }
    }

    /// Admits this placement over its recorded logical slot domains. Reads no
    /// member. The domains are not compared with the leaf's physical row
    /// counts: a withdrawn sparse-overlay row keeps its physical row while its
    /// logical slot is unmapped, so rows may exceed occupied slots.
    pub fn admit(&self) -> Result<RegionChunkDirectory, ChunkPlacementRefusal> {
        let directory = RegionChunkDirectory::admit(
            REGION_CHUNK_LAYOUT,
            self.entity_domain,
            self.relation_domain,
            self.chunks.iter().cloned(),
        )
        .map_err(ChunkPlacementRefusal::Coverage)?;
        if directory.root_semantic_merkle() != self.root_semantic_merkle {
            return Err(ChunkPlacementRefusal::RootMismatch);
        }
        self.admit_roster()?;
        Ok(directory)
    }

    fn admit_roster(&self) -> Result<(), ChunkPlacementRefusal> {
        let limit = REGION_CHUNK_LAYOUT.max_chunk_image_bytes as u64;
        let ascending = self.members.windows(2).all(|pair| pair[0] < pair[1]);
        if !ascending || self.members.iter().any(|member| member.length > limit) {
            return Err(ChunkPlacementRefusal::Roster);
        }
        let held =
            |member: &RelationalDurableMemberLocator| self.members.binary_search(member).is_ok();
        // Each chunk reaches its two images, one continuation page per
        // continuation entry, and at most two directory pages per entry plus
        // the root (every directory page holds at least one entry).
        let mut reachable = 0_u64;
        for (_, binding) in &self.chunks {
            let continuation = binding.continuation_directory_locator();
            let bound = [binding.head_locator(), binding.history_locator()]
                .into_iter()
                .chain(continuation.map(|root| root.root_page));
            if !bound.into_iter().all(|member| held(&member)) {
                return Err(ChunkPlacementRefusal::Roster);
            }
            let pages = continuation.map_or(0, |root| {
                root.entry_count.saturating_mul(3).saturating_add(1)
            });
            reachable = reachable.saturating_add(2).saturating_add(pages);
        }
        if self.members.len() as u64 > reachable {
            return Err(ChunkPlacementRefusal::Roster);
        }
        Ok(())
    }

    /// Flattens every chunk's history into a new member and rebinds it only
    /// after re-deriving the bit-identical semantic entry, so the returned
    /// placement keeps `root_semantic_merkle` exactly.
    pub fn consolidate_history(
        &self,
        directory: &RegionChunkDirectory,
        residency: &ChunkResidency,
        source: &dyn ChunkMemberSource,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<Self, ChunkVerificationRefusal> {
        let mut consolidated = directory.clone();
        for chunk in directory.chunks() {
            let candidate = flatten_history_image(
                residency,
                source,
                sink,
                REGION_CHUNK_LAYOUT,
                chunk.binding(),
            )
            .map_err(ChunkVerificationRefusal::Member)?;
            let rebound = consolidate_history_chunk(
                residency,
                source,
                chunk.semantic(),
                chunk.binding(),
                candidate,
            )?;
            consolidated
                .install_consolidated(rebound)
                .map_err(|_| ChunkVerificationRefusal::RangeMismatch)?;
        }
        let replaced = directory
            .chunks()
            .iter()
            .map(|chunk| chunk.binding().history_locator());
        let installed = consolidated
            .chunks()
            .iter()
            .map(|chunk| chunk.binding().history_locator())
            .collect::<std::collections::BTreeSet<_>>();
        let mut members = self
            .members
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        for retired in replaced {
            members.remove(&retired);
        }
        members.extend(installed);
        let chunks = consolidated
            .chunks()
            .iter()
            .map(|chunk| (chunk.semantic().clone(), *chunk.binding()))
            .collect::<Vec<_>>();
        let root_semantic_merkle = root_semantic_merkle(
            &chunks
                .iter()
                .map(|(semantic, _)| semantic.clone())
                .collect::<Vec<_>>(),
        );
        debug_assert_eq!(root_semantic_merkle, self.root_semantic_merkle);
        Ok(Self {
            entity_domain: self.entity_domain,
            relation_domain: self.relation_domain,
            root_semantic_merkle,
            chunks,
            members: members.into_iter().collect(),
        })
    }
}

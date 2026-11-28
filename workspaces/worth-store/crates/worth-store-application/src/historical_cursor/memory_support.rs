//! Support types, O(1) per-basis member views, and restart simulation for
//! [`super::MemoryHistoricalBasisStore`].

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_relational::facade::history::CommitId;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::HistoricalReadDenial;
use worth_store::pin_table::{RootPinDropPolicy, RootPinId, RootPinState, VolatileRootPinTable};

use super::aux_index::AuxiliaryIndexRegistry;
use super::catalog::{HistoricalBasisCatalog, RetainedBranchIndex};
use super::chunk_format::{ChunkMemberId, SealedChunkDescriptor};
use super::stream::LoanedMemberReader;

/// O(1) shareable immutable member table for a single retained basis.
pub(super) type MemoryBasisMembers = Arc<BTreeMap<ChunkMemberId, Arc<[u8]>>>;

pub(super) struct MemoryCatalogState {
    pub(super) catalog: HistoricalBasisCatalog,
    pub(super) retained_branches: RetainedBranchIndex,
    pub(super) basis_manifests: BTreeMap<CommitId, Arc<[SealedChunkDescriptor]>>,
    pub(super) basis_members: BTreeMap<CommitId, MemoryBasisMembers>,
    pub(super) next_root_seq: u64,
}

pub(super) struct MemoryLoanCustody {
    pub(super) pins: Arc<VolatileRootPinTable>,
    pub(super) pin: RootPinId,
}

impl Drop for MemoryLoanCustody {
    fn drop(&mut self) {
        self.pins.release_loan(self.pin);
    }
}

pub(super) struct MemoryMemberReader {
    pub(super) pinned_members: MemoryBasisMembers,
    pub(super) corrupted_overrides: Arc<std::sync::RwLock<BTreeMap<ChunkMemberId, Vec<u8>>>>,
}

impl LoanedMemberReader for MemoryMemberReader {
    fn read_member(
        &self,
        id: ChunkMemberId,
        _locator_bytes: Option<&[u8; 64]>,
    ) -> Result<Vec<u8>, HistoricalReadDenial> {
        if let Some(bytes) = self
            .corrupted_overrides
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
        {
            return Ok(bytes.clone());
        }
        self.pinned_members
            .get(&id)
            .map(|bytes| bytes.to_vec())
            .ok_or_else(corrupt)
    }
}

pub(super) fn build_restarted_state(
    pins: &VolatileRootPinTable,
    state: &MemoryCatalogState,
) -> (
    Arc<VolatileRootPinTable>,
    Arc<AuxiliaryIndexRegistry>,
    MemoryCatalogState,
) {
    let new_pins = Arc::new(VolatileRootPinTable::default());
    let aux_indexes = Arc::new(AuxiliaryIndexRegistry::default());
    let mut new_catalog = HistoricalBasisCatalog::fresh(state.catalog.mac_key);
    let mut new_basis_manifests = BTreeMap::new();
    let mut new_basis_members = BTreeMap::new();
    let mut seq = 1u64;

    for (commit_id, entry_arc) in &state.catalog.bases {
        if pins.state(RootPinId::from_ordinal(entry_arc.pin_ordinal)) != Some(RootPinState::Active)
        {
            continue;
        }
        let pin = new_pins
            .pin(
                seq,
                None,
                RootPinDropPolicy::RetainUntilChildrenDrain,
                entry_arc.root_semantic_commitment.to_vec(),
            )
            .expect("fresh pin table");
        seq += 1;
        let mut cloned = (**entry_arc).clone();
        cloned.pin_ordinal = pin.ordinal();
        cloned.previous_pin_ordinal = None;
        if let Some(manifest) = state.basis_manifests.get(commit_id) {
            aux_indexes.build_for_basis(
                cloned.commit_id,
                cloned.root_semantic_commitment,
                manifest,
            );
            new_basis_manifests.insert(*commit_id, Arc::clone(manifest));
        }
        if let Some(members) = state.basis_members.get(commit_id) {
            new_basis_members.insert(*commit_id, Arc::clone(members));
        }
        new_catalog.bases.insert(*commit_id, Arc::new(cloned));
    }

    let retained_branches = RetainedBranchIndex::from_catalog(&new_catalog);
    (
        new_pins,
        aux_indexes,
        MemoryCatalogState {
            catalog: new_catalog,
            retained_branches,
            basis_manifests: new_basis_manifests,
            basis_members: new_basis_members,
            next_root_seq: seq,
        },
    )
}

pub(super) fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}

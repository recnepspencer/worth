//! Retain, consolidate, and prune mutations on [`MemoryHistoricalBasisStore`].

use std::collections::BTreeMap;
use std::sync::Arc;

#[cfg(feature = "certification-test-authority")]
use sha2::{Digest, Sha256};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    BranchOriginIdentity, HistoricalBasisRetentionKind, HistoricalBasisRow, HistoricalReadDenial,
    SchemaVersionIdentity,
};
use worth_store::pin_table::{RootPinDropPolicy, RootPinId, RootPinState};

use super::catalog::RetainedBasisRecord;
use super::chunk_format::{
    compute_root_semantic_commitment, seal_basis_chunks, ChunkMemberId, HistoricalChunkGeometry,
    SealedChunkDescriptor,
};
use super::chunk_scan::consolidate_chunk;
use super::durable_reconcile::check_duplicate_retain;
use super::memory::MemoryHistoricalBasisStore;
use super::memory_support::corrupt;
#[cfg(feature = "certification-test-authority")]
use super::stream::HistoricalCursorCertificationMutant;

impl MemoryHistoricalBasisStore {
    pub fn retain_basis(
        &self,
        commit_id: CommitId,
        branch_origin: BranchOriginIdentity,
        schema_version: SchemaVersionIdentity,
        retention_kind: HistoricalBasisRetentionKind,
        rows: Vec<HistoricalBasisRow>,
        geometry: HistoricalChunkGeometry,
    ) -> Result<([u8; 32], RootPinId), HistoricalReadDenial> {
        let sealed = seal_basis_chunks(rows, geometry)?;
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(existing) = state.catalog.bases.get(&commit_id) {
            check_duplicate_retain(
                existing,
                sealed.root_semantic_commitment,
                &branch_origin,
                &schema_version,
                retention_kind,
                None,
                None,
            )?;
            let existing_pin = RootPinId::from_ordinal(existing.pin_ordinal);
            if self.inner.pins.state(existing_pin) == Some(RootPinState::Active) {
                return Ok((existing.root_semantic_commitment, existing_pin));
            }
        }
        let seq = state.next_root_seq;
        state.next_root_seq = state.next_root_seq.saturating_add(1);
        let pin = self
            .inner
            .pins
            .pin(
                seq,
                None,
                RootPinDropPolicy::RetainUntilChildrenDrain,
                sealed.root_semantic_commitment.to_vec(),
            )
            .map_err(|_| corrupt())?;
        let members: BTreeMap<ChunkMemberId, Arc<[u8]>> = sealed
            .members
            .into_iter()
            .map(|(id, bytes)| (id, Arc::from(bytes)))
            .collect();
        let chunks_arc: Arc<[SealedChunkDescriptor]> = Arc::from(sealed.chunks);
        state
            .basis_manifests
            .insert(commit_id, Arc::clone(&chunks_arc));
        state.basis_members.insert(commit_id, Arc::new(members));
        if let Some(prev) = state.catalog.bases.insert(
            commit_id,
            Arc::new(RetainedBasisRecord {
                commit_id,
                root_semantic_commitment: sealed.root_semantic_commitment,
                branch_origin,
                schema_version,
                retention_kind,
                pin_ordinal: pin.ordinal(),
                previous_pin_ordinal: None,
                consolidation_epoch: 0,
                retained_branch_name: None,
                retained_snapshot_version: None,
                basis_manifest_member: None,
                basis_manifest_locator: None,
            }),
        ) {
            state
                .retained_branches
                .decrement(prev.retained_branch_name.as_deref());
        }
        self.inner.aux_indexes.build_for_basis(
            commit_id,
            sealed.root_semantic_commitment,
            &chunks_arc,
        );
        Ok((sealed.root_semantic_commitment, pin))
    }

    pub fn consolidate_basis(
        &self,
        commit_id: CommitId,
    ) -> Result<([u8; 32], RootPinId), HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let entry = state
            .catalog
            .bases
            .get(&commit_id)
            .cloned()
            .ok_or(HistoricalReadDenial::HistoryUnavailable { commit_id })?;
        let current_members = state
            .basis_members
            .get(&commit_id)
            .cloned()
            .unwrap_or_else(|| Arc::new(BTreeMap::new()));
        let current_chunks = state
            .basis_manifests
            .get(&commit_id)
            .cloned()
            .unwrap_or_else(|| Arc::from(Vec::new()));
        let mut new_chunks = Vec::with_capacity(current_chunks.len());
        let mut next_members: BTreeMap<ChunkMemberId, Arc<[u8]>> = BTreeMap::new();
        for chunk in current_chunks.iter() {
            let chunk_bytes = current_members
                .get(&chunk.chunk_member)
                .cloned()
                .ok_or_else(corrupt)?;
            let (updated, flat_bytes) = consolidate_chunk(chunk, &chunk_bytes, |cont_id| {
                current_members
                    .get(&cont_id)
                    .map(|bytes| bytes.to_vec())
                    .ok_or_else(corrupt)
            })?;
            next_members.insert(updated.chunk_member, Arc::from(flat_bytes));
            for cont in &updated.continuation_members {
                if let Some(bytes) = current_members.get(cont) {
                    next_members.insert(*cont, Arc::clone(bytes));
                }
            }
            new_chunks.push(updated);
        }
        let base_root = if new_chunks.is_empty() {
            entry.root_semantic_commitment
        } else {
            let commitments: Vec<[u8; 32]> =
                new_chunks.iter().map(|c| c.semantic_commitment).collect();
            let derived = compute_root_semantic_commitment(&commitments);
            if derived != entry.root_semantic_commitment {
                return Err(corrupt());
            }
            derived
        };
        #[cfg(feature = "certification-test-authority")]
        let derived_root = if self.active_mutant()
            == HistoricalCursorCertificationMutant::LocatorCoupledCommitment
        {
            let mut h = Sha256::new();
            h.update(base_root);
            for chunk in &new_chunks {
                h.update(chunk.chunk_member.sha256);
            }
            h.finalize().into()
        } else {
            base_root
        };
        #[cfg(not(feature = "certification-test-authority"))]
        let derived_root = base_root;

        let old_pin = RootPinId::from_ordinal(entry.pin_ordinal);
        let seq = state.next_root_seq;
        state.next_root_seq = state.next_root_seq.saturating_add(1);
        let new_pin = self
            .inner
            .pins
            .pin(
                seq,
                None,
                RootPinDropPolicy::RetainUntilChildrenDrain,
                derived_root.to_vec(),
            )
            .map_err(|_| corrupt())?;
        let new_chunks_arc: Arc<[SealedChunkDescriptor]> = Arc::from(new_chunks);
        state
            .basis_manifests
            .insert(commit_id, Arc::clone(&new_chunks_arc));
        state
            .basis_members
            .insert(commit_id, Arc::new(next_members));
        let mut updated = (*entry).clone();
        updated.root_semantic_commitment = derived_root;
        updated.pin_ordinal = new_pin.ordinal();
        updated.consolidation_epoch = updated.consolidation_epoch.saturating_add(1);
        state.catalog.bases.insert(commit_id, Arc::new(updated));
        self.inner.pins.expire(old_pin).map_err(|_| corrupt())?;
        if self.inner.aux_indexes.index_count() > 0 {
            self.inner
                .aux_indexes
                .build_for_basis(commit_id, derived_root, &new_chunks_arc);
        }
        Ok((derived_root, new_pin))
    }

    pub fn prune_basis(
        &self,
        commit_id: CommitId,
    ) -> Result<Option<RootPinState>, HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let Some(removed) = state.catalog.bases.remove(&commit_id) else {
            return Ok(None);
        };
        state
            .retained_branches
            .decrement(removed.retained_branch_name.as_deref());
        if let Some(branch_name) = &removed.retained_branch_name
            && !state.catalog.pending_retire_branches.contains(branch_name)
        {
            state
                .catalog
                .pending_retire_branches
                .push(branch_name.clone());
        }
        state.basis_manifests.remove(&commit_id);
        state.basis_members.remove(&commit_id);
        let pin = RootPinId::from_ordinal(removed.pin_ordinal);
        let pin_state = self.inner.pins.expire(pin).map_err(|_| corrupt())?;
        Ok(Some(pin_state))
    }
}

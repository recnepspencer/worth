//! Crash-recoverable two-phase retain, consolidation, and prune transitions on
//! [`DurableHistoricalBasisStore`] (Store Milestone 4, Phase M4.4).

use std::sync::Arc;

#[cfg(feature = "certification-test-authority")]
use sha2::{Digest, Sha256};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    BranchOriginIdentity, HistoricalBasisRetentionKind, HistoricalBasisRow, HistoricalReadDenial,
    SchemaVersionIdentity,
};
use worth_store::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinId, RootPinState};

use super::catalog::RetainedBasisRecord;
use super::chunk_format::{seal_basis_chunks, HistoricalChunkGeometry};
use super::durable::DurableHistoricalBasisStore;
use super::durable_io::{
    clear_basis_stage, consolidate_durable_chunks, corrupt, encode_pin_payload,
    load_basis_manifest_on_demand, pin_write, publish_basis_with_manifest, publish_catalog,
    DurableMemberReader,
};
#[cfg(feature = "certification-test-authority")]
use super::durable_reconcile::DurableHistoricalCrashCut;
use super::durable_reconcile::{check_duplicate_retain, ensure_reconciled_locked};
#[cfg(feature = "certification-test-authority")]
use super::stream::HistoricalCursorCertificationMutant;

impl DurableHistoricalBasisStore {
    /// Seals and durably retains a chunked historical basis under Store's `PhysicalRootPins`.
    pub fn retain_basis(
        &self,
        commit_id: CommitId,
        branch_origin: BranchOriginIdentity,
        schema_version: SchemaVersionIdentity,
        retention_kind: HistoricalBasisRetentionKind,
        rows: Vec<HistoricalBasisRow>,
        geometry: HistoricalChunkGeometry,
    ) -> Result<([u8; 32], RootPinId), HistoricalReadDenial> {
        self.retain_basis_internal(
            commit_id,
            branch_origin,
            schema_version,
            retention_kind,
            None,
            rows,
            geometry,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn retain_basis_internal(
        &self,
        commit_id: CommitId,
        branch_origin: BranchOriginIdentity,
        schema_version: SchemaVersionIdentity,
        retention_kind: HistoricalBasisRetentionKind,
        retained_branch_name: Option<String>,
        rows: Vec<HistoricalBasisRow>,
        geometry: HistoricalChunkGeometry,
    ) -> Result<([u8; 32], RootPinId), HistoricalReadDenial> {
        let mut sealed = seal_basis_chunks(rows, geometry)?;
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        let _ = ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)?;
        let pins = self.inner.serving.root_pins().map_err(|_| corrupt())?;

        if let Some(existing) = state.catalog.bases.get(&commit_id) {
            check_duplicate_retain(
                existing,
                sealed.root_semantic_commitment,
                &branch_origin,
                &schema_version,
                retention_kind,
                retained_branch_name.as_deref(),
                None,
            )?;
            let existing_pin = RootPinId::from_ordinal(existing.pin_ordinal);
            if pins.state(existing_pin) == Some(RootPinState::Active) {
                return Ok((existing.root_semantic_commitment, existing_pin));
            }
        }

        state.needs_reconcile = true;
        let (basis_manifest_member, basis_manifest_locator) = publish_basis_with_manifest(
            &self.inner.serving,
            &mut state,
            commit_id,
            0,
            &sealed.root_semantic_commitment,
            &mut sealed.chunks,
            &sealed.members,
        )?;
        let mut next_catalog = state.catalog.clone();
        next_catalog.bases.insert(
            commit_id,
            Arc::new(RetainedBasisRecord {
                commit_id,
                root_semantic_commitment: sealed.root_semantic_commitment,
                branch_origin,
                schema_version,
                retention_kind,
                pin_ordinal: 0,
                previous_pin_ordinal: None,
                consolidation_epoch: 0,
                retained_branch_name,
                retained_snapshot_version: None,
                basis_manifest_member,
                basis_manifest_locator,
            }),
        );

        publish_catalog(&self.inner.serving, &mut state, &next_catalog)?;
        #[cfg(feature = "certification-test-authority")]
        if self.take_crash_cut(DurableHistoricalCrashCut::AfterRetainStageBeforePin) {
            return Err(HistoricalReadDenial::RequestStopped);
        }

        let pin_id = pins
            .pin_current(
                None,
                RootPinDropPolicy::RetainUntilChildrenDrain,
                encode_pin_payload(commit_id, &sealed.root_semantic_commitment, 0),
                pin_write()?,
            )
            .map_err(|_| corrupt())?
            .pin();
        #[cfg(feature = "certification-test-authority")]
        if self.take_crash_cut(DurableHistoricalCrashCut::AfterRetainPinBeforeBind) {
            return Err(HistoricalReadDenial::RequestStopped);
        }

        if let Some(entry_arc) = next_catalog.bases.get_mut(&commit_id) {
            let mut updated = (**entry_arc).clone();
            updated.pin_ordinal = pin_id.ordinal();
            *entry_arc = Arc::new(updated);
        }
        let previous_branch = state
            .catalog
            .bases
            .get(&commit_id)
            .and_then(|e| e.retained_branch_name.clone());
        let new_branch = next_catalog
            .bases
            .get(&commit_id)
            .and_then(|e| e.retained_branch_name.clone());
        publish_catalog(&self.inner.serving, &mut state, &next_catalog)?;
        state.catalog = next_catalog;
        state
            .retained_branches
            .decrement(previous_branch.as_deref());
        state.retained_branches.increment(new_branch.as_deref());
        state.needs_reconcile = false;
        self.inner.aux_indexes.build_for_basis(
            commit_id,
            sealed.root_semantic_commitment,
            &sealed.chunks,
        );
        Ok((sealed.root_semantic_commitment, pin_id))
    }

    /// Flattens layered chunk history into brand-new physical locators while
    /// keeping `root_semantic_commitment` bit-identical across crash cuts.
    pub fn consolidate_basis(
        &self,
        commit_id: CommitId,
    ) -> Result<([u8; 32], RootPinId), HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)? {
            self.clear_cached_memberships();
        }
        let entry = state
            .catalog
            .bases
            .get(&commit_id)
            .cloned()
            .ok_or(HistoricalReadDenial::HistoryUnavailable { commit_id })?;
        let old_pin = RootPinId::from_ordinal(entry.pin_ordinal);
        let pins = self.inner.serving.root_pins().map_err(|_| corrupt())?;
        let pin_loan = pins.loan(old_pin).map_err(|err| match err {
            RootPinDenial::LeaseExpired | RootPinDenial::PinNotFound => {
                HistoricalReadDenial::HistoryUnavailable { commit_id }
            }
            _ => corrupt(),
        })?;
        let pinned_reader = pin_loan.records().map_err(|_| corrupt())?;
        let manifest_reader = DurableMemberReader::new(
            pin_loan.records().map_err(|_| corrupt())?,
            Arc::clone(&self.inner.corrupted_overrides),
        );
        let existing_chunks =
            load_basis_manifest_on_demand(&self.inner.residency, &manifest_reader, &entry)?;
        let (mut new_chunks, new_members) = if existing_chunks.is_empty() {
            (Vec::new(), std::collections::BTreeMap::new())
        } else {
            consolidate_durable_chunks(
                &self.inner.residency,
                pinned_reader,
                Arc::clone(&self.inner.corrupted_overrides),
                &existing_chunks,
                entry.root_semantic_commitment,
            )?
        };
        drop(manifest_reader);
        drop(pin_loan);

        #[cfg(feature = "certification-test-authority")]
        let derived_root = if self.active_mutant()
            == HistoricalCursorCertificationMutant::LocatorCoupledCommitment
        {
            let mut h = Sha256::new();
            h.update(entry.root_semantic_commitment);
            for chunk in &new_chunks {
                h.update(chunk.chunk_member.sha256);
            }
            h.finalize().into()
        } else {
            entry.root_semantic_commitment
        };
        #[cfg(not(feature = "certification-test-authority"))]
        let derived_root = entry.root_semantic_commitment;

        let next_epoch = entry.consolidation_epoch.saturating_add(1);
        state.needs_reconcile = true;
        let (basis_manifest_member, basis_manifest_locator) = publish_basis_with_manifest(
            &self.inner.serving,
            &mut state,
            commit_id,
            next_epoch,
            &derived_root,
            &mut new_chunks,
            &new_members,
        )?;

        let mut next_catalog = state.catalog.clone();
        if let Some(target_arc) = next_catalog.bases.get_mut(&commit_id) {
            let mut updated = (**target_arc).clone();
            updated.root_semantic_commitment = derived_root;
            updated.basis_manifest_member = basis_manifest_member;
            updated.basis_manifest_locator = basis_manifest_locator;
            updated.consolidation_epoch = next_epoch;
            updated.previous_pin_ordinal = Some(old_pin.ordinal());
            *target_arc = Arc::new(updated);
        }

        publish_catalog(&self.inner.serving, &mut state, &next_catalog)?;
        #[cfg(feature = "certification-test-authority")]
        if self.take_crash_cut(DurableHistoricalCrashCut::AfterConsolidateStageBeforeNewPin) {
            return Err(HistoricalReadDenial::RequestStopped);
        }

        let new_pin = pins
            .pin_current(
                None,
                RootPinDropPolicy::RetainUntilChildrenDrain,
                encode_pin_payload(commit_id, &derived_root, next_epoch),
                pin_write()?,
            )
            .map_err(|_| corrupt())?
            .pin();
        #[cfg(feature = "certification-test-authority")]
        if self.take_crash_cut(DurableHistoricalCrashCut::AfterConsolidateNewPinBeforeBind) {
            return Err(HistoricalReadDenial::RequestStopped);
        }

        if let Some(target_arc) = next_catalog.bases.get_mut(&commit_id) {
            let mut updated = (**target_arc).clone();
            updated.pin_ordinal = new_pin.ordinal();
            updated.previous_pin_ordinal = None;
            *target_arc = Arc::new(updated);
        }
        if !next_catalog
            .pending_retire_pins
            .contains(&old_pin.ordinal())
        {
            next_catalog.pending_retire_pins.push(old_pin.ordinal());
        }
        publish_catalog(&self.inner.serving, &mut state, &next_catalog)?;
        state.catalog = next_catalog;
        self.evict_cached_membership(commit_id, old_pin.ordinal());
        if self.inner.aux_indexes.index_count() > 0 {
            self.inner
                .aux_indexes
                .build_for_basis(commit_id, derived_root, &new_chunks);
        }
        #[cfg(feature = "certification-test-authority")]
        if self.take_crash_cut(DurableHistoricalCrashCut::AfterConsolidateBindBeforeOldExpire) {
            return Err(HistoricalReadDenial::RequestStopped);
        }

        let old_state = pins.expire(old_pin, pin_write()?).map_err(|_| corrupt())?;
        if old_state == RootPinState::Expired {
            state
                .catalog
                .pending_retire_pins
                .retain(|ord| *ord != old_pin.ordinal());
            let clean = state.catalog.clone();
            publish_catalog(&self.inner.serving, &mut state, &clean)?;
        }
        state.needs_reconcile = false;
        Ok((derived_root, new_pin))
    }

    /// Atomically prunes a retained basis under the shared Home publication fence.
    pub fn prune_basis(
        &self,
        commit_id: CommitId,
    ) -> Result<Option<RootPinState>, HistoricalReadDenial> {
        let mut state = self.inner.state.lock().unwrap_or_else(|e| e.into_inner());
        if ensure_reconciled_locked(&self.inner.serving, &mut state, &self.inner.aux_indexes)? {
            self.clear_cached_memberships();
        }
        let Some(removed) = state.catalog.bases.get(&commit_id).cloned() else {
            return Ok(None);
        };
        let mut next_catalog = state.catalog.clone();
        next_catalog.bases.remove(&commit_id);
        if !next_catalog
            .pending_retire_pins
            .contains(&removed.pin_ordinal)
        {
            next_catalog.pending_retire_pins.push(removed.pin_ordinal);
        }
        if let Some(branch_name) = &removed.retained_branch_name
            && !next_catalog.pending_retire_branches.contains(branch_name)
        {
            next_catalog
                .pending_retire_branches
                .push(branch_name.clone());
        }
        state.needs_reconcile = true;
        publish_catalog(&self.inner.serving, &mut state, &next_catalog)?;
        state.catalog = next_catalog;
        state
            .retained_branches
            .decrement(removed.retained_branch_name.as_deref());
        self.evict_cached_membership(commit_id, removed.pin_ordinal);
        if state.catalog.bases.is_empty() {
            clear_basis_stage(&self.inner.serving, &mut state)?;
        }
        #[cfg(feature = "certification-test-authority")]
        if self.take_crash_cut(DurableHistoricalCrashCut::AfterPruneCatalogBeforePinExpire) {
            return Err(HistoricalReadDenial::RequestStopped);
        }

        let pins = self.inner.serving.root_pins().map_err(|_| corrupt())?;
        let pin_state = pins
            .expire(RootPinId::from_ordinal(removed.pin_ordinal), pin_write()?)
            .map_err(|_| corrupt())?;
        if pin_state == RootPinState::Expired {
            state
                .catalog
                .pending_retire_pins
                .retain(|ord| *ord != removed.pin_ordinal);
            let clean = state.catalog.clone();
            publish_catalog(&self.inner.serving, &mut state, &clean)?;
        }
        state.needs_reconcile = false;
        Ok(Some(pin_state))
    }
}

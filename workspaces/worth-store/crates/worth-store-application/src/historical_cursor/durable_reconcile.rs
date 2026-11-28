//! Crash-cut reopen and in-process staged-mutation reconciliation for two-phase
//! pin + catalog transitions (`retain_basis`, `consolidate_basis`, `prune_basis`).
//!
//! Guarantees zero orphaned active pins, zero silently lost retained commits,
//! seamless in-process recovery after a failed C1/C2/R1/R2/P1 cut without
//! requiring a Home reopen, and strict duplicate-retain metadata equality.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::{
    BranchOriginIdentity, HistoricalBasisRetentionKind, HistoricalReadDenial, SchemaVersionIdentity,
};
use worth_store::physical_runtime::ServingPhysicalRuntime;
use worth_store::pin_table::{RootPinDropPolicy, RootPinId, RootPinState};

use super::aux_index::AuxiliaryIndexRegistry;
use super::catalog::{HistoricalBasisCatalog, RetainedBasisRecord};
use super::durable_io::{
    anchor_name, clear_basis_stage, corrupt, decode_pin_payload, encode_pin_payload, pin_write,
    publish_catalog, DecodedPinPayload, DurableCatalogState, MAX_CATALOG_BYTES,
};

/// Certification-only simulated crash cuts between durable steps of
/// `retain_basis`, `consolidate_basis`, and `prune_basis`.
#[cfg(feature = "certification-test-authority")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableHistoricalCrashCut {
    /// Crash after step R1 (staged catalog + members published), before pin acquisition.
    AfterRetainStageBeforePin,
    /// Crash after step R2 (Store pin acquired), before catalog binds `pin_ordinal`.
    AfterRetainPinBeforeBind,
    /// Crash after step C1 (consolidated chunks staged with `previous_pin_ordinal`), before new pin.
    AfterConsolidateStageBeforeNewPin,
    /// Crash after step C2 (replacement pin acquired), before catalog binds new pin.
    AfterConsolidateNewPinBeforeBind,
    /// Crash after step C3 (catalog binds new pin + records `pending_retire_pins`), before old pin expires.
    AfterConsolidateBindBeforeOldExpire,
    /// Crash after step P1 (basis removed from catalog + pin added to `pending_retire_pins`), before pin expires.
    AfterPruneCatalogBeforePinExpire,
}

/// If a prior in-process multi-step durable mutation failed mid-flight (e.g.
/// after C1/C2 or R1/R2), reloads the durable catalog anchor and reconciles
/// state in-process BEFORE executing the next mutation. Returns `Ok(true)` iff
/// reconciliation ran so callers can evict any stale cached read memberships.
pub(super) fn ensure_reconciled_locked(
    serving: &ServingPhysicalRuntime,
    state: &mut DurableCatalogState,
    _aux_indexes: &AuxiliaryIndexRegistry,
) -> Result<bool, HistoricalReadDenial> {
    if !state.needs_reconcile {
        return Ok(false);
    }
    let records = serving.anchored_records().map_err(|_| corrupt())?;
    if let Some(record) = records
        .read(anchor_name(), MAX_CATALOG_BYTES)
        .map_err(|_| corrupt())?
    {
        state.anchor_generation = record.version().generation();
        state.catalog = HistoricalBasisCatalog::decode(record.bytes())?;
    }
    reconcile_catalog(serving, state, false)?;
    state.needs_reconcile = false;
    Ok(true)
}

/// Reconciles the restored durable catalog against Store's already-restored
/// `PhysicalRootPins` before admitting any request on cold reopen.
pub(super) fn reconcile_on_open(
    serving: &ServingPhysicalRuntime,
    state: &mut DurableCatalogState,
) -> Result<(), HistoricalReadDenial> {
    reconcile_catalog(serving, state, true)
}

fn reconcile_catalog(
    serving: &ServingPhysicalRuntime,
    state: &mut DurableCatalogState,
    promote_crashed_inflight_intents: bool,
) -> Result<(), HistoricalReadDenial> {
    let pins = serving.root_pins().map_err(|_| corrupt())?;
    let mut dirty = false;
    let mut rolled_back_commits = Vec::new();

    // Index every live WHPIN1 pin owned by this historical cursor subsystem.
    let mut owned_live_pins: BTreeMap<RootPinId, DecodedPinPayload> = BTreeMap::new();
    for pin_id in pins.live_pin_ids() {
        if let Some(bytes) = pins.payload(pin_id)
            && let Some(decoded) = decode_pin_payload(&bytes)
        {
            owned_live_pins.insert(pin_id, decoded);
        }
    }

    // Step 1: Reconcile half-finished retain (pin_ordinal == 0) and consolidation
    // (previous_pin_ordinal.is_some()) entries.
    let commit_ids: Vec<CommitId> = state.catalog.bases.keys().copied().collect();
    for cid in commit_ids {
        let Some(entry_arc) = state.catalog.bases.get(&cid).cloned() else {
            continue;
        };
        let mut entry = (*entry_arc).clone();

        // Case A: Staged retain (R1 or R2 crash cut): `retain_basis` never
        // bound `pin_ordinal` or acknowledged the retain to the caller, so roll
        // back the uncommitted staged entry; Step 3 then expires any orphaned R2 pin.
        if entry.pin_ordinal == 0 {
            if let Some(branch_name) = entry.retained_branch_name.take()
                && !state.catalog.pending_retire_branches.contains(&branch_name)
            {
                state.catalog.pending_retire_branches.push(branch_name);
            }
            state.catalog.bases.remove(&cid);
            rolled_back_commits.push(cid);
            dirty = true;
            continue;
        }

        // Case B: Half-finished consolidation (C1 or C2 crash cut).
        if let Some(old_pin_ord) = entry.previous_pin_ordinal {
            let matching_new = find_matching_active_pin(
                &pins,
                &owned_live_pins,
                entry.commit_id,
                entry.root_semantic_commitment,
                entry.consolidation_epoch,
            );
            let new_pin = if let Some(pin_id) = matching_new {
                pin_id
            } else {
                // Crashed after C1 before C2: acquire the replacement pin now for
                // the already-published consolidated chunks.
                let receipt = pins
                    .pin_current(
                        None,
                        RootPinDropPolicy::RetainUntilChildrenDrain,
                        encode_pin_payload(
                            entry.commit_id,
                            &entry.root_semantic_commitment,
                            entry.consolidation_epoch,
                        ),
                        pin_write()?,
                    )
                    .map_err(|_| corrupt())?;
                owned_live_pins.insert(
                    receipt.pin(),
                    DecodedPinPayload {
                        commit_id: entry.commit_id,
                        root_semantic_commitment: entry.root_semantic_commitment,
                        consolidation_epoch: entry.consolidation_epoch,
                    },
                );
                receipt.pin()
            };
            entry.pin_ordinal = new_pin.ordinal();
            entry.previous_pin_ordinal = None;
            if old_pin_ord != 0
                && old_pin_ord != new_pin.ordinal()
                && !state.catalog.pending_retire_pins.contains(&old_pin_ord)
            {
                state.catalog.pending_retire_pins.push(old_pin_ord);
            }
            state.catalog.bases.insert(cid, Arc::new(entry));
            dirty = true;
        }
    }

    // Step 2: Drop any catalog entry whose bound pin is no longer Active.
    let before_len = state.catalog.bases.len();
    state.catalog.bases.retain(|_, entry| {
        pins.state(RootPinId::from_ordinal(entry.pin_ordinal)) == Some(RootPinState::Active)
    });
    if state.catalog.bases.len() != before_len {
        dirty = true;
    }

    // Step 3: Collect all bound active pin ordinals so any other live WHPIN1 pin
    // (pending_retire_pins or orphan from a crash cut) is expired deterministically.
    let bound_pins: BTreeSet<u64> = state
        .catalog
        .bases
        .values()
        .map(|entry| entry.pin_ordinal)
        .collect();
    let mut to_expire: BTreeSet<u64> = state.catalog.pending_retire_pins.iter().copied().collect();
    for pin_id in owned_live_pins.keys() {
        if !bound_pins.contains(&pin_id.ordinal()) {
            to_expire.insert(pin_id.ordinal());
        }
    }

    let mut remaining_pending = Vec::new();
    for ord in to_expire {
        if ord == 0 || bound_pins.contains(&ord) {
            continue;
        }
        let pin = RootPinId::from_ordinal(ord);
        match pins.state(pin) {
            Some(RootPinState::Active | RootPinState::Expiring) => {
                let next_state = pins.expire(pin, pin_write()?).map_err(|_| corrupt())?;
                if next_state != RootPinState::Expired {
                    remaining_pending.push(ord);
                }
                dirty = true;
            }
            Some(RootPinState::Expired) | None => {
                if state.catalog.pending_retire_pins.contains(&ord) {
                    dirty = true;
                }
            }
        }
    }
    if state.catalog.pending_retire_pins != remaining_pending {
        state.catalog.pending_retire_pins = remaining_pending;
        dirty = true;
    }

    // Step 4 (cold reopen only): Any in-flight retain intent surviving from a
    // crashed prior incarnation is abandoned; promote unclaimed branch names
    // into `pending_retire_branches` for deterministic retirement.
    state.retained_branches.rebuild(&state.catalog);
    if promote_crashed_inflight_intents && !state.catalog.inflight_retain_branches.is_empty() {
        let abandoned = std::mem::take(&mut state.catalog.inflight_retain_branches);
        for branch_name in abandoned {
            if !state.retained_branches.contains(&branch_name)
                && !state.catalog.pending_retire_branches.contains(&branch_name)
            {
                state.catalog.pending_retire_branches.push(branch_name);
            }
        }
        dirty = true;
    }

    state.hydrate_stage_generation(serving)?;
    if !rolled_back_commits.is_empty() || state.catalog.bases.is_empty() {
        let _ = clear_basis_stage(serving, state);
    }
    if dirty {
        let next = state.catalog.clone();
        publish_catalog(serving, state, &next)?;
    }
    Ok(())
}

fn find_matching_active_pin(
    pins: &worth_store::physical_runtime::stability::durable_pins::PhysicalRootPins<'_>,
    owned_live_pins: &BTreeMap<RootPinId, DecodedPinPayload>,
    commit_id: CommitId,
    commitment: [u8; 32],
    epoch: u64,
) -> Option<RootPinId> {
    owned_live_pins.iter().find_map(|(pin_id, payload)| {
        if payload.commit_id == commit_id
            && payload.root_semantic_commitment == commitment
            && payload.consolidation_epoch == epoch
            && pins.state(*pin_id) == Some(RootPinState::Active)
        {
            Some(*pin_id)
        } else {
            None
        }
    })
}

/// Checks whether a duplicate `retain_basis` matches an existing live entry
/// identically across commitment, branch origin, schema version, retention
/// kind, retained branch identity, AND retained snapshot version.
pub(super) fn check_duplicate_retain(
    existing: &RetainedBasisRecord,
    commitment: [u8; 32],
    branch_origin: &BranchOriginIdentity,
    schema_version: &SchemaVersionIdentity,
    retention_kind: HistoricalBasisRetentionKind,
    retained_branch_name: Option<&str>,
    retained_snapshot_version: Option<u64>,
) -> Result<(), HistoricalReadDenial> {
    if existing.root_semantic_commitment == commitment
        && existing.branch_origin == *branch_origin
        && existing.schema_version == *schema_version
        && existing.retention_kind == retention_kind
        && existing.retained_branch_name.as_deref() == retained_branch_name
        && existing.retained_snapshot_version == retained_snapshot_version
    {
        Ok(())
    } else if existing.schema_version != *schema_version
        || existing.retention_kind != retention_kind
        || existing.retained_branch_name.as_deref() != retained_branch_name
        || existing.retained_snapshot_version != retained_snapshot_version
    {
        Err(HistoricalReadDenial::UnsupportedHistoricalMeaning {
            schema_version: schema_version.clone(),
        })
    } else {
        Err(corrupt())
    }
}

mod exact_materialization;
pub(crate) use exact_materialization::materialize_exact_state;
mod historical_materialization;
pub(crate) use historical_materialization::{
    ensure_historical_state, materialize_historical_visibility,
};

use crate::config::data::VisibilityCachePolicy;
use crate::runtime::state::subsystems::services::{
    instrumentation::RuntimeInstrumentation, symbol_table::RuntimeSymbolTable,
};
use crate::runtime::state::subsystems::visibility::cache::{VisibilityCache, VisibilityResidency};
use crate::visibility::branch_scope::HistoricalBasisInputs;
use crate::visibility::snapshot_states::SnapshotRecordReadInputs;

#[derive(Clone, Copy)]
pub(crate) struct VisibilityCacheInputs<'a> {
    pub(crate) runtime_identity: u64,
    pub(crate) cache: &'a VisibilityCache,
    pub(crate) policy: &'a VisibilityCachePolicy,
    pub(crate) history: HistoricalBasisInputs<'a>,
    pub(crate) symbols: &'a RuntimeSymbolTable,
    pub(crate) records: SnapshotRecordReadInputs<'a>,
    pub(crate) instrumentation: &'a RuntimeInstrumentation,
}
impl VisibilityCacheInputs<'_> {
    pub(crate) fn protects(self, residency: &VisibilityResidency) -> bool {
        residency.branch_head_refs > 0
            || residency.replay_refs > 0
            || (self.policy.protect_active_snapshots && residency.active_snapshot_refs > 0)
    }
}

use crate::visibility::snapshot_states::{
    HistoricalVisibilityBasis, HistoricalVisibilityDenial, SnapshotState,
    VisibilitySnapshotStateKey,
};

pub(crate) fn historical_basis_for_retained_version<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    version_id: crate::identity::data::VersionId,
) -> Result<HistoricalVisibilityBasis, HistoricalVisibilityDenial> {
    let inputs = inputs.into();
    HistoricalVisibilityBasis::resolve(inputs.history, version_id)
}

pub(crate) fn cached_state<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
) -> Option<SnapshotState> {
    let inputs = inputs.into();
    inputs.cache.state(key)
}

pub(crate) fn cached_historical_state_for_version<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    version_id: crate::identity::data::VersionId,
) -> Option<SnapshotState> {
    let inputs = inputs.into();
    let basis = historical_basis_for_retained_version(inputs, version_id).ok()?;
    inputs.cache.state(&basis_key(&basis))
}

pub(crate) fn residency<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
) -> VisibilityResidency {
    let inputs = inputs.into();
    inputs.cache.residency(key)
}

pub(crate) fn residency_for_version<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    version_id: crate::identity::data::VersionId,
) -> VisibilityResidency {
    let inputs = inputs.into();
    historical_basis_for_retained_version(inputs, version_id)
        .ok()
        .map(|basis| inputs.cache.residency(&basis_key(&basis)))
        .unwrap_or_default()
}

pub(crate) fn insert_state<'a>(inputs: impl Into<VisibilityCacheInputs<'a>>, state: SnapshotState) {
    let inputs = inputs.into();
    inputs.cache.insert_state(state);
}

pub(crate) fn bump_active_snapshot_ref<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
    delta: i32,
) {
    let inputs = inputs.into();
    let was_active = residency(inputs, key).active_snapshot_refs > 0;
    bump_visibility_ref(inputs, key, |residency| {
        residency.active_snapshot_refs =
            residency.active_snapshot_refs.saturating_add_signed(delta);
    });
    let is_active = residency(inputs, key).active_snapshot_refs > 0;
    if delta > 0 && !was_active && is_active {
        inputs
            .instrumentation
            .count(|counters| counters.visibility_cache_snapshot_promotions += 1);
    }
}

pub(crate) fn bump_replay_ref<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    version_id: crate::identity::data::VersionId,
    delta: i32,
) {
    let inputs = inputs.into();
    let Ok(basis) = historical_basis_for_retained_version(inputs, version_id) else {
        return;
    };
    bump_visibility_ref(inputs, &basis_key(&basis), |residency| {
        residency.replay_refs = residency.replay_refs.saturating_add_signed(delta);
    });
    if delta > 0 {
        inputs
            .instrumentation
            .count(|counters| counters.visibility_cache_replay_promotions += delta as usize);
    }
}

pub(crate) fn bump_visibility_ref<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
    update: impl FnOnce(&mut VisibilityResidency),
) {
    let inputs = inputs.into();
    inputs.cache.update_residency(key, update);
    maybe_remove_unprotected_state(inputs, key);
}

pub(crate) fn protect_branch_head_state<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
) {
    let inputs = inputs.into();
    inputs.cache.track_branch_head_state(key);
    bump_visibility_ref(inputs, key, |residency| {
        residency.branch_head_refs += 1;
    });
}

pub(crate) fn is_protected<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
) -> bool {
    let inputs = inputs.into();
    let residency = residency(inputs, key);
    inputs.protects(&residency)
}

pub(crate) fn mark_recent_state<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
) {
    let inputs = inputs.into();
    mark_recent_state_key(inputs, key);
}

fn mark_recent_state_key(inputs: VisibilityCacheInputs<'_>, key: &VisibilitySnapshotStateKey) {
    if !inputs.policy.enabled || inputs.cache.recent_window() == 0 {
        return;
    }
    if !inputs.cache.mark_recent_resident(key) {
        return;
    }
    evict_cache_if_needed(inputs);
}

fn basis_key(basis: &HistoricalVisibilityBasis) -> VisibilitySnapshotStateKey {
    VisibilitySnapshotStateKey::historical(basis)
}

fn is_key_protected(inputs: VisibilityCacheInputs<'_>, key: &VisibilitySnapshotStateKey) -> bool {
    let residency = inputs.cache.residency(key);
    inputs.protects(&residency)
}

pub(crate) fn evict_cache_if_needed<'a>(inputs: impl Into<VisibilityCacheInputs<'a>>) {
    let inputs = inputs.into();
    let window = inputs.cache.recent_window();
    if !inputs.policy.enabled || window == 0 {
        return;
    }
    loop {
        if inputs.cache.resident_recent_count() <= window {
            break;
        }
        let scan_len = inputs.cache.recent_candidate_count();
        if scan_len == 0 {
            break;
        }
        let mut evicted = false;
        for _ in 0..scan_len {
            let Some(candidate) = inputs.cache.pop_oldest_recent_candidate() else {
                break;
            };
            let candidate_residency = inputs.cache.residency(&candidate);
            if inputs.protects(&candidate_residency) {
                inputs.cache.enqueue_recent_candidate(candidate);
                continue;
            }
            if !inputs
                .cache
                .evict_recent_resident_if_unprotected(&candidate)
            {
                continue;
            }
            inputs.cache.remove_state(&candidate);
            inputs
                .instrumentation
                .count(|counters| counters.visibility_cache_recent_evictions += 1);
            evicted = true;
            break;
        }
        if !evicted {
            break;
        }
    }
}

pub(crate) fn maybe_remove_unprotected_state<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
) {
    let inputs = inputs.into();
    let residency = inputs.cache.residency(key);
    if residency.branch_head_refs == 0
        && residency.replay_refs == 0
        && residency.active_snapshot_refs == 0
        && !residency.recent_resident
    {
        inputs.cache.remove_state(key);
    }
}

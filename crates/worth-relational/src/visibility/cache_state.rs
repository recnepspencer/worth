use crate::capabilities::VisibilityPolicySource;
use crate::runtime::{RelationalRuntime, VisibilityResidency};
use crate::snapshots::data::{SnapshotId, SnapshotReadPolicy};
use crate::visibility::snapshot_states::{
    HistoricalVisibilityBasis, HistoricalVisibilityDenial, SnapshotState, VisibilitySnapshotBasis,
    VisibilitySnapshotStateKey,
};

pub(crate) fn historical_basis_for_retained_version(
    runtime: &RelationalRuntime,
    version_id: crate::identity::data::VersionId,
) -> Result<HistoricalVisibilityBasis, HistoricalVisibilityDenial> {
    HistoricalVisibilityBasis::resolve(runtime, version_id)
}

pub(crate) fn cached_state(
    runtime: &RelationalRuntime,
    basis: &VisibilitySnapshotBasis,
) -> Option<SnapshotState> {
    runtime.visibility.cache.state(basis.key())
}

pub(crate) fn cached_historical_state_for_version(
    runtime: &RelationalRuntime,
    version_id: crate::identity::data::VersionId,
) -> Option<SnapshotState> {
    let basis = historical_basis_for_retained_version(runtime, version_id).ok()?;
    runtime.visibility.cache.state(&basis_key(&basis))
}

pub(crate) fn residency(
    runtime: &RelationalRuntime,
    basis: &VisibilitySnapshotBasis,
) -> VisibilityResidency {
    runtime.visibility.cache.residency(basis.key())
}

pub(crate) fn residency_for_version(
    runtime: &RelationalRuntime,
    version_id: crate::identity::data::VersionId,
) -> VisibilityResidency {
    historical_basis_for_retained_version(runtime, version_id)
        .ok()
        .map(|basis| runtime.visibility.cache.residency(&basis_key(&basis)))
        .unwrap_or_default()
}

pub(crate) fn insert_state(runtime: &RelationalRuntime, state: SnapshotState) {
    runtime.visibility.cache.insert_state(state);
}

pub(crate) fn bump_active_snapshot_ref(
    runtime: &RelationalRuntime,
    basis: &VisibilitySnapshotBasis,
    delta: i32,
) {
    let was_active = residency(runtime, basis).active_snapshot_refs > 0;
    bump_visibility_ref(runtime, basis.key(), |residency| {
        residency.active_snapshot_refs =
            residency.active_snapshot_refs.saturating_add_signed(delta);
    });
    let is_active = residency(runtime, basis).active_snapshot_refs > 0;
    if delta > 0 && !was_active && is_active {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_snapshot_promotions += 1);
    }
}

pub(crate) fn bump_replay_ref(
    runtime: &RelationalRuntime,
    version_id: crate::identity::data::VersionId,
    delta: i32,
) {
    let Ok(basis) = historical_basis_for_retained_version(runtime, version_id) else {
        return;
    };
    bump_visibility_ref(runtime, &basis_key(&basis), |residency| {
        residency.replay_refs = residency.replay_refs.saturating_add_signed(delta);
    });
    if delta > 0 {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_replay_promotions += delta as usize);
    }
}

pub(crate) fn bump_visibility_ref(
    runtime: &RelationalRuntime,
    key: &VisibilitySnapshotStateKey,
    update: impl FnOnce(&mut VisibilityResidency),
) {
    runtime.visibility.cache.update_residency(key, update);
    maybe_remove_unprotected_state(runtime, key);
}

pub(crate) fn protect_branch_head_state(
    runtime: &RelationalRuntime,
    basis: &VisibilitySnapshotBasis,
) {
    runtime.visibility.track_branch_head_state(basis.key());
    bump_visibility_ref(runtime, basis.key(), |residency| {
        residency.branch_head_refs += 1;
    });
}

pub(crate) fn ensure_state(
    runtime: &RelationalRuntime,
    basis: VisibilitySnapshotBasis,
    recent_candidate: bool,
) -> SnapshotState {
    if let Some(state) = cached_state(runtime, &basis) {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return state;
    }
    runtime
        .services
        .instrumentation
        .count(|counters| counters.visibility_exact_state_materializations += 1);
    let state = crate::visibility::snapshot_states::build_visibility_state(
        runtime,
        basis.clone(),
        SnapshotId(0),
        SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
    );
    insert_state(runtime, state.clone());
    if recent_candidate {
        mark_recent_state(runtime, &basis);
    }
    state
}

pub(crate) fn ensure_historical_state(
    runtime: &RelationalRuntime,
    basis: HistoricalVisibilityBasis,
    recent_candidate: bool,
) -> SnapshotState {
    let key = basis_key(&basis);
    if let Some(state) = runtime.visibility.cache.state(&key) {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return state;
    }
    runtime
        .services
        .instrumentation
        .count(|counters| counters.visibility_cache_miss_reconstructions += 1);
    let state = crate::visibility::snapshot_states::build_historical_visibility_state(
        runtime,
        basis,
        SnapshotId(0),
        SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
    );
    insert_state(runtime, state.clone());
    if recent_candidate {
        mark_recent_state_key(runtime, &key);
    }
    state
}

pub(crate) fn materialize_exact_state_for_basis(
    runtime: &RelationalRuntime,
    basis: VisibilitySnapshotBasis,
    allow_recent_admission: bool,
) -> Option<SnapshotState> {
    let version_id = basis.version_id();
    if version_id.as_u64() > runtime.current_version_id().as_u64() {
        return None;
    }
    if let Some(state) = cached_state(runtime, &basis) {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return Some(state);
    }
    let recent_candidate = allow_recent_admission
        && runtime.config.visibility.cache_policy.enabled
        && runtime.visibility.cache.recent_window() > 0
        && !is_protected(runtime, &basis);
    if recent_candidate || is_protected(runtime, &basis) {
        return Some(ensure_state(runtime, basis, recent_candidate));
    }
    runtime
        .services
        .instrumentation
        .count(|counters| counters.visibility_exact_state_materializations += 1);
    Some(crate::visibility::snapshot_states::build_visibility_state(
        runtime,
        basis,
        SnapshotId(0),
        SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
    ))
}

pub(crate) fn materialize_historical_visibility(
    runtime: &RelationalRuntime,
    version_id: crate::identity::data::VersionId,
    allow_recent_admission: bool,
) -> Option<SnapshotState> {
    let basis = historical_basis_for_retained_version(runtime, version_id).ok()?;
    let key = basis_key(&basis);
    if let Some(state) = runtime.visibility.cache.state(&key) {
        runtime
            .services
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return Some(state);
    }
    let recent_candidate = allow_recent_admission
        && runtime.config.visibility.cache_policy.enabled
        && runtime.visibility.cache.recent_window() > 0
        && !is_key_protected(runtime, &key);
    if recent_candidate || is_key_protected(runtime, &key) {
        return Some(ensure_historical_state(runtime, basis, recent_candidate));
    }
    runtime
        .services
        .instrumentation
        .count(|counters| counters.visibility_cache_miss_reconstructions += 1);
    Some(
        crate::visibility::snapshot_states::build_historical_visibility_state(
            runtime,
            basis,
            SnapshotId(0),
            SnapshotReadPolicy::ImmutablePinnedNoLazyMutation,
        ),
    )
}

pub(crate) fn is_protected(runtime: &RelationalRuntime, basis: &VisibilitySnapshotBasis) -> bool {
    let residency = residency(runtime, basis);
    residency.branch_head_refs > 0
        || residency.replay_refs > 0
        || (runtime.protect_active_snapshots() && residency.active_snapshot_refs > 0)
}

pub(crate) fn mark_recent_state(runtime: &RelationalRuntime, basis: &VisibilitySnapshotBasis) {
    mark_recent_state_key(runtime, basis.key());
}

fn mark_recent_state_key(runtime: &RelationalRuntime, key: &VisibilitySnapshotStateKey) {
    if !runtime.config.visibility.cache_policy.enabled
        || runtime.visibility.cache.recent_window() == 0
    {
        return;
    }
    if !runtime.visibility.cache.mark_recent_resident(key) {
        return;
    }
    evict_cache_if_needed(runtime);
}

fn basis_key(basis: &HistoricalVisibilityBasis) -> VisibilitySnapshotStateKey {
    VisibilitySnapshotStateKey::historical(basis)
}

fn is_key_protected(runtime: &RelationalRuntime, key: &VisibilitySnapshotStateKey) -> bool {
    let residency = runtime.visibility.cache.residency(key);
    residency.branch_head_refs > 0
        || residency.replay_refs > 0
        || (runtime.protect_active_snapshots() && residency.active_snapshot_refs > 0)
}

pub(crate) fn evict_cache_if_needed(runtime: &RelationalRuntime) {
    let window = runtime.visibility.cache.recent_window();
    if !runtime.config.visibility.cache_policy.enabled || window == 0 {
        return;
    }
    loop {
        if runtime.visibility.cache.resident_recent_count() <= window {
            break;
        }
        let scan_len = runtime.visibility.cache.recent_candidate_count();
        if scan_len == 0 {
            break;
        }
        let mut evicted = false;
        for _ in 0..scan_len {
            let Some(candidate) = runtime.visibility.cache.pop_oldest_recent_candidate() else {
                break;
            };
            let candidate_residency = runtime.visibility.cache.residency(&candidate);
            if candidate_residency.branch_head_refs > 0
                || candidate_residency.replay_refs > 0
                || (runtime.protect_active_snapshots()
                    && candidate_residency.active_snapshot_refs > 0)
            {
                runtime.visibility.cache.enqueue_recent_candidate(candidate);
                continue;
            }
            if !runtime
                .visibility
                .cache
                .evict_recent_resident_if_unprotected(&candidate)
            {
                continue;
            }
            runtime.visibility.cache.remove_state(&candidate);
            runtime
                .services
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

pub(crate) fn maybe_remove_unprotected_state(
    runtime: &RelationalRuntime,
    key: &VisibilitySnapshotStateKey,
) {
    let residency = runtime.visibility.cache.residency(key);
    if residency.branch_head_refs == 0
        && residency.replay_refs == 0
        && residency.active_snapshot_refs == 0
        && !residency.recent_resident
    {
        runtime.visibility.cache.remove_state(key);
    }
}

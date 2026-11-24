//! Admit exact cache entries by key while the selector supplies the retained state.
use super::{cached_state, insert_state, is_protected, mark_recent_state, VisibilityCacheInputs};
use crate::visibility::snapshot_states::{SnapshotState, VisibilitySnapshotStateKey};

fn ensure_state<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
    recent_candidate: bool,
    build: impl FnOnce() -> SnapshotState,
) -> SnapshotState {
    let inputs = inputs.into();
    if let Some(state) = cached_state(inputs, key) {
        inputs
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return state;
    }
    inputs
        .instrumentation
        .count(|counters| counters.visibility_exact_state_materializations += 1);
    let state = build();
    insert_state(inputs, state.clone());
    if recent_candidate {
        mark_recent_state(inputs, key);
    }
    state
}

pub(crate) fn materialize_exact_state<'a>(
    inputs: impl Into<VisibilityCacheInputs<'a>>,
    key: &VisibilitySnapshotStateKey,
    allow_recent_admission: bool,
    build: impl FnOnce() -> SnapshotState,
) -> Option<SnapshotState> {
    let inputs = inputs.into();
    if key.version_id().as_u64() > inputs.history.history.branch.current_version_id().as_u64() {
        return None;
    }
    if let Some(state) = cached_state(inputs, key) {
        inputs
            .instrumentation
            .count(|counters| counters.visibility_cache_hits += 1);
        return Some(state);
    }
    let recent_candidate = allow_recent_admission
        && inputs.policy.enabled
        && inputs.cache.recent_window() > 0
        && !is_protected(inputs, key);
    if recent_candidate || is_protected(inputs, key) {
        return Some(ensure_state(inputs, key, recent_candidate, build));
    }
    inputs
        .instrumentation
        .count(|counters| counters.visibility_exact_state_materializations += 1);
    Some(build())
}

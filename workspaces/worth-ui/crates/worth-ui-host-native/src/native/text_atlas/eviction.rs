//! Deterministic unpinned atlas eviction policy.

use std::collections::HashSet;

use worth_ui_host_contract::{UiGlyphRasterKey, UiGlyphRasterKeyEvidence};

use super::key::canonical_raster_key_bytes;
use super::ownership::AtlasStore;

pub(crate) fn evict_one(
    alpha: &mut AtlasStore,
    color: &mut AtlasStore,
    protected: &HashSet<UiGlyphRasterKey>,
) -> Option<UiGlyphRasterKey> {
    let alpha_candidate = first_candidate(alpha, protected);
    let color_candidate = first_candidate(color, protected);
    let key = match (alpha_candidate, color_candidate) {
        (Some(left), Some(right)) => {
            if (left.0, left.1) <= (right.0, right.1) {
                left.2
            } else {
                right.2
            }
        }
        (Some(left), None) => left.2,
        (None, Some(right)) => right.2,
        (None, None) => return None,
    };
    if alpha.remove(key).is_none() {
        color.remove(key);
    }
    Some(key)
}

fn first_candidate(
    store: &AtlasStore,
    protected: &HashSet<UiGlyphRasterKey>,
) -> Option<(u64, UiGlyphRasterKeyEvidence, UiGlyphRasterKey)> {
    // One pass, one encoding per entry: the oldest epoch wins and the key's
    // canonical evidence breaks ties. Distinct keys never share evidence.
    store
        .entries
        .values()
        .filter(|entry| !entry.pinned() && !protected.contains(&entry.key))
        .map(|entry| {
            (
                entry.completed_use_epoch,
                canonical_raster_key_bytes(entry.key),
                entry.key,
            )
        })
        .min_by(|left, right| (left.0, &left.1).cmp(&(right.0, &right.1)))
}

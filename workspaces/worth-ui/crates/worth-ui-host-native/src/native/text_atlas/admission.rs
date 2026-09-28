//! Pure demand validation, deterministic eviction, and pin admission.

use std::collections::HashMap;

use worth_ui_host_contract::UiGlyphRasterKey;

use super::capacity::{source_channels, MAX_EXTENT};
use super::entry::UiAtlasEntry;
use super::key::{canonical_raster_key_bytes, UiNativeValidatedRasterKey};
use super::ownership::AtlasStore;
use super::recovery::UiNativeTextAtlasDenial;
use super::transaction::UiNativeTextAtlasTransactionPlan;
use super::UiNativeTextAtlasDemand;

pub(crate) fn normalize_demands(
    demands: &[UiNativeTextAtlasDemand],
) -> Result<Vec<UiNativeTextAtlasDemand>, UiNativeTextAtlasDenial> {
    let mut by_key = HashMap::with_capacity(demands.len());
    for demand in demands.iter().copied() {
        UiNativeValidatedRasterKey::from_native_host(demand.key())?;
        if demand.width() == 0
            || demand.height() == 0
            || demand.width() > MAX_EXTENT
            || demand.height() > MAX_EXTENT
        {
            return Err(UiNativeTextAtlasDenial::GlyphExtentExceeded);
        }
        let expected_bytes = u64::from(demand.width())
            .checked_mul(u64::from(demand.height()))
            .and_then(|pixels| pixels.checked_mul(source_channels(demand.key().source())))
            .ok_or(UiNativeTextAtlasDenial::RasterGeometryMismatch)?;
        if demand.staged_bytes() != expected_bytes {
            return Err(UiNativeTextAtlasDenial::RasterGeometryMismatch);
        }
        if let Some(previous) = by_key.insert(demand.key(), demand) {
            if previous.identity() != demand.identity()
                || previous.width() != demand.width()
                || previous.height() != demand.height()
                || previous.staged_bytes() != demand.staged_bytes()
            {
                return Err(UiNativeTextAtlasDenial::RasterGeometryMismatch);
            }
            let representative = if demand_order_key(demand) < demand_order_key(previous) {
                demand
            } else {
                previous
            };
            by_key.insert(demand.key(), representative);
        }
    }
    if let Some(identity) = by_key.values().next().map(|demand| demand.identity()) {
        if by_key.values().any(|demand| demand.identity() != identity) {
            return Err(UiNativeTextAtlasDenial::RasterBatchMismatch);
        }
    }
    let mut normalized = by_key.into_values().collect::<Vec<_>>();
    normalized.sort_by_cached_key(|demand| canonical_raster_key_bytes(demand.key()));
    Ok(normalized)
}

fn demand_order_key(demand: UiNativeTextAtlasDemand) -> ([u8; 32], [u8; 32], u8) {
    (
        demand.source_identity().digest(),
        demand.source_layout().digest(),
        match demand.source_lane() {
            worth_ui_host_contract::UiGlyphRasterLane::Ordinary => 0,
            worth_ui_host_contract::UiGlyphRasterLane::Reconstruction => 1,
        },
    )
}

pub(crate) fn candidate_entry_mut<'entry>(
    alpha: &'entry mut AtlasStore,
    color: &'entry mut AtlasStore,
    key: UiGlyphRasterKey,
) -> Option<&'entry mut UiAtlasEntry> {
    alpha
        .entries
        .get_mut(&key)
        .or_else(|| color.entries.get_mut(&key))
}

pub(crate) fn next_entry_after_plan(plan: &UiNativeTextAtlasTransactionPlan) -> u64 {
    plan.next_entry
}

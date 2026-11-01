//! Concrete field-storage peaks; nested key/value heaps are accounted separately.
use super::{field_map::DecodedField, AspectValue, FieldKey};
use std::alloc::Layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructAspectValueStorageBounds {
    /// Maximum final pair backing; duplicate input keys can reduce it.
    pub retained_fields_bytes: usize,
    /// Additional backing above retained_fields_bytes at the decode peak.
    /// Their sum includes growth, rows/output overlap, and boxing overlap.
    pub transient_decode_bytes: usize,
}

pub(super) fn decode(count: usize) -> Option<StructAspectValueStorageBounds> {
    if count == 0 {
        return Some(StructAspectValueStorageBounds {
            retained_fields_bytes: 0,
            transient_decode_bytes: 0,
        });
    }
    let capacity = count.checked_next_power_of_two()?;
    let rows = Layout::array::<DecodedField>(capacity).ok()?.size();
    let old_rows = Layout::array::<DecodedField>(capacity / 2).ok()?.size();
    let fields = Layout::array::<(FieldKey, AspectValue)>(count).ok()?.size();
    let growth_peak = rows.checked_add(old_rows)?;
    // Even a shrinking conversion may keep its old output allocation while
    // obtaining the final box. No nested payload is cloned in that move.
    let boxing_peak = rows.checked_add(fields.checked_mul(2)?)?;
    let peak = growth_peak.max(boxing_peak);
    Some(StructAspectValueStorageBounds {
        retained_fields_bytes: fields,
        transient_decode_bytes: peak.checked_sub(fields)?,
    })
}

//! Checked tree storage and contiguous decode peaks; nested heaps are separate.
use super::{field_map::DecodedField, AspectValue, FieldKey};
use std::alloc::Layout;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructAspectValueStorageBounds {
    /// Maximum final tree/key/value Arc backing; duplicates can reduce it.
    pub retained_fields_bytes: usize,
    /// Additional backing above retained_fields_bytes at the decode peak.
    /// Their sum covers row growth, pair boxing and pair/tree coexistence.
    pub transient_decode_bytes: usize,
}

pub(super) fn sorted_construction(count: usize) -> Option<usize> {
    super::super::field_tree::StructFields::checked_allocation_bytes_per_field()?.checked_mul(count)
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
    let retained = sorted_construction(count)?;
    let growth_peak = rows.checked_add(old_rows)?;
    // canonical_fields retains rows while moving into a final pair Vec.
    // Boxing may overlap old/new pair backing. Tree construction follows
    // after rows have dropped and retains the pair iterator until completion.
    let boxing_peak = rows.checked_add(fields.checked_mul(2)?)?;
    let tree_peak = fields.checked_add(retained)?;
    let peak = growth_peak.max(boxing_peak).max(tree_peak);
    Some(StructAspectValueStorageBounds {
        retained_fields_bytes: retained,
        transient_decode_bytes: peak.checked_sub(retained)?,
    })
}

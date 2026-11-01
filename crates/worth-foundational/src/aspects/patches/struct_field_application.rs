//! Clear-then-set application over the actual canonical field streams.
use super::FieldLevelAspectPatch;
use crate::aspects::structs::{FieldKey, StructAspectValue};
use crate::values::AspectValue;
use std::alloc::Layout;

impl FieldLevelAspectPatch {
    /// Checked allocation bound for this struct-field merge only, in O(n+m+c).
    /// Includes cloned key/value capacity and output backing/boxing overlap.
    /// Excludes existing inputs, outer aspect maps, contracts and readmission.
    pub fn struct_field_merge_allocation_bound(
        &self,
        current: &StructAspectValue,
    ) -> Option<usize> {
        let (count, payload) = selected_fields(current, self).try_fold(
            (0usize, 0usize),
            |(count, bytes), (key, value)| {
                Some((
                    count.checked_add(1)?,
                    bytes
                        .checked_add(key.owned_allocation_capacity_bytes())?
                        .checked_add(value.owned_allocation_capacity_bytes())?,
                ))
            },
        )?;
        let fields = Layout::array::<(FieldKey, AspectValue)>(count).ok()?.size();
        // A positive pair allocation also makes a saturated nested measurement
        // fail checked addition rather than becoming an accepted smaller quote.
        fields.checked_mul(2)?.checked_add(payload)
    }
}

pub(super) fn apply(
    current: &StructAspectValue,
    patch: &FieldLevelAspectPatch,
) -> StructAspectValue {
    let count = selected_fields(current, patch)
        .try_fold(0usize, |count, _| count.checked_add(1))
        .expect("the actual canonical field roster fits the address space");
    let mut fields = Vec::with_capacity(count);
    fields.extend(selected_fields(current, patch).map(|(key, value)| (key.clone(), value.clone())));
    StructAspectValue::from_sorted_fields(fields.into_boxed_slice())
        .expect("merging canonical streams preserves strict field order")
}

pub(super) fn selected_fields<'a>(
    current: &'a StructAspectValue,
    patch: &'a FieldLevelAspectPatch,
) -> impl Iterator<Item = (&'a FieldKey, &'a AspectValue)> {
    let mut current = current.fields().peekable();
    let mut sets = patch.field_sets().peekable();
    let mut clears = patch.field_clears().peekable();
    std::iter::from_fn(move || loop {
        let use_set = match (current.peek(), sets.peek()) {
            (None, None) => return None,
            (None, Some(_)) => true,
            (Some(_), None) => false,
            (Some((left, _)), Some((right, _))) => {
                if left == right {
                    current.next();
                    true
                } else {
                    right < left
                }
            }
        };
        let (key, value) = if use_set {
            sets.next()?
        } else {
            current.next()?
        };
        while clears.peek().is_some_and(|clear| *clear < key) {
            clears.next();
        }
        let cleared = clears.peek().is_some_and(|clear| *clear == key);
        if use_set || !cleared {
            return Some((key, value));
        }
    })
}

#[cfg(test)]
mod tests;

//! Descriptive backing bounds for owned portable admission and canonical apply.
use super::{AuthoritativeRecordAspectPatch, FieldLevelAspectPatch};
use crate::aspects::{AspectContract, CanonicalFieldPath, ContractValidatedAspectValue, FieldKey};
use crate::values::AspectValue;
use std::alloc::Layout;

#[cfg(test)]
mod tests;

/// Checked upper bounds on this owner's concrete contiguous allocations.
/// Excludes nested contract, key and value heaps, existing portable input, and
/// arbitrary lazy iterator construction. Callers fund those separately before
/// invoking owned readmission or any owned contract lookup.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct AspectPatchStorageBounds {
    pub retained_bytes: usize,
    pub transient_bytes: usize,
}

impl AspectPatchStorageBounds {
    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            retained_bytes: self.retained_bytes.checked_add(other.retained_bytes)?,
            transient_bytes: self.transient_bytes.checked_add(other.transient_bytes)?,
        })
    }
}

pub(crate) fn backing_bytes<T>(count: usize) -> Option<usize> {
    Some(Layout::array::<T>(count).ok()?.size())
}

impl AuthoritativeRecordAspectPatch {
    /// Final outer rosters plus exact-capacity Vec/boxed-output overlap. Nested
    /// field-patch rosters are described by their own construction bound.
    pub fn construction_storage_bounds(
        sets: usize,
        clears: usize,
        fields: usize,
    ) -> Option<AspectPatchStorageBounds> {
        let bytes = backing_bytes::<ContractValidatedAspectValue>(sets)?
            .checked_add(backing_bytes::<AspectContract>(clears)?)?
            .checked_add(backing_bytes::<FieldLevelAspectPatch>(fields)?)?;
        Some(AspectPatchStorageBounds {
            retained_bytes: bytes,
            transient_bytes: bytes,
        })
    }
}

impl FieldLevelAspectPatch {
    /// Owned portable single-field mask construction only. Counts are actual
    /// input rosters; duplicate clears/mask fields may reduce retained storage.
    /// Includes borrowed duplicate ordinals, owned clear ordinals and the
    /// original/clone mask path backings. No hidden standard tree/sort storage.
    pub fn construction_storage_bounds(
        sets: usize,
        clears: usize,
        selected: usize,
    ) -> Option<AspectPatchStorageBounds> {
        let set_bytes = backing_bytes::<(FieldKey, AspectValue)>(sets)?;
        let clear_bytes = backing_bytes::<FieldKey>(clears)?;
        let mask_bytes = backing_bytes::<CanonicalFieldPath>(selected)?
            .checked_add(backing_bytes::<FieldKey>(selected)?)?;
        let retained = set_bytes
            .checked_add(clear_bytes)?
            .checked_add(mask_bytes)?;
        let transient = set_bytes
            .checked_add(clear_bytes)?
            .checked_add(mask_bytes)?
            .checked_add(backing_bytes::<(&FieldKey, usize)>(sets)?)?
            .checked_add(super::construction::clear_ordinal_storage_bytes(clears)?)?;
        Some(AspectPatchStorageBounds {
            retained_bytes: retained,
            transient_bytes: transient,
        })
    }
}

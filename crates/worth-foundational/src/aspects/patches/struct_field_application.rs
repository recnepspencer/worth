//! Retained output and copy-path storage for canonical immutable field updates.
use super::FieldLevelAspectPatch;
use crate::aspects::structs::StructAspectValue;

impl FieldLevelAspectPatch {
    /// Independent successor backing plus construction scratch. Shared retained
    /// values count toward the holder even though they are not physically cloned.
    pub fn struct_field_merge_allocation_bound(
        &self,
        current: &StructAspectValue,
    ) -> Option<usize> {
        let (retained, scratch) =
            current.patched_storage_bounds(&self.field_sets, &self.field_clears)?;
        retained.checked_add(scratch)
    }
}

#[cfg(test)]
mod tests;

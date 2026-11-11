//! Pre-lookup quote of actual owned portable operation/field rosters.
use super::{OperationKeyOrdinal, PortableAspectPatchOperation, PortableRecordAspectPatch};
use crate::aspects::patches::backing_bytes;
use crate::aspects::{
    AspectPatchStorageBounds, AuthoritativeRecordAspectPatch, FieldLevelAspectPatch,
};

impl PortableRecordAspectPatch {
    /// Backing-only bound for this existing owned readmission entry. Does not
    /// invoke contract lookup. The caller separately funds portable inputs,
    /// returned contract clones and all nested key/value heaps before calling
    /// readmission. Ordinary lazy public constructors are outside this bound.
    pub fn readmission_storage_bounds(&self) -> Option<AspectPatchStorageBounds> {
        let mut sets = 0usize;
        let mut clears = 0usize;
        let mut fields = 0usize;
        let mut nested = AspectPatchStorageBounds::default();
        for operation in self.operations() {
            match operation {
                PortableAspectPatchOperation::SetWhole { .. } => sets = sets.checked_add(1)?,
                PortableAspectPatchOperation::ClearWhole { .. } => {
                    clears = clears.checked_add(1)?
                }
                PortableAspectPatchOperation::PatchFields {
                    selected_fields,
                    field_sets,
                    field_clears,
                    ..
                } => {
                    fields = fields.checked_add(1)?;
                    nested =
                        nested.checked_add(FieldLevelAspectPatch::construction_storage_bounds(
                            field_sets.len(),
                            field_clears.len(),
                            selected_fields.len(),
                        )?)?;
                }
            }
        }
        let outer =
            AuthoritativeRecordAspectPatch::construction_storage_bounds(sets, clears, fields)?;
        // Disjoint single-operation boxes coexist with final output and the
        // preallocated admitted-owner Vec. Borrowed key ordinals die before
        // callbacks; including that roster still bounds the phase overlap.
        let transient = outer
            .transient_bytes
            .checked_add(outer.retained_bytes)?
            .checked_add(backing_bytes::<AuthoritativeRecordAspectPatch>(
                self.operations().len(),
            )?)?
            .checked_add(backing_bytes::<OperationKeyOrdinal<'_>>(
                self.operations().len(),
            )?)?;
        nested.checked_add(AspectPatchStorageBounds {
            retained_bytes: outer.retained_bytes,
            transient_bytes: transient,
        })
    }
}

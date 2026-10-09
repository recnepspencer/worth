use super::*;

fn boxed_bytes<T>(length: usize) -> Option<u64> {
    u64::try_from(length)
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

impl ImmutablePhysicalRedoPlan {
    /// Heap retained by the immutable plan; transient planning scratch is separate.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = boxed_bytes::<PhysicalRedoRecord>(self.records.len())?
            .checked_add(boxed_bytes::<PhysicalRedoDecision>(self.decisions.len())?)?
            .checked_add(boxed_bytes::<PhysicalRedoProjection>(
                self.projections.len(),
            )?)?
            .checked_add(boxed_bytes::<PhysicalRewriteRedo>(self.rewrites.len())?)?
            .checked_add(boxed_bytes::<PhysicalRewriteAdmission>(
                self.rewrite_admissions.len(),
            )?)?
            .checked_add(boxed_bytes::<PhysicalExtentCopyAdmission>(
                self.source_copies.len(),
            )?)?;
        bytes = self.records.iter().try_fold(bytes, |sum, record| {
            sum.checked_add(record.owned_heap_bytes()?)
        })?;
        bytes = self.projections.iter().try_fold(bytes, |sum, projection| {
            sum.checked_add(projection.materialization.owned_heap_bytes()?)
        })?;
        self.source_copies.iter().try_fold(bytes, |sum, copy| {
            sum.checked_add(copy.projection().owned_heap_bytes()?)
        })
    }
}

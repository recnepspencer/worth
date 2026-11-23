use super::*;

fn boxed_bytes<T>(length: usize) -> Option<u64> {
    u64::try_from(length)
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

impl AdmittedPhysicalRedoMembers {
    /// Heap retained by admitted redo members before page-observation planning.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = boxed_bytes::<AdmittedPhysicalRedoMember>(self.members.len())?
            .checked_add(boxed_bytes::<([u8; 32], u64)>(self.group_allocations.len())?)?
            .checked_add(boxed_bytes::<PhysicalRewriteRedo>(self.rewrites.len())?)?
            .checked_add(boxed_bytes::<PhysicalRewriteAdmission>(
                self.rewrite_admissions.len(),
            )?)?
            .checked_add(boxed_bytes::<PhysicalExtentCopyAdmission>(
                self.source_copies.len(),
            )?)?;
        for member in &self.members {
            bytes = bytes
                .checked_add(boxed_bytes::<PhysicalRedoRecord>(member.records.len())?)?
                .checked_add(member.projection.owned_heap_bytes()?)?
                .checked_add(boxed_bytes::<projection_admission::AdmittedInlineFrame>(
                    member.inline_frames.len(),
                )?)?;
            bytes = member.records.iter().try_fold(bytes, |sum, record| {
                sum.checked_add(record.owned_heap_bytes()?)
            })?;
            bytes = member.inline_frames.iter().try_fold(bytes, |sum, frame| {
                sum.checked_add(boxed_bytes::<(
                    worth_store_physical_format::DurableInlineRecordPlacement,
                    std::ops::Range<usize>,
                )>(frame.records.capacity())?)
            })?;
        }
        self.source_copies.iter().try_fold(bytes, |sum, copy| {
            sum.checked_add(copy.projection().owned_heap_bytes()?)
        })
    }
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

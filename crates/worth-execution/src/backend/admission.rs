use std::{mem::size_of, sync::Mutex};

use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

use crate::authority::ExecutionResourceLease;
use crate::report::ChargedBytes;

use super::meter::RunLimits;
use super::port::TaskOutcome;

#[cfg(test)]
mod destruction_tests;

/// Admission binds checked identities, values, and declared capacities. The
/// map pattern holds the separate checked access-family proof.
pub(crate) struct AdmittedBatch<T> {
    declaration: BatchDeclaration,
    values: Vec<T>,
}

/// Immutable identity and capacity truth shared by both input modes.
pub(crate) struct BatchDeclaration {
    identities: CanonicalUniqueVec<PartitionIdentity>,
    max_result_bytes: Vec<u64>,
    kernel_scratch_bytes: u64,
    declared_result_bytes: u64,
    access_memory_bytes: u64,
}

#[derive(Debug)]
pub enum BatchDenial {
    Identities,
    MemoryOverflow,
}

impl<T> AdmittedBatch<T> {
    pub(crate) fn retained_result_bytes<R>(&self) -> Option<u64> {
        let slots = self.len().checked_mul(size_of::<R>())?;
        u64::try_from(slots)
            .ok()?
            .checked_add(self.declaration.declared_result_bytes)
    }
    pub(crate) fn try_admit(
        entries: Vec<(PartitionIdentity, T, u64, u64)>,
        access_memory_bytes: u64,
    ) -> Result<Self, BatchDenial> {
        let identities = entries.iter().map(|item| item.0).collect();
        let identities = match CanonicalUniqueVec::try_from_sorted_unique(identities) {
            Ok(checked) => checked,
            Err(_) => return Err(BatchDenial::Identities),
        };
        let (values, capacities) = entries
            .into_iter()
            .map(|(_, value, scratch, result_bytes)| (value, (scratch, result_bytes)))
            .unzip();
        Self::admit_canonical(identities, values, capacities, access_memory_bytes)
            .ok_or(BatchDenial::MemoryOverflow)
    }

    /// Admits `values` in the order of the checked `identities`, one value
    /// and one `(kernel scratch, result)` capacity pair per identity; `None`
    /// when the declared bytes do not fit.
    pub(crate) fn admit_canonical(
        identities: CanonicalUniqueVec<PartitionIdentity>,
        values: Vec<T>,
        capacities: Vec<(u64, u64)>,
        access_memory_bytes: u64,
    ) -> Option<Self> {
        debug_assert_eq!(identities.as_slice().len(), values.len());
        debug_assert_eq!(values.len(), capacities.len());
        let (kernel_scratch_bytes, declared_result_bytes) =
            capacities
                .iter()
                .try_fold((0_u64, 0_u64), |(scratch, results), &(more, result)| {
                    Some((scratch.checked_add(more)?, results.checked_add(result)?))
                })?;
        let max_result_bytes = capacities.into_iter().map(|(_, result)| result).collect();
        Some(Self {
            declaration: BatchDeclaration {
                identities,
                max_result_bytes,
                kernel_scratch_bytes,
                declared_result_bytes,
                access_memory_bytes,
            },
            values,
        })
    }

    pub(crate) fn declaration(&self) -> &BatchDeclaration {
        &self.declaration
    }

    pub(crate) fn values(&self) -> &[T] {
        &self.values
    }

    pub(crate) fn into_parts(mut self) -> (BatchDeclaration, Vec<T>) {
        (
            std::mem::take(&mut self.declaration),
            std::mem::take(&mut self.values),
        )
    }

    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn identities(&self) -> &[PartitionIdentity] {
        self.declaration.identities()
    }
}

impl<T> Drop for AdmittedBatch<T> {
    fn drop(&mut self) {
        // Release identities, then the complete values Vec, then capacities.
        // A value's destructor must run while its declared capacities exist.
        drop(std::mem::replace(
            &mut self.declaration.identities,
            CanonicalUniqueVec::from_btree_set(Default::default()),
        ));
        drop(std::mem::take(&mut self.values));
    }
}

impl Default for BatchDeclaration {
    fn default() -> Self {
        Self {
            identities: CanonicalUniqueVec::from_btree_set(Default::default()),
            max_result_bytes: Vec::new(),
            kernel_scratch_bytes: 0,
            declared_result_bytes: 0,
            access_memory_bytes: 0,
        }
    }
}

impl BatchDeclaration {
    pub(crate) fn len(&self) -> usize {
        self.identities.as_slice().len()
    }

    pub(crate) fn identities(&self) -> &[PartitionIdentity] {
        self.identities.as_slice()
    }

    pub(crate) fn result_capacity(&self, index: usize) -> u64 {
        self.max_result_bytes[index]
    }
}

impl<T: ChargedBytes> AdmittedBatch<T> {
    /// Reserve all declared kernel/result bytes and framework-owned buffers
    /// before either a result slot or native scheduling order is allocated.
    pub(crate) fn execution_memory_bytes<R, E>(&self) -> Option<u64> {
        self.declaration
            .execution_memory_bytes::<T, R, E>(&self.values)
    }
}

impl BatchDeclaration {
    pub(super) fn execution_memory_bytes<T: ChargedBytes, R, E>(
        &self,
        values: &[T],
    ) -> Option<u64> {
        let input_heap = values.iter().try_fold(0_u64, |sum, item| {
            sum.checked_add(item.additional_charged_bytes())
        })?;
        execution_memory_requirement::<T, R, E>(
            self.len(),
            input_heap,
            self.kernel_scratch_bytes,
            self.declared_result_bytes,
            self.access_memory_bytes,
        )
    }
}

/// Read-only arithmetic for the same buffers admitted by `run_checked_batch`.
/// Calling this does not reserve memory or grant execution authority.
pub(crate) fn execution_memory_requirement<T, R, E>(
    count: usize,
    input_heap: u64,
    kernel_scratch_bytes: u64,
    declared_result_bytes: u64,
    access_memory_bytes: u64,
) -> Option<u64> {
    let count_bytes = |item_size: usize| -> Option<u64> {
        let bytes = count.checked_mul(item_size)?;
        u64::try_from(bytes).ok()
    };
    let fixed = [
        count_bytes(size_of::<T>())?,
        count_bytes(size_of::<PartitionIdentity>())?,
        count_bytes(size_of::<u64>())?,
        count_bytes(size_of::<Mutex<Option<TaskOutcome<R, E>>>>())?,
        count_bytes(size_of::<usize>())?,
        count_bytes(size_of::<R>())?,
        input_heap,
        kernel_scratch_bytes,
        declared_result_bytes,
        access_memory_bytes,
    ];
    fixed.into_iter().try_fold(0_u64, u64::checked_add)
}

/// The complete reservation asked of an actual leased checked batch,
/// including its inherited cancellation/checkpoint contexts.
pub(crate) fn execution_memory_requirement_for_lease<T, R, E>(
    lease: &ExecutionResourceLease<'_>,
    count: usize,
    input_heap: u64,
    kernel_scratch_bytes: u64,
    declared_result_bytes: u64,
    access_memory_bytes: u64,
) -> Option<u64> {
    let batch = execution_memory_requirement::<T, R, E>(
        count,
        input_heap,
        kernel_scratch_bytes,
        declared_result_bytes,
        access_memory_bytes,
    )?;
    let context = RunLimits::framework_context_bytes_for_lease(lease, count)?;
    batch.checked_add(context)
}

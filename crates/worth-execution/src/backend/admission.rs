use std::{mem::size_of, sync::Mutex};

use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

use crate::report::ChargedBytes;

use super::port::TaskOutcome;

/// Admission binds checked identities, values, and declared capacities. The
/// map pattern holds the separate checked access-family proof.
pub(crate) struct AdmittedBatch<T> {
    identities: CanonicalUniqueVec<PartitionIdentity>,
    values: Vec<T>,
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
            .checked_add(self.declared_result_bytes)
    }
    pub(crate) fn try_admit(
        entries: Vec<(PartitionIdentity, T, u64, u64)>,
        access_memory_bytes: u64,
    ) -> Result<Self, BatchDenial> {
        let sums = entries
            .iter()
            .try_fold((0_u64, 0_u64), |(scratch, results), item| {
                Some((scratch.checked_add(item.2)?, results.checked_add(item.3)?))
            });
        let Some((kernel_scratch_bytes, declared_result_bytes)) = sums else {
            return Err(BatchDenial::MemoryOverflow);
        };
        let identities = entries.iter().map(|item| item.0).collect();
        let identities = match CanonicalUniqueVec::try_from_sorted_unique(identities) {
            Ok(checked) => checked,
            Err(_) => return Err(BatchDenial::Identities),
        };
        let (values, max_result_bytes) = entries
            .into_iter()
            .map(|(_, value, _, result_bytes)| (value, result_bytes))
            .unzip();
        Ok(Self {
            identities,
            values,
            max_result_bytes,
            kernel_scratch_bytes,
            declared_result_bytes,
            access_memory_bytes,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn identities(&self) -> &[PartitionIdentity] {
        self.identities.as_slice()
    }

    pub(crate) fn value(&self, index: usize) -> &T {
        &self.values[index]
    }

    pub(crate) fn result_capacity(&self, index: usize) -> u64 {
        self.max_result_bytes[index]
    }
}

impl<T: ChargedBytes> AdmittedBatch<T> {
    /// Reserve all declared kernel/result bytes and framework-owned buffers
    /// before either a result slot or native scheduling order is allocated.
    pub(crate) fn execution_memory_bytes<R, E>(&self) -> Option<u64> {
        let count = self.len();
        let count_bytes = |item_size: usize| -> Option<u64> {
            let bytes = count.checked_mul(item_size)?;
            u64::try_from(bytes).ok()
        };
        let input_heap = self.values.iter().try_fold(0_u64, |sum, item| {
            sum.checked_add(item.additional_charged_bytes())
        })?;
        let fixed = [
            count_bytes(size_of::<T>())?,
            count_bytes(size_of::<PartitionIdentity>())?,
            count_bytes(size_of::<u64>())?,
            count_bytes(size_of::<Mutex<Option<TaskOutcome<R, E>>>>())?,
            count_bytes(size_of::<usize>())?,
            count_bytes(size_of::<R>())?,
            input_heap,
            self.kernel_scratch_bytes,
            self.declared_result_bytes,
            self.access_memory_bytes,
        ];
        fixed.into_iter().try_fold(0_u64, u64::checked_add)
    }
}

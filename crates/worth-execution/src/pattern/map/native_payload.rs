use std::collections::BTreeMap;

use super::{
    backend_for, ExecutionMap, MapKernelContext, MapKernelFailure, MapOutcome, NativePayload,
};
use crate::{
    backend::AdmittedBatch, ExecutionPhysicalAllocationDenialKind, ExecutionResourceLease,
};
use worth_foundational::PartitionIdentity;
use worth_proof::{CanonicalUniqueVec, DisjointKeySetFamily};

impl<T> ExecutionMap<T, ()> {
    /// Native keyless partitions. Payloads remain owned values; callers provide
    /// neither heap invoices nor result forecasts. Identity order is canonical.
    pub fn from_native_partitions(
        partitions: BTreeMap<PartitionIdentity, T>,
    ) -> Result<ExecutionMap<T, (), NativePayload>, ExecutionPhysicalAllocationDenialKind> {
        let mut identities = Vec::new();
        let mut values = Vec::new();
        identities
            .try_reserve_exact(partitions.len())
            .map_err(|_| ExecutionPhysicalAllocationDenialKind::Allocator)?;
        values
            .try_reserve_exact(partitions.len())
            .map_err(|_| ExecutionPhysicalAllocationDenialKind::Allocator)?;
        for (identity, value) in partitions {
            identities.push(identity);
            values.push(value);
        }
        let identities = CanonicalUniqueVec::try_from_sorted_unique(identities)
            .expect("BTreeMap identities are canonical");
        let write_sets = DisjointKeySetFamily::with_empty_sets(&identities);
        Ok(ExecutionMap {
            batch: AdmittedBatch::admit_native(identities, values),
            _read_sets: Vec::new(),
            _write_sets: write_sets,
            _payload: std::marker::PhantomData,
        })
    }
}

impl<T: Send, K> ExecutionMap<T, K, NativePayload> {
    /// Consume inputs through the ordinary native worker and canonical prefix
    /// lifecycle, without measuring or admitting application payload heaps.
    /// Framework storage, request lineage, cancellation and nested stops remain
    /// governed by the existing Execution authority.
    /// A declared payload contract cannot enter this mode:
    ///
    /// ```compile_fail,E0599
    /// use std::collections::BTreeMap;
    /// use worth_execution::{ExecutionMap, KeylessPartition, MapKernelFailure};
    /// use worth_foundational::PartitionIdentity;
    /// let bounded = ExecutionMap::<u64, ()>::from_keyless_partitions(BTreeMap::from([
    ///     (PartitionIdentity::new(1), KeylessPartition {
    ///         value: 1, kernel_scratch_bytes: 0, max_result_bytes: 1,
    ///     }),
    /// ])).unwrap();
    /// bounded.run_owned_native(None, |value, _| {
    ///     Ok::<_, MapKernelFailure<()>>(vec![value; 2])
    /// });
    /// ```
    pub fn run_owned_native<R: Send, E: Send, F>(
        self,
        lease: Option<&ExecutionResourceLease<'_>>,
        kernel: F,
    ) -> MapOutcome<R, E>
    where
        F: Fn(T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        crate::backend::run_owned_native_batch(lease, self.batch, backend_for(lease), &kernel)
            .into()
    }

    /// Controlled acquisition order on this map's same native lifecycle.
    pub fn run_owned_native_perturbed_for_test<R: Send, E: Send, F>(
        self,
        lease: Option<&ExecutionResourceLease<'_>>,
        seed: u64,
        kernel: F,
    ) -> (MapOutcome<R, E>, Vec<usize>)
    where
        F: Fn(T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        let (outcome, order) = crate::backend::run_owned_native_batch_perturbed_for_test(
            lease, self.batch, seed, &kernel,
        );
        (outcome.into(), order)
    }
}

#[cfg(test)]
mod retained_tests;
#[cfg(test)]
mod tests;

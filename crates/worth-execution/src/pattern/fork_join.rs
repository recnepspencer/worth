use worth_foundational::PartitionIdentity;

use crate::{authority::ExecutionResourceLease, oracle::CanonicalBits, report::ChargedBytes};

use super::{
    ExecutionMap, MapDenial, MapKernelContext, MapKernelFailure, MapOutcome, MapPartition,
    OracleMismatch,
};

/// A recursive child declares the same access proof as an independent map
/// partition. Child kernels may invoke another structured pattern.
pub struct ForkChild<T, K> {
    pub identity: PartitionIdentity,
    pub input: T,
    pub read_keys: Vec<K>,
    pub write_keys: Vec<K>,
    pub kernel_scratch_bytes: u64,
    pub max_result_bytes: u64,
}

pub type ForkJoinDenial = MapDenial;
pub type ForkJoinOutcome<R, E> = MapOutcome<R, E>;

/// Fork admission reuses map's checked identity and disjoint access family.
/// The shared proof prevents a recursive fork from opening a weaker lane.
pub struct ExecutionForkJoin<T, K> {
    admitted: ExecutionMap<T, K>,
}

impl<T, K: Ord + ChargedBytes> ExecutionForkJoin<T, K> {
    pub fn try_from_children(
        expected_identities: Vec<PartitionIdentity>,
        children: Vec<ForkChild<T, K>>,
    ) -> Result<Self, ForkJoinDenial> {
        let partitions = children
            .into_iter()
            .map(|child| MapPartition {
                identity: child.identity,
                value: child.input,
                read_keys: child.read_keys,
                write_keys: child.write_keys,
                kernel_scratch_bytes: child.kernel_scratch_bytes,
                max_result_bytes: child.max_result_bytes,
            })
            .collect();
        ExecutionMap::try_from_declared_partitions(expected_identities, partitions)
            .map(|admitted| Self { admitted })
    }
}

impl<T, K> ExecutionForkJoin<T, K> {
    pub fn child_count(&self) -> usize {
        self.admitted.partition_count()
    }
}

impl<T: ChargedBytes + Sync, K> ExecutionForkJoin<T, K> {
    pub fn run<R, E, F>(
        &self,
        lease: Option<&ExecutionResourceLease<'_>>,
        child: F,
    ) -> ForkJoinOutcome<R, E>
    where
        R: ChargedBytes + Send,
        E: ChargedBytes + Send,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.admitted.run(lease, child)
    }

    pub fn certify<R, E, F>(
        &self,
        lease: &ExecutionResourceLease<'_>,
        seed: u64,
        child: F,
    ) -> Result<ForkJoinOutcome<R, E>, OracleMismatch>
    where
        R: ChargedBytes + Send + CanonicalBits,
        E: ChargedBytes + Send + CanonicalBits,
        F: Fn(&T, &mut MapKernelContext<'_, '_>) -> Result<R, MapKernelFailure<E>> + Sync,
    {
        self.admitted.certify(lease, seed, child)
    }
}

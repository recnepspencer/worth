//! The sealed transport's storage declaration, composed from its owning types.
use super::*;
use worth_execution::{ChargedBytes, ReductionTree};
impl<S, F, C, O> WorthQueryPreparedPartitionedComputation<S, F, C, O>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    C: ApplicationManagedComputation<S, F>,
    O: WorthQueryPartitionedComputationOwner<S, F, C>,
{
    pub(in super::super) fn transport_bytes(&self) -> u64 {
        let Self {
            installed: _,
            prepared_work: _,
            run,
            deposit: _,
            kernel_observation_for_test,
        } = self;
        let observer_bytes = kernel_observation_for_test
            .as_ref()
            .map_or(0, |observer| observer.bytes());
        observer_bytes.saturating_add(match run {
            PreparedRun::Full {
                map,
                memory,
                remaining_work: _,
                recording,
                cause: _,
            } => map
                .additional_charged_bytes()
                .saturating_sub(memory.held_bytes())
                .saturating_add(
                    recording
                        .as_ref()
                        .map_or(0, |recording| recording.transport_bytes()),
                ),
            PreparedRun::Incremental(prepared) => prepared.transport_bytes(),
        })
    }
    pub(in super::super) fn result_capacity(&self) -> u64 {
        let count = match &self.run {
            PreparedRun::Full { map, .. } => map.partition_count(),
            PreparedRun::Incremental(prepared) => prepared.partition_count(),
        };
        // Use Execution's tree bound, including its construction scratch. The
        // computation declares each reduced value's heap capacity separately.
        type Tree<R> = ReductionTree<R, fn(&R, &R) -> R>;
        let declared = C::RESOURCES.maximum_retained_bytes() as u64;
        let tree = match &self.run {
            PreparedRun::Full { .. } => {
                Tree::<O::PartitionResult>::checked_build_memory_bound(count, declared)
            }
            PreparedRun::Incremental(_) => {
                Tree::<O::PartitionResult>::checked_serial_build_memory_bound(count, declared)
            }
        }
        .unwrap_or(u64::MAX);
        self.transport_bytes()
            .saturating_add(tree)
            .saturating_add(C::RESOURCES.maximum_retained_bytes() as u64)
    }
}
impl<S, F, C, O> WorthQueryCompletedPartitionedComputation<S, F, C, O>
where
    S: ApplicationSchema,
    F: ApplicationFeature<S>,
    C: ApplicationManagedComputation<S, F>,
    O: WorthQueryPartitionedComputationOwner<S, F, C>,
{
    pub(super) fn transport_bytes(&self) -> u64 {
        let Self {
            installed: _,
            reduced,
            charged_work: _,
            completed,
            deposit: _,
        } = self;
        reduced
            .additional_charged_bytes()
            .saturating_add(completed.transport_bytes(size_of::<
                ReductionTree<
                    O::PartitionResult,
                    fn(&O::PartitionResult, &O::PartitionResult) -> O::PartitionResult,
                >,
            >() as u64))
    }
}

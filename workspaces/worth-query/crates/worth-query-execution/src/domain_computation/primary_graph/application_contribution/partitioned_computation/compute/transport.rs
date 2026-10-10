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
    pub(super) fn transport_bytes(&self) -> u64 {
        let Self {
            installed: _,
            prepared_work: _,
            run,
            deposit: _,
        } = self;
        match run {
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
        }
    }
    pub(super) fn result_capacity(&self) -> u64 {
        let count = match &self.run {
            PreparedRun::Full { map, .. } => map.partition_count(),
            PreparedRun::Incremental(prepared) => prepared.partition_count(),
        };
        // Use Execution's tree bound, including its construction scratch. The
        // computation declares each reduced value's heap capacity separately.
        let tree = ReductionTree::<
            O::PartitionResult,
            fn(&O::PartitionResult, &O::PartitionResult) -> O::PartitionResult,
        >::checked_build_memory_bound(
            count, C::RESOURCES.maximum_retained_bytes() as u64
        )
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

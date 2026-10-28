//! One original-request handoff to the existing SDK component coordinator.
use super::{AggregateWork, BuildResult, PlanDenial};
use worth_execution::{
    ComponentEdit, ComponentPartitioner, ExecutionRequest, MapKernelFailure, PartitionItemId,
    PartitionUpdateDenial,
};
use worth_foundational::PartitionIdentity;

pub(super) struct CheckedComponents {
    graph: ComponentPartitioner,
}
impl CheckedComponents {
    pub(super) fn new() -> Self {
        Self {
            graph: ComponentPartitioner::new(),
        }
    }
    pub(super) fn route(&self, item: PartitionItemId) -> Option<PartitionIdentity> {
        self.graph.route(item)
    }
    pub(super) fn edit(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        edit: ComponentEdit,
        work: &mut AggregateWork<'_, '_, '_, '_>,
    ) -> BuildResult<()> {
        work.poll()?;
        self.graph
            .edit_with_request(execution, edit)
            .map_err(|denial| match denial {
                PartitionUpdateDenial::Stop(stop) => {
                    MapKernelFailure::Domain(PlanDenial::Execution(stop.into()))
                }
                PartitionUpdateDenial::Admission(denial) => MapKernelFailure::Domain(
                    PlanDenial::Execution(crate::execution::denial::lease_denial(denial)),
                ),
                PartitionUpdateDenial::Components(_) | PartitionUpdateDenial::Bisection(_) => {
                    MapKernelFailure::Domain(PlanDenial::InvalidComponentGraph)
                }
                PartitionUpdateDenial::KernelPanic => MapKernelFailure::Panic,
                PartitionUpdateDenial::ResultCapacityExceeded => {
                    MapKernelFailure::ResultCapacityExceeded
                }
            })?;
        // The SDK edit already charges its own real work; do not duplicate it.
        work.poll()
    }
}

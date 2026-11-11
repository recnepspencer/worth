//! What a transported row needs to name its committed selection read evidence.

/// The selected step's basis and the restoring commit's basis. They only name
/// the read evidence Native constructs from the actual committed selection;
/// they are never authority, and a mismatched pair yields no evidence.
pub struct HistoricalSelectionCoordinate {
    pub(in crate::domain_computation::primary_graph) selected:
        worth_relational::facade::mvcc::RelationalBranchObservation,
    pub(in crate::domain_computation::primary_graph) restored:
        worth_relational::facade::mvcc::RelationalBranchObservation,
    pub(super) prior: Option<std::sync::Arc<Self>>,
}

impl HistoricalSelectionCoordinate {
    pub(in crate::domain_computation::primary_graph) fn read_evidence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<
        worth_relational::facade::runtime::RelationalHistoricalSelectionRead,
        worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial,
    > {
        use worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial as Denial;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
        let count = execution
            .run_native_with_checkpoint(|context| {
                let mut count = 0usize;
                let mut next = Some(self);
                while let Some(coordinate) = next {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context, 3,
                    )?;
                    count = count
                        .checked_add(1)
                        .ok_or(worth_execution::MapKernelStop::WorkCounterOverflow)?;
                    next = coordinate.prior.as_deref();
                }
                Ok(count)
            })
            .map_err(Denial::Request)?
            .0
            .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        let bytes = (count as u64)
            .checked_mul(std::mem::size_of::<(&Self, &Self)>() as u64)
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let _memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            execution,
            execution.has_accounting(),
            bytes,
        )
        .map_err(Denial::Execution)?;
        let mut selections = Vec::with_capacity(count);
        let mut next = Some(self);
        while let Some(coordinate) = next {
            execution.consult().map_err(Denial::Request)?;
            selections.push((&coordinate.selected, &coordinate.restored));
            next = coordinate.prior.as_deref();
        }
        selections.reverse();
        runtime
            .read_truth()
            .read_historical_selection_chain(execution, &selections)
    }
}

//! Expected restored routes are selection facts, never current ownership facts.
use super::*;

impl RelationalHistoricalSelectionRead {
    /// Enumerate this exact selection's original output obligations. Absence
    /// dispositions are separate negative events. The currentness owner must
    /// still verify the resulting roster, presence, source facts and witnesses.
    pub fn selected_output_routes<'a>(
        &'a self,
        execution: &'a ExecutionRequest<'_, '_>,
    ) -> Result<
        impl Iterator<
                Item = Result<
                    &'a crate::mvcc::RequiredOutputPublicationMeaning,
                    RelationalHistoricalSelectionReadDenial,
                >,
            > + 'a,
        RelationalHistoricalSelectionReadDenial,
    > {
        let origins = self
            .latest()
            .envelope
            .historical_output_selection
            .as_ref()
            .map_or(&[][..], |selection| selection.publications.as_slice());
        // A complete-route read pays the known header count before traversal.
        // Per-item polling preserves cancellation even on worker-only requests.
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, origins.len() as u64)
            })
            .map_err(RelationalHistoricalSelectionReadDenial::Request)?
            .0
            .map_err(|stop| {
                RelationalHistoricalSelectionReadDenial::Request(
                    worth_execution::WorkCeilingDenial::Stopped(stop),
                )
            })?;
        Ok(origins.iter().map(move |origin| {
            execution
                .run_native_with_checkpoint(|context| {
                    crate::execution::request_stage::checkpoint(context, 0)
                })
                .map_err(RelationalHistoricalSelectionReadDenial::Request)?
                .0
                .map_err(|stop| {
                    RelationalHistoricalSelectionReadDenial::Request(
                        worth_execution::WorkCeilingDenial::Stopped(stop),
                    )
                })?;
            Ok(&origin.meaning)
        }))
    }
}

/// A derived index of the authenticated latest selection, never roster truth.
pub(super) struct SelectedOutputRouteIndex {
    positions: Vec<usize>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl SelectedOutputRouteIndex {
    pub(super) fn build(
        execution: &ExecutionRequest<'_, '_>,
        envelope: &CanonicalCommitEnvelope,
    ) -> Result<Self, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        let origins = envelope
            .historical_output_selection
            .as_ref()
            .map_or(&[][..], |selection| selection.publications.as_slice());
        let bytes = origins
            .len()
            .checked_mul(std::mem::size_of::<usize>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        // Sort allocation is fixed; charge the comparison upper bound before it.
        let levels = u64::from(usize::BITS - origins.len().leading_zeros()) + 1;
        let mut work = 0u64;
        for origin in origins {
            poll(execution, 1)?;
            work = work
                .checked_add(origin.meaning.family.len() as u64 + 64)
                .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        }
        poll(
            execution,
            work.checked_mul(levels)
                .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?,
        )?;
        let mut positions: Vec<_> = (0..origins.len()).collect();
        positions.sort_unstable_by(|a, b| {
            let a = &origins[*a].meaning;
            let b = &origins[*b].meaning;
            (a.source_scope, a.source_partition, a.family.as_str()).cmp(&(
                b.source_scope,
                b.source_partition,
                b.family.as_str(),
            ))
        });
        Ok(Self {
            positions,
            _memory: memory,
        })
    }
}
impl RelationalHistoricalSelectionRead {
    /// Resolve one exact admitted source partition. Ambiguous selected routes
    /// cannot establish currentness and deliberately return no route.
    pub fn selected_output_route(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: [u8; 32],
    ) -> Result<
        Option<&crate::mvcc::RequiredOutputPublicationMeaning>,
        RelationalHistoricalSelectionReadDenial,
    > {
        let Some(selection) = &self.latest().envelope.historical_output_selection else {
            poll(execution, 1)?;
            return Ok(None);
        };
        if self.routes.get().is_none() {
            let index = SelectedOutputRouteIndex::build(execution, &self.latest().envelope)?;
            let _ = self.routes.set(index);
        }
        let routes = self.routes.get().expect("a successful index is retained");
        let levels = u64::from(usize::BITS - routes.positions.len().leading_zeros()) + 1;
        poll(execution, levels.saturating_mul(family.len() as u64 + 64))?;
        let compare = |index: &usize| {
            let meaning = &selection.publications[*index].meaning;
            (
                meaning.source_scope,
                meaning.source_partition,
                meaning.family.as_str(),
            )
                .cmp(&(source, partition, family))
        };
        let start = routes
            .positions
            .partition_point(|index| compare(index).is_lt());
        let end = routes
            .positions
            .partition_point(|index| !compare(index).is_gt());
        Ok((end == start + 1).then(|| &selection.publications[routes.positions[start]].meaning))
    }
}
fn poll(
    execution: &ExecutionRequest<'_, '_>,
    work: u64,
) -> Result<(), RelationalHistoricalSelectionReadDenial> {
    use RelationalHistoricalSelectionReadDenial as Denial;
    execution
        .run_native_with_checkpoint(|context| {
            crate::execution::request_stage::checkpoint(context, work)
        })
        .map_err(Denial::Request)?
        .0
        .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))
}

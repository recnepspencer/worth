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

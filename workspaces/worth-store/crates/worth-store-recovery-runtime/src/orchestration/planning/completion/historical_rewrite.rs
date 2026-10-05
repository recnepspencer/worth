//! A later selected root does not erase an indeterminate sealed rewrite.
//! Authenticate its exact canonical result root and WAL-bound destination
//! route; ordinary displaced source media may already have been retired.

use worth_store_physical_format::{CurrentPhysicalRecordPlacement, PhysicalRewriteRedo};
use worth_store_recovery_physics::PhysicalRewriteAdmission;

use super::super::historical_publication::{self, HistoricalFailure};
use crate::orchestration::planning::{
    context::PlanningContext, resolved_basis::ResolvedPlanningBasis,
};

#[path = "historical_rewrite/inline_membership.rs"]
mod inline_membership;

enum SourceRoute {
    Extent(worth_store_physical_format::DurableExtentRecordPlacement),
    Inline(inline_membership::SourceInlineSpan),
}

pub(super) fn verify(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    admission: PhysicalRewriteAdmission,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let rewrite = admission.redo();
    let Some(record) = super::decode_record(rewrite.record_identity()) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let format = context.authority.record_format;
    if rewrite.source_root_generation().checked_add(1) != Some(rewrite.resulting_root_generation())
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (context, source) = historical_publication::observe(
        context,
        basis,
        rewrite.source_root_generation(),
        record,
        |discovery, root, route, budget, trace, scratch| match route {
            Some(CurrentPhysicalRecordPlacement::Extent(extent)) => {
                verify_source_extent(rewrite, extent)?;
                Ok(SourceRoute::Extent(extent))
            }
            Some(CurrentPhysicalRecordPlacement::Inline(inline)) => {
                let source = inline_membership::source(
                    discovery, root, budget, trace, scratch, format, rewrite, inline,
                )?;
                Ok(SourceRoute::Inline(source))
            }
            None => Err(HistoricalFailure::Invalid),
        },
    )?;
    historical_publication::observe(
        context,
        basis,
        rewrite.resulting_root_generation(),
        record,
        |discovery, root, route, budget, trace, scratch| {
            let route = route.ok_or(HistoricalFailure::Invalid)?;
            match (source, route) {
                (SourceRoute::Extent(prior), CurrentPhysicalRecordPlacement::Extent(extent)) => {
                    verify_extent(rewrite, prior, extent)
                }
                (SourceRoute::Inline(prior), CurrentPhysicalRecordPlacement::Inline(inline)) => {
                    inline_membership::verify(
                        discovery, root, budget, trace, scratch, format, rewrite, inline, &prior,
                    )
                }
                _ => Err(HistoricalFailure::Invalid),
            }
        },
    )
    .map(|(context, ())| context)
}

fn verify_source_extent(
    rewrite: PhysicalRewriteRedo,
    placement: worth_store_physical_format::DurableExtentRecordPlacement,
) -> Result<(), HistoricalFailure> {
    let arena = rewrite.extent_arena().ok_or(HistoricalFailure::Invalid)?;
    if placement.extent().get() != rewrite.source_placement()
        || placement.extent_generation() != rewrite.source_generation()
        || placement.arena_range() != arena.source()
        || placement.payload_bytes() != u64::from(rewrite.source_length())
    {
        return Err(HistoricalFailure::Invalid);
    }
    Ok(())
}

fn verify_extent(
    rewrite: PhysicalRewriteRedo,
    source: worth_store_physical_format::DurableExtentRecordPlacement,
    placement: worth_store_physical_format::DurableExtentRecordPlacement,
) -> Result<(), HistoricalFailure> {
    let arena = rewrite.extent_arena().ok_or(HistoricalFailure::Invalid)?;
    if rewrite.source_placement() != rewrite.destination_placement()
        || source.record() != placement.record()
        || source.extent() != placement.extent()
        || placement.extent().get() != rewrite.destination_placement()
        || rewrite.source_generation().checked_add(1) != Some(rewrite.destination_generation())
        || placement.extent_generation() != rewrite.destination_generation()
        || placement.arena_range() != arena.destination()
        || placement.payload_bytes() != u64::from(rewrite.source_length())
        || placement.payload_bytes() != u64::from(rewrite.destination_length())
    {
        return Err(HistoricalFailure::Invalid);
    }
    Ok(())
}

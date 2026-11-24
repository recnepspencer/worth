//! Metadata selection through the actual installed Native Source roster.
use super::RelationalAuthoredWriteDenial as Denial;
pub mod selected_catalog;

pub(super) fn require_interval(
    generation: &crate::history::access::relevant_writes::generation::AuthoredWriteGeneration,
    catalog: &std::sync::Arc<
        crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog,
    >,
    next: u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    super::visit(context)?;
    // Empty intervals require no historical packet. Nonempty intervals retain
    // exact immutable roster continuity and the greatest missing ordinal.
    if next <= generation.ordinal
        && (generation.managed_last_gap.is_some_and(|gap| gap >= next)
            || next < generation.managed_run_start
            || generation
                .managed_run_catalog
                .as_ref()
                .is_none_or(|actual| !std::sync::Arc::ptr_eq(actual, catalog)))
    {
        return Err(Denial::IncompleteManagedSourceHistory);
    }
    Ok(())
}

pub(super) fn require_packet(
    envelope: &std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>,
    catalog: &std::sync::Arc<
        crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog,
    >,
    keys: &[crate::history::access::relevant_writes::posting_key::AuthoredWritePostingKey],
    runtime: u64,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), Denial> {
    super::visit(context)?;
    let packet = crate::history::data::LiveManagedSourceHistory::verified(envelope)
        .ok_or(Denial::IncompleteManagedSourceHistory)?;
    if packet.runtime != runtime || !std::sync::Arc::ptr_eq(&packet.catalog, catalog) {
        return Err(Denial::IncompleteManagedSourceHistory);
    }
    for key in keys {
        super::visit(context)?;
        if let crate::history::access::relevant_writes::posting_key::AuthoredWritePostingKey::ManagedSource(coordinate) = key {
            super::ordered_visit(context, packet.impacts.len())?;
            if packet.impacts.contains_key(coordinate) {
                return Ok(());
            }
        }
    }
    Err(Denial::IncompleteManagedSourceHistory)
}

pub mod impact;
pub use impact::source_impact;
pub use impact::{
    RelationalAuthoredManagedSourceImpact, RelationalAuthoredManagedSourceMaterial,
    RelationalAuthoredManagedSourcePayloads,
};

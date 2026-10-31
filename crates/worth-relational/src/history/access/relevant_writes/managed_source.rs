//! Metadata selection through the actual installed Native Source roster.
use super::{RelationalAuthoredManagedSourceSelector, RelationalAuthoredWriteDenial as Denial};
use crate::identity::data::EntityId;
use crate::runtime::RelationalRuntime;
use crate::validation::candidate_source::ManagedIndexSourceProgram;
use worth_execution::ExecutionRequest;

impl RelationalRuntime {
    /// Issue a membership-only impact-candidate coordinate. Shared-row scopes
    /// can be conservatively included without an authored local edge. No principal, row,
    /// Query proof, mutation eligibility or Source READ is admitted here.
    pub fn authored_managed_source_selector(
        &self,
        execution: ExecutionRequest<'_, '_>,
        program: &ManagedIndexSourceProgram,
        scope: EntityId,
    ) -> Result<RelationalAuthoredManagedSourceSelector, Denial> {
        super::run(execution, |context| {
            let catalog = &self
                .schema_contract_runtime
                .custom_invariant_registries
                .source_history;
            super::ordered_visit(context, 0, catalog.len() as u64)?;
            catalog.selector(program, scope)
        })
    }
}

pub(super) fn require_interval(
    generation: &crate::runtime::AuthoredWriteGeneration,
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
    keys: &[crate::history::AuthoredWritePostingKey],
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
        if let crate::history::AuthoredWritePostingKey::ManagedSource(coordinate) = key {
            super::ordered_visit(
                context,
                packet.impacts.len(),
                size_of_val(coordinate) as u64,
            )?;
            if packet.impacts.contains_key(coordinate) {
                return Ok(());
            }
        }
    }
    Err(Denial::IncompleteManagedSourceHistory)
}

//! Freeze one actual catalog for both durable declarations and authored writes.
use crate::history::data::CanonicalIndexDefinitionDelta;
use crate::runtime::state::subsystems::indexing::CapturedIndexDefinitionCatalog;
use crate::runtime::state::subsystems::indexing::IndexDefinitionReadBinding;

pub(in crate::authority::commit::pipeline) struct PublicationIndexCatalogInputs<'a> {
    pub(in crate::authority::commit::pipeline) history: &'a crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory,
    pub(in crate::authority::commit::pipeline) checkpoint: &'a crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection,
    pub(in crate::authority::commit::pipeline) catalog: &'a IndexDefinitionReadBinding,
    pub(in crate::authority::commit::pipeline) home_catalog: Option<&'a CapturedIndexDefinitionCatalog>,
    pub(in crate::authority::commit::pipeline) original_catalog: Option<&'a CanonicalIndexDefinitionDelta>,
    pub(in crate::authority::commit::pipeline) work: Option<&'a crate::execution::RequestWorkBudget>,
}
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};

pub(super) fn prepare(
    inputs: PublicationIndexCatalogInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        CapturedIndexDefinitionCatalog,
        CanonicalIndexDefinitionDelta,
    ),
    TransactionCommitError,
> {
    // Accepted facts precede the live snapshot: a concurrent later registration
    // cannot make this baseline exceed the captured next issued identity.
    let base = inputs
        .history
        .index_definition_frontier()
        .max(inputs.checkpoint.acknowledged_index_definition_frontier());
    let mut captured = inputs
        .home_catalog
        .cloned()
        .unwrap_or_else(|| inputs.catalog.capture_catalog());
    if let Some(original) = inputs.original_catalog {
        if original.next_index_id > captured.next_index_id {
            return Err(invalid(
                "recovered original catalog prefix is not installed",
            ));
        }
        captured.next_index_id = original.next_index_id;
        captured.coverage = captured.catalog.coverage_before(original.next_index_id);
        return Ok((captured, original.clone()));
    }
    if base > captured.next_index_id {
        return Err(invalid(
            "accepted index catalog frontier exceeds the actual owner snapshot",
        ));
    }
    if base == captured.next_index_id {
        return Ok((
            captured,
            CanonicalIndexDefinitionDelta::prepared(base, base, Vec::new(), None),
        ));
    }
    let prepare = || prepare_delta(&captured, base, execution);
    let delta = match inputs.work {
        Some(budget) => budget.run_stage(*execution, prepare)??,
        None => prepare()?,
    };
    Ok((captured, delta))
}

fn prepare_delta(
    captured: &CapturedIndexDefinitionCatalog,
    base: u64,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<CanonicalIndexDefinitionDelta, TransactionCommitError> {
    let (prepared, _) = execution.run_native_with_checkpoint(|context| {
        let mut bytes = CanonicalIndexDefinitionDelta::root_bytes()
            .ok_or(MapKernelStop::WorkCounterOverflow)?;
        let mut count = 0usize;
        for definition in captured.introduced(base) {
            context.poll()?;
            bytes = bytes
                .checked_add(std::mem::size_of_val(definition.as_ref()) as u64)
                .and_then(|n| n.checked_add(definition.owned_allocation_capacity_bytes()))
                .ok_or(MapKernelStop::WorkCounterOverflow)?;
            count = count
                .checked_add(1)
                .ok_or(MapKernelStop::WorkCounterOverflow)?;
        }
        if context.has_accounting() {
            context.checkpoint(bytes)?;
        }
        let memory = execution.reserve_retained_memory(bytes);
        let mut definitions = Vec::new();
        let result = (|| {
            let memory = memory.map_err(TransactionCommitError::from)?;
            definitions.try_reserve_exact(count).map_err(|cause| {
                invalid(&format!("index catalog backing allocation failed: {cause}"))
            })?;
            for definition in captured.introduced(base) {
                context.poll().map_err(|cause| {
                    TransactionCommitError::from(WorkCeilingDenial::Stopped(cause))
                })?;
                definitions.push(definition.as_ref().clone());
            }
            Ok(CanonicalIndexDefinitionDelta::prepared(
                base,
                captured.next_index_id,
                definitions,
                memory,
            ))
        })();
        Ok::<_, MapKernelStop>(result)
    })?;
    prepared.map_err(|cause| TransactionCommitError::from(WorkCeilingDenial::Stopped(cause)))?
}

fn invalid(detail: &str) -> TransactionCommitError {
    TransactionCommitError::publication(crate::publication::data::PublicationError::new(
        crate::publication::bundle::PublicationStage::BundleAssembly,
        detail,
    ))
}

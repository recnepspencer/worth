//! Freeze one actual catalog for both durable declarations and authored writes.
use crate::history::data::CanonicalIndexDefinitionDelta;
use crate::runtime::{CapturedIndexDefinitionCatalog, RelationalPreparationRuntime};
use crate::transactions::data::TransactionCommitError;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};

pub(super) fn prepare(
    runtime: &RelationalPreparationRuntime,
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
    let base = runtime.history.index_definition_frontier().max(
        runtime
            .checkpoint_selection
            .snapshot()
            .map_or(1, |directory| directory.manifest.index_definition_frontier),
    );
    let mut captured = runtime.index_definitions.capture_catalog();
    if let Some(original) = &runtime.recovered_index_catalog {
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
    let delta = match &runtime.commit_work_budget {
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

//! A successful checkpoint transport preserves the actual selected generation.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::indexes::data::DerivedIndexGeneration;

pub(super) fn read(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    embedded: bool,
    expected: &DerivedIndexGeneration,
) -> Result<bool, Execution> {
    if embedded {
        let image = runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .map_err(|stop| match stop {
                crate::durability::data::RelationalNativeCheckpointCaptureDenial::Durability(
                    error,
                ) => Execution::StorageUnavailable(
                    error
                        .storage_unavailable
                        .expect("cold index export remains typed"),
                ),
                other => panic!("funded native transport: {other:?}"),
            })?;
        let mut receiving =
            runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
        receiving
            .durability_recovery()
            .restore_native_checkpoint(execution, &image)
            .unwrap();
        return Ok(receiving
            .index_access()
            .latest_generation(expected.index_id, &expected.source_branch_id)
            .as_deref()
            == Some(expected));
    }
    let image = runtime
        .durability_authority()
        .checkpoint(execution)
        .map_err(|stop| match stop {
            RelationalRecoveryStop::Durability(error) => Execution::StorageUnavailable(
                error
                    .storage_unavailable
                    .expect("cold memory index remains typed"),
            ),
            other => panic!("funded memory transport: {other:?}"),
        })?;
    Ok(image
        .derived_index_checkpoint
        .as_ref()
        .expect("captured checkpoint uses native generation encoding")
        .readmit()
        .unwrap()
        .iter()
        .any(|generation| generation == expected))
}

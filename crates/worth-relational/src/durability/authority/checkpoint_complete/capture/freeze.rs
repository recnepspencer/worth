//! Freeze the Complete producer roots under one configuration and publication epoch.
use super::super::current_metadata::FrozenCurrentCheckpointMetadata;
use super::{Denial, FrozenCompleteMetadata, FrozenOriginalInputs};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::symbols::data::string_interner::StringInternerOwnerPort as _;
use worth_relational_durability::internal_port::durability::data::checkpoint_member::RelationalCheckpointArtifactPolicyOwnerPort as _;

pub(in super::super) fn capture(
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
    placement: &crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
) -> Result<FrozenCompleteMetadata, Denial> {
    let binding = runtime.configuration_binding();
    let configuration = binding.operation();
    let routes = runtime.history.canonical_checkpoint_gate();
    let snapshot = routes
        .with_checkpoint_selection(|selection| {
            let original = FrozenOriginalInputs::capture(
                &runtime.history,
                || {
                    runtime
                        .history
                        .capture_original_branch_schema_inputs(request)
                },
                runtime.durability.checkpoint_selection(),
                runtime.durability.cold_source(),
                &runtime.services.symbols,
                selection.selected_receipt(),
                selection.covered_transitions(),
                request,
            )?;
            // Home selection retains immutable members by reference. Reject
            // an Embedded demand before protecting/reading any source body.
            if original.base.is_some() && placement.is_embedded() {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let FrozenCurrentCheckpointMetadata {
                branches,
                components,
                identity,
                main_cell,
            } = FrozenCurrentCheckpointMetadata::capture(
                &runtime.history,
                &runtime.record_identity,
                request,
                &configuration.config.execution.runtime_name,
                &configuration.config.history.main_branch,
            )?;
            let indexes = runtime.indexes.checkpoint_inputs_for_request(request)?;
            let symbol_frontier = runtime
                .services
                .symbols
                .with_read(|symbols| symbols.identity_frontier());
            Ok::<_, Denial>(FrozenCompleteMetadata {
                original,
                branches,
                components,
                identity,
                indexes,
                symbol_frontier,
                main_cell,
            })
        })
        .map_err(super::super::super::checkpointing::checkpoint_admission_error)??;
    drop(configuration);
    // No protected provider callback, journal read or reconstruction was
    // performed while either of the original logical epochs was held.
    Ok(snapshot)
}

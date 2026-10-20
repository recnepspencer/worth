//! Restore tracked output lineage at the checkpoint boundary, before any demand.
use super::super::{
    application_checkpoint::decode_checkpoint_computation,
    application_query::WorthQueryCheckpointSourceIdentity, output_lineage::RecordedSourceIdentity,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn restore<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let Some(authority) = application
        .product_runtime
        .recovered_root_authority
        .as_ref()
    else {
        return Ok(());
    };
    execution.consult().map_err(|cause| {
        super::super::application_checkpoint::CheckpointDecodeDenial::execution(cause)
            .into_installation_denial()
    })?;
    let observation = authority.product_branch();
    let mut lineage = application
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // This is one pass over bounded, installed-producer-readmitted checkpoint records.
    // Current-output reads stay indexed and check these facts at their selected basis.
    for recovered in application.recovered_outputs.iter() {
        let checkpoint = &recovered.checkpoint;
        let (Some(binding), Some(bytes)) = (
            recovered.correspondence.binding_type(),
            checkpoint.producer_facts.as_deref(),
        ) else {
            // An output without producer facts retains descriptive prior identity only.
            continue;
        };
        let (facts, computation_source) = decode_checkpoint_computation(
            bytes,
            checkpoint.producer_fact_wire_version,
            None,
            execution.allocation_policy(),
        )
        .map_err(|cause| {
            super::super::application_checkpoint::CheckpointDecodeDenial::from_fact(cause)
                .into_installation_denial()
        })?
        .into_parts();
        lineage.record_restoration(
            binding,
            application.runtime.authority_identity().as_u64(),
            application.installed_schema.binding_identity(),
            checkpoint.scope,
            observation,
            std::sync::Arc::clone(&recovered.correspondence),
            RecordedSourceIdentity::Checkpoint(WorthQueryCheckpointSourceIdentity::new(
                checkpoint.source,
            )),
            checkpoint.source_partition,
            checkpoint.producer_dependency,
            checkpoint.idempotency_key,
            computation_source.retain_facts(facts),
            // The row is recorded, not verified: its first reader or demand
            // compares its facts and output in full.
            None,
        );
    }
    Ok(())
}

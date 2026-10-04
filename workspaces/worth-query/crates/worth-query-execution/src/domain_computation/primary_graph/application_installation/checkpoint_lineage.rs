//! Restore tracked output lineage at the checkpoint boundary, before any demand.
use super::super::{
    application_checkpoint::decode_producer_facts,
    application_query::WorthQueryCheckpointSourceIdentity, output_lineage::RecordedSourceIdentity,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn restore<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let Some(authority) = application
        .product_runtime
        .recovered_root_authority
        .as_ref()
    else {
        return Ok(());
    };
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
            // Older outputs without producer facts retain descriptive prior identity only.
            continue;
        };
        let facts = decode_producer_facts(bytes).map_err(|detail| {
            WorthQueryPrimaryGraphInstallationDenial::new(
                WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                detail,
            )
        })?;
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
            facts,
            checkpoint.resources,
        );
    }
    Ok(())
}

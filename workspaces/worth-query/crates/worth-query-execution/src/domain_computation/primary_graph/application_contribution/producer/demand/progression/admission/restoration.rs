use worth_query_installation::facade::ApplicationSchema;

use super::{
    denial, WorthQueryObservedSource, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(super) fn readmit_checkpoint_output<Query>(
        &self,
        producer: &str,
        observed_source: &WorthQueryObservedSource<Query>,
        source_epoch: crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    ) -> Result<
        Option<
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput,
        >,
        WorthQueryOutputDemandDenial,
    >{
        let checkpoint_source = source_epoch.checkpoint_identity();
        let Some(readmitted) = self
            .recovered_outputs
            .matching_source_partition(source_scope, observed_source.partition_identity())
            .find(|readmitted| {
                readmitted.checkpoint.producer == producer
                    && readmitted.checkpoint.source == checkpoint_source.bytes()
            })
        else {
            return Ok(None);
        };
        let authority = self
            .product_runtime
            .recovered_root_authority
            .as_ref()
            .ok_or_else(|| {
                denial(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "checkpoint output has no World recovery adoption authority",
                )
            })?;
        let observation = authority.product_branch();
        if observed_source.selected_product_occurrence()
            != Some(observation.lifecycle_incarnation())
        {
            return Ok(None);
        }
        let observed_source_facts = observed_source
            .retained_checkpoint_facts(&self.primary_provider.graph.layout)
            .map_err(|error| {
                denial(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    format!("checkpoint output source facts were rejected: {error}"),
                )
            })?;
        Ok(Some(
            crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput {
                checkpoint: readmitted.checkpoint.clone(),
                correspondence: std::sync::Arc::clone(&readmitted.correspondence),
                observation: observation.clone(),
                source_scope,
                source_identity: source_epoch.checkpoint_identity(),
                observed_source_facts,
            },
        ))
    }

    pub(super) fn record_restored_output(
        &self,
        source_scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        restored: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryRestoredAcceptedOutput,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let output_binding = restored.correspondence.binding_type().ok_or_else(|| {
            denial(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                "checkpoint output has no installed output binding",
            )
        })?;
        self.primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .record_restoration(
                output_binding,
                self.runtime.authority_identity().as_u64(),
                self.installed_schema.binding_identity(),
                source_scope,
                &restored.observation,
                std::sync::Arc::clone(&restored.correspondence),
                crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(
                    restored.source_identity,
                ),
                restored.checkpoint.source_partition,
                restored.checkpoint.producer_dependency,
                restored.checkpoint.idempotency_key,
                std::sync::Arc::clone(&restored.observed_source_facts),
                restored.checkpoint.resources,
            );
        Ok(())
    }
}

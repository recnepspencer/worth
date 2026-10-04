//! Register exact checkpoint candidates without asserting current output truth.

use super::*;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn record_recovered_output_candidates<Family>(
        &self,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        observation: &crate::basis::WorthQueryProductBranchReadIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        output_bindings: &[std::any::TypeId],
        lineage: &mut crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        for recovered in self
            .recovered_outputs
            .matching_source_partition(scope, source.partition_identity())
        {
            let checkpoint = &recovered.checkpoint;
            let Some(binding) = recovered.correspondence.binding_type() else {
                continue;
            };
            if !output_bindings.contains(&binding) {
                continue;
            }
            if let (Some(authority), Some(fact_bytes)) = (
                self.product_runtime.recovered_root_authority.as_ref(),
                checkpoint.producer_facts.as_deref(),
            ) {
                let recovered_observation = authority.product_branch();
                if checkpoint.source == source.checkpoint_identity().bytes()
                    && recovered_observation.lifecycle_incarnation()
                        == observation.lifecycle_incarnation()
                {
                    let facts = crate::domain_computation::primary_graph::application_checkpoint::decode_producer_facts_for_wire_version(
                        fact_bytes,
                        checkpoint.producer_fact_wire_version,
                    )
                        .map_err(|error| WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                            error,
                        ))?;
                    lineage.record_restoration(
                        binding,
                        self.runtime.authority_identity().as_u64(),
                        self.installed_schema.binding_identity(),
                        scope,
                        recovered_observation,
                        std::sync::Arc::clone(&recovered.correspondence),
                        crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(
                            crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(checkpoint.source),
                        ),
                        checkpoint.source_partition,
                        checkpoint.producer_dependency,
                        checkpoint.idempotency_key,
                        facts,
                        checkpoint.resources,
                        None,
                    );
                    continue;
                }
            }
            lineage.record_recovered_prior_output(
                binding,
                self.runtime.authority_identity().as_u64(),
                self.installed_schema.binding_identity(),
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                std::sync::Arc::clone(&recovered.correspondence),
                crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(
                    crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(checkpoint.source),
                ),
                checkpoint.source_partition,
                checkpoint.producer_dependency,
                checkpoint.idempotency_key,
                checkpoint.resources,
            );
        }
        Ok(())
    }
}

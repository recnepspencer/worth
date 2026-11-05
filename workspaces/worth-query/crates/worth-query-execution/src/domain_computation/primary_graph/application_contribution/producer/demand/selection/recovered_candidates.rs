//! Register exact checkpoint candidates without asserting current output truth.

use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn record_recovered_output_candidates(
        &self,
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        partition: [u8; 32],
        checkpoint_source: &[u8; 32],
        observation: &crate::basis::WorthQueryProductBranchReadIdentity,
        scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        output_bindings: &[TypeId],
        lineage: &mut crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage,
    ) -> Result<(), worth_relational::facade::mvcc::CompanionPreflightStop> {
        for recovered in self
            .runtime
            .recovered_outputs
            .matching_source_partition(scope, partition)
        {
            if let Some(execution) = execution {
                execution
                    .run_native_with_checkpoint(|context| {
                        context.checkpoint((output_bindings.len() as u64).saturating_add(1))
                    })
                    .map_err(|cause| {
                        worth_relational::facade::mvcc::CompanionPreflightStop::Execution(
                            cause.into(),
                        )
                    })?
                    .0
                    .map_err(|cause| {
                        worth_relational::facade::mvcc::CompanionPreflightStop::Execution(
                            cause.into(),
                        )
                    })?;
            }
            let checkpoint = &recovered.checkpoint;
            let Some(binding) = recovered.correspondence.binding_type() else {
                continue;
            };
            if !output_bindings.contains(&binding) {
                continue;
            }
            if let (Some(authority), Some(_)) = (
                self.runtime
                    .product_runtime
                    .recovered_root_authority
                    .as_ref(),
                checkpoint.producer_facts.as_deref(),
            ) {
                let recovered_observation = authority.product_branch();
                if &checkpoint.source == checkpoint_source
                    && recovered_observation.lifecycle_incarnation()
                        == observation.lifecycle_incarnation()
                {
                    // Installation recorded this fact-bearing row.
                    continue;
                }
            }
            lineage.record_prior_output(
            binding,
            self.runtime.authority_identity().as_u64(),
            self.installed_schema.binding_identity(),
            scope,
            observation.lifecycle_incarnation(),
            observation.reference_generation().get(),
            std::sync::Arc::clone(&recovered.correspondence),
            crate::domain_computation::primary_graph::output_lineage::PriorOutputOrigin::Recovered { source_identity: crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(
                    crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(checkpoint.source),
                ), checkpoint: Some(checkpoint.clone()) },
            checkpoint.source_partition,
            checkpoint.producer_dependency,
            checkpoint.idempotency_key,
        );
        }
        Ok(())
    }
}

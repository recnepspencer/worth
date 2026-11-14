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
    let checkpoint_observation = application
        .runtime
        .product_runtime
        .recovered_root_authority
        .as_ref()
        .map(|authority| authority.product_branch());
    execution.consult().map_err(|cause| {
        super::super::application_checkpoint::CheckpointDecodeDenial::execution(cause)
            .into_installation_denial()
    })?;
    let mut lineage = application
        .runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // This is one pass over bounded, installed-producer-readmitted checkpoint records.
    // Current-output reads stay indexed and check these facts at their selected basis.
    for recovered in application.runtime.recovered_outputs.iter() {
        let Some(observation) = recovered.observation.as_ref().or(checkpoint_observation) else {
            continue;
        };
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
            Some(checkpoint.clone()),
            None,
        );
    }
    Ok(())
}

pub(in crate::domain_computation::primary_graph) fn readmit_journal<
    Schema: ApplicationSchema + 'static,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    originals: &worth_runtime_world::facade::WorldProgramActivationHistory,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial>
{
    use crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial as Denial;
    let Some(support) = application.installed_program_support() else {
        return Ok(());
    };
    let branches = application
        .runtime
        .product_runtime
        .owner
        .observation_port()
        .observe_installed_product_branches(execution)
        .map_err(|_| Denial::ObligationSubjectMismatch)?;
    let mut accepted = Vec::new();
    let mut records = support.lifecycle().accepted_outputs();
    for (commit, branch, payload) in support.lifecycle().output_checkpoint_prerequisites() {
        let checkpoint =
            super::super::application_checkpoint::accepted_output::decode(&payload, execution)
                .map_err(|_| Denial::MalformedFrame)?;
        records.push(super::super::query_obligations::QueryAcceptedOutputRecord {
            commit,
            branch,
            producer: checkpoint.producer.as_ref().clone(),
            scope: (
                checkpoint.scope.partition_id(),
                checkpoint.scope.local_slot(),
                checkpoint.scope.generation(),
            ),
            partition: checkpoint.source_partition,
            payload,
        });
    }
    for row in records {
        let original = originals
            .recorded_entry(row.commit)
            .ok_or(Denial::ObligationSubjectMismatch)?;
        if original.branch().name() != row.branch {
            return Err(Denial::ObligationSubjectMismatch);
        }
        let checkpoint =
            super::super::application_checkpoint::accepted_output::decode(&row.payload, execution)
                .map_err(|_| Denial::MalformedFrame)?;
        if checkpoint.producer.as_str() != row.producer
            || checkpoint.source_partition != row.partition
            || (
                checkpoint.scope.partition_id(),
                checkpoint.scope.local_slot(),
                checkpoint.scope.generation(),
            ) != row.scope
        {
            return Err(Denial::ObligationSubjectMismatch);
        }
        let correspondence = application
            .installed_producers
            .readmit_checkpoint_output(&application.installed_schema, &checkpoint, execution)
            .map_err(|_| Denial::ObligationSubjectMismatch)?;
        accepted.push(
            super::super::application_output_demand::WorthQueryReadmittedAcceptedOutput {
                observation: Some(
                    branches
                        .branches()
                        .iter()
                        .find(|known| {
                            known.branch_identity().name().as_str() == row.branch
                                && known.owner_identity().durable() == Some(row.commit.owner())
                        })
                        .cloned()
                        .ok_or(Denial::ObligationSubjectMismatch)?,
                ),
                checkpoint,
                correspondence,
            },
        );
    }
    application
        .runtime
        .recovered_outputs
        .readmit_journal(accepted, execution)
        .map_err(|_| Denial::MalformedFrame)?;
    restore(application, execution).map_err(|_| Denial::ObligationSubjectMismatch)
}

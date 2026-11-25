//! Restore tracked output lineage at the checkpoint boundary, before any demand.
use super::super::{
    application_checkpoint::decode_checkpoint_computation,
    application_query::WorthQueryCheckpointSourceIdentity, output_lineage::RecordedSourceIdentity,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrimaryGraphInstallationDenial,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::ApplicationOutputLineageInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductRuntimeInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn restore<Schema: ApplicationSchema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let checkpoint_observation = application
        .inner
        .runtime
        .product_runtime_ref()
        .recovered_root_authority_ref()
        .as_ref()
        .map(|authority| authority.product_branch());
    execution.consult().map_err(|cause| {
        super::super::application_checkpoint::CheckpointDecodeDenial::execution(cause)
            .into_installation_denial()
    })?;
    let mut lineage = application
        .inner
        .runtime
        .primary_provider_ref()
        .graph_ref()
        .output_lineage_ref()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // This is one pass over bounded, installed-producer-readmitted checkpoint records.
    // Current-output reads stay indexed and check these facts at their selected basis.
    for recovered in application.inner.runtime.recovered_outputs_ref().iter() {
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
            application.inner.runtime.authority_identity().as_u64(),
            application.inner.installed_schema.binding_identity(),
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
    let world = application
        .inner
        .runtime
        .product_runtime_ref()
        .owner_ref()
        .observation_port();
    let branches = world
        .installed_product_branches(execution)
        .map_err(|cause| match cause {
            worth_runtime_world::facade::RuntimeWorldServiceDenial::Denied(
                worth_runtime_world::facade::ProductBranchInventoryDenial::Request(cause),
            ) => Denial::Execution(cause),
            _ => match execution.consult() {
                Err(cause) => Denial::Execution(cause),
                Ok(()) => Denial::ObligationSubjectMismatch,
            },
        })?;
    let mut head_observations: std::collections::BTreeMap<
        worth_runtime_world::facade::ProductBranchIncarnation,
        worth_runtime_world::facade::ProductBranchObservation,
    > = std::collections::BTreeMap::new();
    let mut observe_head = |incarnation: worth_runtime_world::facade::ProductBranchIncarnation| -> Result<
        worth_runtime_world::facade::ProductBranchObservation,
        Denial,
    > {
        if let Some(existing) = head_observations.get(&incarnation) {
            return Ok(existing.clone());
        }
        let observation = world
            .observe_product_branch_occurrence(incarnation)
            .map_err(|_| match execution.consult() {
                Err(cause) => Denial::Execution(cause),
                Ok(()) => Denial::ObligationSubjectMismatch,
            })?;
        head_observations.insert(incarnation, observation.clone());
        Ok(observation)
    };
    for (acquire_position, basis, branch) in support.obligation_history().active_output_demands() {
        let (incarnation, retired) = originals
            .branch_status_for_commit(basis, &branch, execution)
            .map_err(Denial::Execution)?
            .ok_or(Denial::ObligationSubjectMismatch)?;
        if retired {
            support
                .lifecycle()
                .discharge_output_demand(execution, acquire_position)?;
        } else {
            support
                .obligation_history()
                .record_output_demand_lifecycle(acquire_position, incarnation.ordinal());
        }
    }
    for row in support.obligation_history().output_sources() {
        let retired = originals
            .branch_status_for_commit_incarnation(row.commit, &row.branch, row.lifecycle, execution)
            .map_err(Denial::Execution)?
            .ok_or(Denial::ObligationSubjectMismatch)?;
        if retired {
            support
                .lifecycle()
                .discharge_output_source(execution, row.commit)?;
        }
    }
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
    let mut decoded = Vec::new();
    for row in records {
        let (incarnation, retired) = originals
            .branch_status_for_commit(row.commit, &row.branch, execution)
            .map_err(Denial::Execution)?
            .ok_or(Denial::ObligationSubjectMismatch)?;
        if retired {
            continue;
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
            .inner
            .installed_producers
            .readmit_checkpoint_output(&application.inner.installed_schema, &checkpoint, execution)
            .map_err(|_| Denial::ObligationSubjectMismatch)?;
        let target = branches
            .branches()
            .iter()
            .find(|known| {
                known.branch_identity().name().as_str() == row.branch
                    && known.lifecycle_incarnation().ordinal() == incarnation.ordinal()
                    && known.owner_identity().durable() == Some(row.commit.owner())
            })
            .ok_or(Denial::ObligationSubjectMismatch)?;
        let observation = observe_head(target.lifecycle_incarnation())?;
        decoded.push((
            row.commit,
            row.branch,
            row.producer,
            observation,
            checkpoint,
            correspondence,
        ));
    }
    let lineage = application
        .inner
        .runtime
        .primary_provider_ref()
        .graph_ref()
        .output_lineage_ref()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut latest_slot = std::collections::BTreeMap::new();
    let mut latest_family = std::collections::BTreeMap::new();
    for (commit, branch, producer, _, checkpoint, correspondence) in &decoded {
        let slot_key = (
            branch.clone(),
            commit.owner(),
            producer.clone(),
            checkpoint.scope,
            checkpoint.source_partition,
        );
        latest_slot
            .entry(slot_key)
            .and_modify(|max: &mut u64| *max = (*max).max(commit.ordinal()))
            .or_insert(commit.ordinal());
        if let Some((family, role)) = lineage
            .checkpoint_family_role(correspondence.binding_type())
            .map_err(|_| Denial::ObligationSubjectMismatch)?
            && let Some(entity) = correspondence.publication_entity_for_role(role)
        {
            let family_key = (
                branch.clone(),
                commit.owner(),
                family.to_owned(),
                role.to_owned(),
                checkpoint.scope,
                checkpoint.source_partition,
                entity,
            );
            latest_family
                .entry(family_key)
                .and_modify(|max: &mut u64| *max = (*max).max(commit.ordinal()))
                .or_insert(commit.ordinal());
        }
    }
    let mut seen_slots = std::collections::BTreeSet::new();
    for (commit, branch, producer, observation, checkpoint, correspondence) in
        decoded.into_iter().rev()
    {
        let slot_key = (
            branch.clone(),
            commit.owner(),
            producer,
            checkpoint.scope,
            checkpoint.source_partition,
        );
        if latest_slot.get(&slot_key).copied() != Some(commit.ordinal()) {
            continue;
        }
        if let Some((family, role)) = lineage
            .checkpoint_family_role(correspondence.binding_type())
            .map_err(|_| Denial::ObligationSubjectMismatch)?
            && let Some(entity) = correspondence.publication_entity_for_role(role)
        {
            let family_key = (
                branch,
                commit.owner(),
                family.to_owned(),
                role.to_owned(),
                checkpoint.scope,
                checkpoint.source_partition,
                entity,
            );
            if latest_family.get(&family_key).copied() != Some(commit.ordinal()) {
                continue;
            }
        }
        if !seen_slots.insert(slot_key) {
            continue;
        }
        accepted.push(
            super::super::application_output_demand::WorthQueryReadmittedAcceptedOutput {
                observation: Some(observation),
                checkpoint,
                correspondence,
            },
        );
    }
    drop(lineage);
    application
        .inner
        .runtime
        .recovered_outputs_ref()
        .readmit_journal(accepted, execution)
        .map_err(|_| Denial::MalformedFrame)?;
    if let Some(support) = application.installed_program_support() {
        let recovered = application.inner.runtime.recovered_outputs_ref();
        for (commit, branch) in support.obligation_history().active_output_basis_pins() {
            execution.consult().map_err(Denial::Execution)?;
            if recovered.has_obligation_basis_pin(commit) {
                continue;
            }
            if originals.recorded_entry(commit).is_none() {
                return Err(Denial::ObligationSubjectMismatch);
            }
            let expected_incarnation = match originals
                .branch_status_for_commit(commit, &branch, execution)
                .map_err(Denial::Execution)?
            {
                Some((_, true)) => return Err(Denial::ObligationSubjectMismatch),
                Some((incarnation, false)) => Some(incarnation.ordinal()),
                None => None,
            };
            let current = branches
                .branches()
                .iter()
                .find(|known| {
                    known.branch_identity().name().as_str() == branch
                        && expected_incarnation.is_none_or(|ordinal| {
                            known.lifecycle_incarnation().ordinal() == ordinal
                        })
                        && known.owner_identity().durable() == Some(commit.owner())
                })
                .ok_or(Denial::ObligationSubjectMismatch)?;
            if current.selected_commit().owner_identity().durable() == Some(commit.owner())
                && current.selected_commit().ordinal() == commit.ordinal()
            {
                let observation = observe_head(current.lifecycle_incarnation())?;
                recovered.pin_obligation_basis(commit, observation);
                continue;
            }
            let mut history = world
                .trace_product_branch_ancestry(execution, current.lifecycle_incarnation(), None)
                .map_err(|_| match execution.consult() {
                    Err(cause) => Denial::Execution(cause),
                    Ok(()) => Denial::ObligationSubjectMismatch,
                })?;
            loop {
                execution.consult().map_err(Denial::Execution)?;
                if let Some(index) = history.commits().position(|entry| {
                    entry.identity().owner_identity().durable() == Some(commit.owner())
                        && entry.identity().ordinal() == commit.ordinal()
                }) {
                    let observation = world
                        .observe_product_branch_history_entry(&history, index)
                        .map_err(|_| Denial::ObligationSubjectMismatch)?;
                    recovered.pin_obligation_basis(commit, observation);
                    break;
                }
                if history.is_complete() {
                    return Err(Denial::ObligationSubjectMismatch);
                }
                history = world
                    .continue_product_branch_ancestry(execution, &history, None)
                    .map_err(|_| match execution.consult() {
                        Err(cause) => Denial::Execution(cause),
                        Ok(()) => Denial::ObligationSubjectMismatch,
                    })?;
            }
        }
    }
    restore(application, execution).map_err(|_| Denial::ObligationSubjectMismatch)
}

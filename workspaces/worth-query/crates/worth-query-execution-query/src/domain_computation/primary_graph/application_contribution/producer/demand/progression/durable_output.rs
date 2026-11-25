//! Preserve exact accepted facts across quiescent Directory teardown.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::{self as checkpoint, facts};
use crate::domain_computation::primary_graph::application_output_demand::{
    AcceptedCheckpointFactSource as Source, WorthQueryOutputDemandKey,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn retain_accepted_output(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &WorthQueryOutputDemandKey,
        basis: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        occurrence: Option<worth_runtime_world::facade::ProductBranchIncarnation>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(basis) = basis.filter(|basis| basis.owner_identity().durable().is_some()) else {
            return Ok(());
        };
        self.flush_pending_coupled_outputs(execution)?;
        let support = self.installed_program_support().ok_or_else(unavailable)?;
        let Some((mut identity, source, _)) =
            self.runtime.output_demands.durable_checkpoint_record(key)
        else {
            return Ok(());
        };
        if let Some(source) = source {
            let lineage = self
                .runtime
                .primary_provider
                .graph_ref()
                .output_lineage_ref()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            identity.producer_facts = match source {
                Source::Committed(receipt) => lineage
                    .checkpoint_facts_for_publication((&receipt).into())
                    .and_then(|(facts, witness)| {
                        checkpoint::capture::output_facts::encode(&facts, witness)
                    }),
                Source::Recovered(receipt) => lineage
                    .checkpoint_facts_for_publication(receipt.view())
                    .and_then(|(facts, witness)| {
                        checkpoint::capture::output_facts::encode(&facts, witness)
                    }),
                Source::Stable(stable) => stable.checkpoint_source_facts().and_then(|source| {
                    match stable.checkpoint_output_witness() {
                        Some(witness) => {
                            checkpoint::capture::output_facts::encode(&source, witness)
                        }
                        None => facts::encode(&source),
                    }
                }),
            }
            .map(std::sync::Arc::new);
            identity.producer_fact_wire_version = if identity.producer_facts.is_some() {
                facts::WIRE_VERSION
            } else {
                0
            };
        }
        let payload =
            checkpoint::accepted_output::encode(&identity, execution.allocation_policy())
                .map_err(|_| unavailable())?;
        let observation = self
            .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                occurrence.ok_or_else(unavailable)?,
            ))
            .select_with_request(execution)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::product_selection(cause, "accepted output retention")
            })?;
        let commit = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            basis.owner_identity().durable().expect("durable checked"),
            basis.ordinal(),
        );
        support.lifecycle().record_accepted_output(execution, crate::domain_computation::primary_graph::query_obligations::QueryAcceptedOutputRecord {
            commit, branch: observation.product().observation().branch_identity().name().as_str().to_owned(), producer: identity.producer.as_ref().clone(),
            scope: (identity.scope.partition_id(),identity.scope.local_slot(),identity.scope.generation()), partition: identity.source_partition, payload,
        }).map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ObligationPublication(cause), "accepted output retention"))
    }
}
fn unavailable() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        "accepted output retention",
    )
}

impl<S> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Flush any coupled output records already committed by World that remain
    /// in retryable owner custody because an earlier write stopped or failed.
    pub fn flush_pending_coupled_outputs(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let lineage_lock = self.runtime.primary_provider.graph_ref().output_lineage_ref();
        if !lineage_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .has_pending_coupled_outputs()
        {
            return Ok(());
        }
        let map_stop = |cause| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ExecutionRequest(
                    crate::domain_computation::primary_graph::application_contribution::execution_denial::advancement_scope_stop(cause),
                ),
                "coupled output retention",
            )
        };
        execution.consult().map_err(map_stop)?;
        let support = self.installed_program_support().ok_or_else(unavailable)?;
        let result = execution.in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    loop {
                        let item = {
                            lineage_lock
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner)
                                .peek_pending_coupled_output()
                        };
                        let Some(item) = item else {
                            break;
                        };
                        context
                            .checkpoint(1)
                            .map_err(|stop| map_stop(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
                        let payload = checkpoint::accepted_output::encode(
                            &item.identity,
                            execution.allocation_policy(),
                        )
                        .map_err(|_| unavailable())?;
                        support
                            .lifecycle()
                            .record_accepted_output(
                                execution,
                                crate::domain_computation::primary_graph::query_obligations::QueryAcceptedOutputRecord {
                                    commit: item.commit,
                                    branch: item.branch.clone(),
                                    producer: item.identity.producer.as_ref().clone(),
                                    scope: (
                                        item.identity.scope.partition_id(),
                                        item.identity.scope.local_slot(),
                                        item.identity.scope.generation(),
                                    ),
                                    partition: item.identity.source_partition,
                                    payload,
                                },
                            )
                            .map_err(|cause| {
                                WorthQueryOutputDemandDenial::new(
                                    WorthQueryOutputDemandDenialKind::ObligationPublication(cause),
                                    "coupled output retention",
                                )
                            })?;
                        lineage_lock
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .complete_pending_coupled_output(&item);
                    }
                    Ok(())
                })
        });
        result.map_err(map_stop)?.map_err(map_stop)?.0
    }

    /// Source publication already completed these exact coupled writers. Enqueue
    /// all immutable facts in retryable owner custody before attempting durable
    /// persistence so an interruption between members never drops unwritten rows.
    pub fn retain_coupled_output_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(owner) = observation.owner_identity().durable() else {
            return Ok(());
        };
        let commit = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            owner,
            observation.selected_commit().ordinal(),
        );
        self.retain_coupled_output_records_for(
            execution,
            receipt,
            commit,
            observation.branch_identity().name().as_str(),
        )
    }

    pub fn retain_coupled_output_records_from_receipt(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let publication = receipt.committed_product_publication();
        let Some(owner) = publication.composite_commit().owner_identity().durable() else {
            return Ok(());
        };
        let commit = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            owner,
            publication.composite_commit().ordinal(),
        );
        self.retain_coupled_output_records_for(
            execution,
            receipt,
            commit,
            publication.product_branch().name().as_str(),
        )
    }

    fn retain_coupled_output_records_for(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        commit: worth_foundational::facade::DurableCompositeCommitIdentity,
        branch: &str,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let provider = &self.runtime.primary_provider;
        let committed = provider
            .observe_completed_application(receipt.commit_reference())
            .ok_or_else(unavailable)?;
        provider
            .graph_ref()
            .output_lineage_ref()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .enqueue_coupled_outputs_from_committed(commit, branch, &committed)?;
        self.flush_pending_coupled_outputs(execution)
    }
}

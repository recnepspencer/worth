//! Runtime settlement construction, observation, and selection hooks.

use std::sync::Arc;
use worth_query_execution_lineage::facade::ports::{
    CommitAuthorityBindingInternalPort as _, CommitReceiptInternalPort as _,
    OutputDemandSettlementInternalPort as _, OutputDemandSettlementParts,
};
use worth_query_execution_runtime::facade::ports::ProductObservationLeaseInternalPort as _;

use super::current_commit::reacquire_current_committed_output;
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::application_output_demand::ReadyCompletion;
use crate::domain_computation::primary_graph::ReadObservationRuntimeExt;

pub trait OriginalSettledProducerObservationExt: Sized {
    fn ready<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>;

    fn stable<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        stable: &crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>;

    fn restored<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        restored: &super::super::WorthQueryRestoredAcceptedOutput,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>;

    fn matches<Schema: worth_query_installation::facade::ApplicationSchema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool;

    fn matches_read(&self, product: &worth_runtime_world::facade::ProductBranchObservation)
        -> bool;
}

impl OriginalSettledProducerObservationExt for OriginalSettledProducerObservation {
    fn ready<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        let (source, correspondence) = match &completion.authority {
            WorthQueryAcceptedOutputAuthority::Committed(receipt) => (
                OriginalSettledProducerSource::Committed {
                    settlement: receipt.exact_output_settlement().cloned(),
                    publication: receipt
                        .committed_product_publication()
                        .observed_product_identity(),
                    changes: receipt.committed_changes().clone(),
                    inputs: receipt.performed_input_facts().cloned(),
                    conditional: completion.readiness.causal_observation().cloned(),
                    evaluation: completion.readiness.output_evaluation().cloned(),
                    disclosure: receipt
                        .authority_binding()
                        .retain_original_inspection_disclosure(),
                    original_publication: completion.original_producing_publication().cloned(),
                },
                receipt.retain_output_correspondence(),
            ),
            WorthQueryAcceptedOutputAuthority::Recovered(publication) => (
                OriginalSettledProducerSource::Committed {
                    settlement: publication.view().exact_output_settlement().cloned(),
                    publication: publication
                        .view()
                        .committed_product_publication()
                        .observed_product_identity(),
                    changes: publication.view().committed_changes().clone(),
                    inputs: publication.view().performed_input_facts().cloned(),
                    conditional: completion.readiness.causal_observation().cloned(),
                    evaluation: completion.readiness.output_evaluation().cloned(),
                    disclosure: publication
                        .view()
                        .authority_binding()
                        .retain_original_inspection_disclosure(),
                    original_publication: completion.original_producing_publication().cloned(),
                },
                publication.view().retain_output_correspondence(),
            ),
            WorthQueryAcceptedOutputAuthority::Stable(stable) => (
                OriginalSettledProducerSource::StableReused {
                    settlement: Arc::clone(stable.exact_settlement()),
                },
                Arc::clone(stable.output_correspondence()),
            ),
            WorthQueryAcceptedOutputAuthority::Restored(restored) => (
                restored_source(restored),
                Arc::clone(&restored.correspondence),
            ),
        };
        capture_settled_observation(execution, runtime, product, source, correspondence)
    }

    fn stable<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        stable: &crate::domain_computation::primary_graph::output_lineage::PublishedStableLineage,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        capture_settled_observation(
            execution,
            runtime,
            product,
            OriginalSettledProducerSource::StableReused {
                settlement: Arc::clone(stable.exact_settlement()),
            },
            Arc::clone(stable.output_correspondence()),
        )
    }

    fn restored<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        restored: &super::super::WorthQueryRestoredAcceptedOutput,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        capture_settled_observation(
            execution,
            runtime,
            product,
            restored_source(restored),
            Arc::clone(&restored.correspondence),
        )
    }

    fn matches<Schema: worth_query_installation::facade::ApplicationSchema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.matches_occurrence(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            product,
        )
    }

    fn matches_read(
        &self,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.read().matches_observation(product)
    }
}

fn capture_settled_observation<Schema: worth_query_installation::facade::ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: &WorthQueryProductObservationLease,
    source: OriginalSettledProducerSource,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
) -> Result<Arc<OriginalSettledProducerObservation>, WorthQueryOutputDemandDenial> {
    let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<OriginalSettledProducerObservation>())
        .expect("fixed settlement observation layout")
        .0
        .pad_to_align();
    let memory = execution
        .reserve_retained_memory(layout.size() as u64)
        .map_err(|cause| {
            crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                crate::domain_computation::primary_graph::output_reuse::RetainedFactStop::Execution(
                    cause.into(),
                ),
                "original settled producer observation",
            )
        })?;
    let historical_reads = match &source {
        OriginalSettledProducerSource::Committed {
            settlement: Some(identity),
            ..
        }
        | OriginalSettledProducerSource::StableReused {
            settlement: identity,
        } => runtime
            .runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain_predecessor_reads(identity),
        OriginalSettledProducerSource::Committed {
            settlement: None, ..
        }
        | OriginalSettledProducerSource::Restored { .. }
        | OriginalSettledProducerSource::HistoricalSelection { .. } => None,
    };
    Ok(Arc::new(OriginalSettledProducerObservation {
        runtime_authority: runtime.runtime.authority_identity().as_u64(),
        schema: runtime.installed_schema.binding_identity(),
        read: WorthQueryProductBranchReadIdentity::from_observation(product.observation()),
        source,
        correspondence,
        historical_reads,
        _memory: memory,
    }))
}

fn restored_source(
    restored: &super::super::WorthQueryRestoredAcceptedOutput,
) -> OriginalSettledProducerSource {
    match &restored.origin {
        super::super::RestoredOutputOrigin::Checkpoint(checkpoint) => {
            OriginalSettledProducerSource::Restored {
                checkpoint: checkpoint.clone(),
            }
        }
        super::super::RestoredOutputOrigin::HistoricalSelection(origin) => {
            OriginalSettledProducerSource::HistoricalSelection {
                origin: origin.clone(),
            }
        }
    }
}

pub trait OutputDemandSettlementRuntimeExt: Sized {
    fn belongs_to<Schema: worth_query_installation::facade::ApplicationSchema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> bool;

    fn from_committed_ready<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        producer_identity: &str,
        output_family_identity: &str,
        producer_contacts_in_this_demand: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>;

    fn from_original_program_ready<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        producer: &str,
        family: &str,
        contacts: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>;
}

impl OutputDemandSettlementRuntimeExt for WorthQueryOutputDemandSettlement {
    fn belongs_to<Schema: worth_query_installation::facade::ApplicationSchema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> bool {
        self.matches_runtime_owner(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
        )
    }

    fn from_committed_ready<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        producer_identity: &str,
        output_family_identity: &str,
        producer_contacts_in_this_demand: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        let Some(receipt) = completion.authority.publication_view() else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "committed settlement requires its actual Ready completion",
            ));
        };
        let readiness_delivery = &completion.readiness;
        let _held_world_observations = if runtime
            .runtime
            .primary_provider
            .take_ready_read_snapshot_pressure()
        {
            Some(runtime.hold_world_snapshot_pressure_for_test(receipt.product_branch()))
        } else {
            None
        };
        let observation = reacquire_current_committed_output(execution, runtime, receipt)?
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "authoritative output is no longer current in its product occurrence",
                )
            })?;
        Ok(Arc::new(Self::from_parts(OutputDemandSettlementParts {
            original_delivery_memory: None,
            posture: match &completion.authority {
                WorthQueryAcceptedOutputAuthority::Committed(_) => {
                    WorthQueryOutputSettlementPosture::Performed
                }
                WorthQueryAcceptedOutputAuthority::Recovered(_) => {
                    WorthQueryOutputSettlementPosture::RecoveredPerformed
                }
                WorthQueryAcceptedOutputAuthority::Stable(_)
                | WorthQueryAcceptedOutputAuthority::Restored(_) => {
                    unreachable!("actual performed evidence checked")
                }
            },
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: producer_identity.to_owned().into(),
            output_family_identity: output_family_identity.to_owned(),
            receipt: Some(match &completion.authority {
                WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                    SettlementReceiptCustody::Owned(receipt.clone())
                }
                WorthQueryAcceptedOutputAuthority::Recovered(publication) => {
                    SettlementReceiptCustody::Recovered(publication.clone())
                }
                WorthQueryAcceptedOutputAuthority::Stable(_)
                | WorthQueryAcceptedOutputAuthority::Restored(_) => {
                    unreachable!("actual performed evidence checked")
                }
            }),
            stable: None,
            output_correspondence: receipt.retain_output_correspondence(),
            restored_source: None,
            readiness_delivery: Some(readiness_delivery.clone()),
            producer_contacts_in_this_demand,
            observation: WorthQueryApplicationReadObservation::from_settled_product(
                runtime,
                OriginalSettledProducerObservation::ready(
                    execution,
                    runtime,
                    completion,
                    &observation,
                )?,
                observation,
            ),
        })))
    }

    fn from_original_program_ready<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        producer: &str,
        family: &str,
        contacts: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed original result header")
            .0
            .pad_to_align();
        let bytes = (layout.size() as u64)
            .saturating_add(producer.len() as u64)
            .saturating_add(family.len() as u64);
        let memory = crate::domain_computation::primary_graph::structural_walk::memory::reserve(
            execution,
            execution.has_accounting(),
            bytes,
        )
        .map_err(|cause| {
            crate::domain_computation::primary_graph::output_reuse::read_denial(cause).with_subject(
                format!("{producer}: {family}: original program root delivery"),
            )
        })?;
        let mut original =
            Self::from_committed_ready(execution, runtime, completion, producer, family, contacts)?;
        Arc::get_mut(&mut original)
            .expect("the new original delivery has one owner")
            .set_original_delivery_memory(memory);
        Ok(original)
    }
}

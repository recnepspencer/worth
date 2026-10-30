//! The real settlement's readonly seed, separate from its operational read lease.
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::{
    output_lineage::RecordedSettlementIdentity,
    source_disclosure::OriginalSourceInspectionDisclosure, WorthQueryApplicationCommittedChanges,
};

/// No variant asserts that a query row or field was produced by this source.
/// Fresh metadata admission and exact revision/output overlap remain necessary.
pub(in crate::domain_computation::primary_graph) enum OriginalSettledProducerSource {
    Committed {
        settlement: Option<Arc<RecordedSettlementIdentity>>,
        publication: Option<WorthQueryProductBranchReadIdentity>,
        changes: WorthQueryApplicationCommittedChanges,
        inputs: Option<
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        >,
        conditional: Option<worth_runtime_bridge::facade::BridgeConditionalCausalObservation>,
        evaluation: Option<Arc<crate::domain_computation::primary_graph::application_contribution::ProducerOutputEvaluation>>,
        disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
        original_publication: Option<Arc<super::OriginalProducingPublication>>,
    },
    StableReused {
        settlement: Arc<RecordedSettlementIdentity>,
    },
    Restored {
        checkpoint: super::super::WorthQueryAcceptedOutputCheckpointIdentity,
    },
}

/// One original-request funded header sharing only original immutable backings.
/// It owns no Product observation, Native reader, Ready cell or commit receipt.
pub(in crate::domain_computation::primary_graph) struct OriginalSettledProducerObservation {
    runtime_authority: u64,
    schema: ApplicationSchemaBindingIdentity,
    read: WorthQueryProductBranchReadIdentity,
    source: OriginalSettledProducerSource,
    correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl std::fmt::Debug for OriginalSettledProducerObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OriginalSettledProducerObservation")
            .finish_non_exhaustive()
    }
}

impl OriginalSettledProducerObservation {
    pub(super) fn ready<Schema: worth_query_installation::facade::ApplicationSchema>(
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
            WorthQueryAcceptedOutputAuthority::Stable(stable) => (
                OriginalSettledProducerSource::StableReused {
                    settlement: Arc::clone(stable.exact_settlement()),
                },
                Arc::clone(stable.output_correspondence()),
            ),
            WorthQueryAcceptedOutputAuthority::Restored(restored) => (
                OriginalSettledProducerSource::Restored {
                    checkpoint: restored.checkpoint.clone(),
                },
                Arc::clone(&restored.correspondence),
            ),
        };
        Self::capture(execution, runtime, product, source, correspondence)
    }

    pub(super) fn stable<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        stable: &super::super::super::output_lineage::PublishedStableLineage,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        Self::capture(
            execution,
            runtime,
            product,
            OriginalSettledProducerSource::StableReused {
                settlement: Arc::clone(stable.exact_settlement()),
            },
            Arc::clone(stable.output_correspondence()),
        )
    }

    pub(super) fn restored<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        restored: &super::super::WorthQueryRestoredAcceptedOutput,
        product: &WorthQueryProductObservationLease,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        Self::capture(
            execution,
            runtime,
            product,
            OriginalSettledProducerSource::Restored {
                checkpoint: restored.checkpoint.clone(),
            },
            Arc::clone(&restored.correspondence),
        )
    }

    fn capture<Schema: worth_query_installation::facade::ApplicationSchema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &WorthQueryProductObservationLease,
        source: OriginalSettledProducerSource,
        correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial> {
        // Match alloc::sync's repr(C) two-atomic ArcInner header, including
        // alignment between that header and this concrete payload.
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("fixed settlement observation layout")
            .0
            .pad_to_align();
        let memory = execution.reserve_retained_memory(layout.size() as u64).map_err(|cause| {
            crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                crate::domain_computation::primary_graph::output_reuse::RetainedFactStop::Execution(cause.into()),
                "original settled producer observation",
            )
        })?;
        Ok(Arc::new(Self {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema: runtime.installed_schema.binding_identity(),
            read: WorthQueryProductBranchReadIdentity::from_observation(product.observation()),
            source,
            correspondence,
            _memory: memory,
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn matches<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.runtime_authority == runtime.runtime.authority_identity().as_u64()
            && self.schema == runtime.installed_schema.binding_identity()
            && self.read.matches_observation(product)
    }

    pub(in crate::domain_computation::primary_graph) fn matches_read(
        &self,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> bool {
        self.read.matches_observation(product)
    }

    pub(in crate::domain_computation::primary_graph) fn read(
        &self,
    ) -> &WorthQueryProductBranchReadIdentity {
        &self.read
    }
    pub(in crate::domain_computation::primary_graph) fn source(
        &self,
    ) -> &OriginalSettledProducerSource {
        &self.source
    }
    pub(in crate::domain_computation::primary_graph) fn correspondence(
        &self,
    ) -> &WorthQueryApplicationOutputCorrespondence {
        &self.correspondence
    }
    pub(in crate::domain_computation::primary_graph) fn retain_correspondence(
        &self,
    ) -> Arc<WorthQueryApplicationOutputCorrespondence> {
        Arc::clone(&self.correspondence)
    }
}

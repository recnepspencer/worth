use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationReadObservation,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};

/// Owner-retained result of one settled output demand.
///
/// The retained World observation is the authority for later exact reads. The
/// receipt remains descriptive publication evidence and is never used to
/// reconstruct or search for the selected occurrence.
pub struct WorthQueryOutputDemandSettlement {
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    receipt: WorthQueryApplicationCommitReceipt,
    readiness_delivery: Option<WorthQueryOutputReadinessDeliveryEvidence>,
    observation: Arc<WorthQueryApplicationReadObservation>,
}

impl WorthQueryOutputDemandSettlement {
    pub(in crate::domain_computation::primary_graph) fn from_commit<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: WorthQueryApplicationCommitReceipt,
        readiness_delivery: WorthQueryOutputReadinessDeliveryEvidence,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        let observation = receipt
            .committed_product_publication()
            .take_output_demand_observation()
            .map(WorthQueryProductObservationLease::new)
            .or_else(|| reacquire_current_committed_output(runtime, &receipt))
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "authoritative output occurrence is no longer the current World head",
                )
            })?;
        Ok(Arc::new(Self {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            receipt,
            readiness_delivery: Some(readiness_delivery),
            observation: WorthQueryApplicationReadObservation::from_product(runtime, observation),
        }))
    }

    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        &self.receipt
    }

    pub fn readiness_delivery(&self) -> Option<&WorthQueryOutputReadinessDeliveryEvidence> {
        self.readiness_delivery.as_ref()
    }

    #[doc(hidden)]
    pub fn retained_read(&self) -> Arc<WorthQueryApplicationReadObservation> {
        Arc::clone(&self.observation)
    }
}

/// Query-owned evidence that a fresh output patch reached its declared
/// readiness conditional and produced a Signal successor.
pub struct WorthQueryOutputReadinessDeliveryEvidence {
    conditional_successor: bool,
    truth_targets_admitted: usize,
    signal_seeds_emitted: usize,
    slots_touched: usize,
    signal_decision: crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision,
    semantic_observation_reads: usize,
}

impl WorthQueryOutputReadinessDeliveryEvidence {
    pub(in crate::domain_computation::primary_graph) fn from_execution(
        delivery: Option<&worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt>,
        execution: &worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    ) -> Self {
        Self {
            conditional_successor: delivery.is_some_and(|receipt| receipt.has_conditional_successor()),
            truth_targets_admitted: delivery.map_or(0, |receipt| receipt.truth_targets_admitted()),
            signal_seeds_emitted: delivery.map_or(0, |receipt| receipt.signal_seeds_emitted()),
            slots_touched: delivery.map_or(0, |receipt| receipt.slots_touched()),
            signal_decision: crate::domain_computation::primary_graph::conditional_operation::classify_bridge_signal(execution),
            semantic_observation_reads: execution.semantic_observation_reads(),
        }
    }

    pub const fn has_conditional_successor(&self) -> bool {
        self.conditional_successor
    }

    pub const fn truth_targets_admitted(&self) -> usize {
        self.truth_targets_admitted
    }

    pub const fn signal_seeds_emitted(&self) -> usize {
        self.signal_seeds_emitted
    }

    pub const fn slots_touched(&self) -> usize {
        self.slots_touched
    }

    pub const fn signal_decision(
        &self,
    ) -> crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision {
        self.signal_decision
    }

    pub const fn semantic_observation_reads(&self) -> usize {
        self.semantic_observation_reads
    }
}

fn reacquire_current_committed_output<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> Option<WorthQueryProductObservationLease>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let committed = receipt.committed_product_publication();
    let selected = runtime.on_branch(receipt.product_branch()).select().ok()?;
    let observation = selected.product().observation();
    (observation.lifecycle_incarnation() == committed.product_incarnation()
        && observation.reference_generation() == committed.product_generation()
        && observation.selected_commit() == committed.composite_commit())
    .then(|| selected.product().read_lease())
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub fn select_output_demand_settlement(
        &self,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<WorthQuerySelectedProductOperation<'_, Schema>, WorthQueryOutputDemandDenial> {
        if settlement.runtime_authority != self.runtime.authority_identity().as_u64()
            || settlement.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "output settlement belongs to another application runtime",
            ));
        }
        self.select_application_read_observation(&settlement.observation)
            .map_err(|error| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    format!("retained output basis unavailable: {error:?}"),
                )
            })
    }
}

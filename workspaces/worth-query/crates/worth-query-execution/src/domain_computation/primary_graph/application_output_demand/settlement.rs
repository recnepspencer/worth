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
/// The retained World observation is the authority for exact reads while a
/// settlement is held. After compaction, the receipt identifies the same
/// committed lineage so the owner can reacquire its still-current occurrence.
pub struct WorthQueryOutputDemandSettlement {
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    receipt: WorthQueryApplicationCommitReceipt,
    readiness_delivery: Option<WorthQueryOutputReadinessDeliveryEvidence>,
    observation: Arc<WorthQueryApplicationReadObservation>,
}

impl WorthQueryOutputDemandSettlement {
    pub(in crate::domain_computation::primary_graph) fn belongs_to<Schema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> bool
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        self.runtime_authority == runtime.runtime.authority_identity().as_u64()
            && self.schema_binding == runtime.installed_schema.binding_identity()
    }

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
                    "authoritative output is not current in the selected product occurrence",
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

    pub(in crate::domain_computation::primary_graph) fn completion(
        self: &Arc<Self>,
        retain_exact: bool,
    ) -> super::registry::WorthQueryCompletedOutputDemand {
        super::registry::WorthQueryCompletedOutputDemand {
            receipt: self.receipt.clone(),
            readiness: self
                .readiness_delivery
                .as_ref()
                .expect("a settled output retains readiness completion")
                .clone(),
            retained: retain_exact.then(|| Arc::clone(self)),
        }
    }

    #[doc(hidden)]
    pub fn retained_read(&self) -> Arc<WorthQueryApplicationReadObservation> {
        Arc::clone(&self.observation)
    }
}

/// Query-owned evidence that a fresh output patch reached its declared
/// readiness conditional and produced a Signal successor.
#[derive(Clone)]
pub struct WorthQueryOutputReadinessDeliveryEvidence {
    // `from_execution` is reached only after the installed producer boundary
    // returned the receipt whose readiness is evaluated.
    producer_contacts: usize,
    // A Bridge delivery contact exists only when that receipt carried a fresh
    // performed change into readiness evaluation.
    delivery_contacts: usize,
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
            producer_contacts: 1,
            delivery_contacts: usize::from(delivery.is_some()),
            conditional_successor: delivery.is_some_and(|receipt| receipt.has_conditional_successor()),
            truth_targets_admitted: delivery.map_or(0, |receipt| receipt.truth_targets_admitted()),
            signal_seeds_emitted: delivery.map_or(0, |receipt| receipt.signal_seeds_emitted()),
            slots_touched: delivery.map_or(0, |receipt| receipt.slots_touched()),
            signal_decision: crate::domain_computation::primary_graph::conditional_operation::classify_bridge_signal(execution),
            semantic_observation_reads: execution.semantic_observation_reads(),
        }
    }

    pub const fn producer_contact_count(&self) -> usize {
        self.producer_contacts
    }

    pub const fn delivery_contact_count(&self) -> usize {
        self.delivery_contacts
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
    let original_publication_is_current = observation.lifecycle_incarnation()
        == committed.product_incarnation()
        && observation.reference_generation() == committed.product_generation()
        && observation.selected_commit() == committed.composite_commit();
    let retained_output_is_current = runtime
        .primary_provider
        .graph
        .output_lineage
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .current_output_matches_receipt(
            runtime.runtime.authority_identity().as_u64(),
            &runtime.installed_schema.binding_identity(),
            observation,
            receipt,
        );
    (original_publication_is_current || retained_output_is_current)
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

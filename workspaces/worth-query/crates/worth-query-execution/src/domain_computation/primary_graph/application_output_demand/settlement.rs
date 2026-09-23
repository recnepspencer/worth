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
    producer_identity: String,
    output_family_identity: String,
    pub(in crate::domain_computation::primary_graph) receipt:
        Option<WorthQueryApplicationCommitReceipt>,
    pub(in crate::domain_computation::primary_graph) output_correspondence:
        Arc<crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence>,
    pub(in crate::domain_computation::primary_graph) restored_source:
        Option<WorthQueryRestoredOutputSource>,
    readiness_delivery: Option<WorthQueryOutputReadinessDeliveryEvidence>,
    observation: Arc<WorthQueryApplicationReadObservation>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryRestoredOutputSource {
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) identity:
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
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
        receipt: &WorthQueryApplicationCommitReceipt,
        readiness_delivery: &WorthQueryOutputReadinessDeliveryEvidence,
        producer_identity: &str,
        output_family_identity: &str,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        #[cfg(feature = "test-primary-graph-faults")]
        let _held_world_observations =
            if runtime.primary_provider.take_ready_read_snapshot_pressure() {
                Some(runtime.hold_world_snapshot_pressure_for_test(receipt))
            } else {
                None
            };
        let observation =
            reacquire_current_committed_output(runtime, receipt)?.ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "authoritative output is no longer current in its product occurrence",
                )
            })?;
        Ok(Arc::new(Self {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: producer_identity.to_owned(),
            output_family_identity: output_family_identity.to_owned(),
            receipt: Some(receipt.clone()),
            output_correspondence: receipt.retain_output_correspondence(),
            restored_source: None,
            readiness_delivery: Some(readiness_delivery.clone()),
            observation: WorthQueryApplicationReadObservation::from_product(runtime, observation),
        }))
    }

    pub fn application_commit_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        self.receipt.as_ref()
    }

    pub fn output_correspondence(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence {
        self.output_correspondence.as_ref()
    }

    pub fn producer_identity(&self) -> &str {
        &self.producer_identity
    }

    pub fn output_family_identity(&self) -> &str {
        &self.output_family_identity
    }

    pub fn readiness_delivery(&self) -> Option<&WorthQueryOutputReadinessDeliveryEvidence> {
        self.readiness_delivery.as_ref()
    }

    #[doc(hidden)]
    pub fn retained_read(&self) -> Arc<WorthQueryApplicationReadObservation> {
        Arc::clone(&self.observation)
    }

    pub(in crate::domain_computation::primary_graph) fn from_restoration<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        restored: &super::WorthQueryRestoredAcceptedOutput,
        output_family_identity: &str,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        let authority = runtime
            .product_runtime
            .recovered_root_authority
            .as_ref()
            .filter(|authority| authority.product_branch() == &restored.observation)
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "restored output is not bound to this World's recovered root authority",
                )
            })?;
        let product = runtime
            .product_runtime
            .lease_from_observation(authority.product_branch().clone())
            .map_err(|error| {
                WorthQueryOutputDemandDenial::product_selection(
                    error,
                    "recovered output root is no longer current",
                )
            })?;
        Ok(Arc::new(Self {
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: restored.checkpoint.producer.clone(),
            output_family_identity: output_family_identity.to_owned(),
            receipt: None,
            output_correspondence: Arc::clone(&restored.correspondence),
            restored_source: Some(WorthQueryRestoredOutputSource {
                scope: restored.source_scope,
                identity: restored.source_identity,
            }),
            readiness_delivery: None,
            observation: WorthQueryApplicationReadObservation::from_product(
                runtime,
                product.read_lease(),
            ),
        }))
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
    pub(in crate::domain_computation::primary_graph) const fn from_restoration() -> Self {
        Self {
            producer_contacts: 0,
            delivery_contacts: 0,
            conditional_successor: false,
            truth_targets_admitted: 0,
            signal_seeds_emitted: 0,
            slots_touched: 0,
            signal_decision: crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision::DependencyUnchanged,
            semantic_observation_reads: 0,
        }
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) const fn for_test() -> Self {
        Self {
            producer_contacts: 1,
            delivery_contacts: 0,
            conditional_successor: false,
            truth_targets_admitted: 0,
            signal_seeds_emitted: 0,
            slots_touched: 0,
            signal_decision: crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision::DependencyUnchanged,
            semantic_observation_reads: 1,
        }
    }

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
) -> Result<Option<WorthQueryProductObservationLease>, WorthQueryOutputDemandDenial>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    let committed = receipt.committed_product_publication();
    let selected = runtime
        .on_branch(receipt.product_branch())
        .select()
        .map_err(|error| {
            WorthQueryOutputDemandDenial::product_selection(
                error,
                "settled output product observation could not be reacquired",
            )
        })?;
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
    Ok(
        (original_publication_is_current || retained_output_is_current)
            .then(|| selected.product().read_lease()),
    )
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

    pub(in crate::domain_computation::primary_graph) fn current_output_source_facts(
        &self,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<
        Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
        WorthQueryOutputDemandDenial,
    > {
        if !settlement.belongs_to(self) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "output settlement belongs to another application runtime",
            ));
        }
        let selected = match settlement.application_commit_receipt() {
            Some(receipt) => {
                self.on_branch(receipt.product_branch())
                    .select()
                    .map_err(|error| {
                        WorthQueryOutputDemandDenial::product_selection(
                            error,
                            "settled output currentness could not select its product occurrence",
                        )
                    })?
            }
            None => self.select_output_demand_settlement(settlement)?,
        };
        let lineage = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let facts = match settlement.application_commit_receipt() {
            Some(receipt) => lineage.source_facts_for_receipt(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                selected.product().observation(),
                receipt,
                usize::MAX,
            ),
            None => lineage.source_facts_for_restored_output(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                selected.product().observation(),
                settlement,
                usize::MAX,
            ),
        };
        facts
            .map_err(|()| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    "settled output currentness exceeded its lineage work budget",
                )
            })?
            .map(|(facts, _)| facts)
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "settled output no longer names the current output lineage",
                )
            })
    }
}

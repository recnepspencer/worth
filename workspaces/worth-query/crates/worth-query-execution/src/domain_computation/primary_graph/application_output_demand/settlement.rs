mod current_accepted;
mod receipt_custody;
use receipt_custody::SettlementReceiptCustody;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{ReadyCompletion, WorthQueryAcceptedOutputAuthority};
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, BoundCurrentAcceptedOutput,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputProjectionDenial, WorthQueryApplicationReadObservation,
    WorthQueryApplicationTypedOutputCorrespondence, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQuerySelectedProductOperation,
};

use super::WorthQueryOutputReadinessDeliveryEvidence;

/// Owner-retained result of one settled output demand.
///
/// The retained World observation is the authority for exact reads while a
/// settlement is held. After compaction, the receipt identifies the same
/// committed lineage so the owner can reacquire its still-current occurrence.
pub struct WorthQueryOutputDemandSettlement {
    posture: WorthQueryOutputSettlementPosture,
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    producer_identity: String,
    output_family_identity: String,
    receipt: Option<SettlementReceiptCustody>,
    pub(in crate::domain_computation::primary_graph) stable:
        Option<super::super::output_lineage::PublishedStableLineage>,
    pub(in crate::domain_computation::primary_graph) output_correspondence:
        Arc<WorthQueryApplicationOutputCorrespondence>,
    pub(in crate::domain_computation::primary_graph) restored_source:
        Option<WorthQueryRestoredOutputSource>,
    readiness_delivery: Option<WorthQueryOutputReadinessDeliveryEvidence>,
    producer_contacts_in_this_demand: usize,
    observation: Arc<WorthQueryApplicationReadObservation>,
}

/// Whether settlement performed an output or reused stable lineage, including
/// the corresponding posture recovered from an existing owner publication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputSettlementPosture {
    Performed,
    StableReused,
    RecoveredPerformed,
    RecoveredStableReused,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryRestoredOutputSource {
    pub(in crate::domain_computation::primary_graph) scope:
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    pub(in crate::domain_computation::primary_graph) identity:
        crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity,
    pub(in crate::domain_computation::primary_graph) partition: [u8; 32],
}

impl From<&super::WorthQueryRestoredAcceptedOutput> for WorthQueryRestoredOutputSource {
    fn from(restored: &super::WorthQueryRestoredAcceptedOutput) -> Self {
        Self {
            scope: restored.source_scope,
            identity: restored.source_identity,
            partition: restored.checkpoint.source_partition,
        }
    }
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
        producer_contacts_in_this_demand: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        #[cfg(feature = "test-primary-graph-faults")]
        let _held_world_observations =
            if runtime.primary_provider.take_ready_read_snapshot_pressure() {
                Some(runtime.hold_world_snapshot_pressure_for_test(receipt.product_branch()))
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
            posture: WorthQueryOutputSettlementPosture::Performed,
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: producer_identity.to_owned(),
            output_family_identity: output_family_identity.to_owned(),
            receipt: Some(SettlementReceiptCustody::Owned(receipt.clone())),
            stable: None,
            output_correspondence: receipt.retain_output_correspondence(),
            restored_source: None,
            readiness_delivery: Some(readiness_delivery.clone()),
            producer_contacts_in_this_demand,
            observation: WorthQueryApplicationReadObservation::from_product(runtime, observation),
        }))
    }

    pub fn application_commit_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        self.receipt
            .as_ref()
            .map(SettlementReceiptCustody::as_receipt)
    }

    pub const fn is_stably_reused(&self) -> bool {
        matches!(
            self.posture,
            WorthQueryOutputSettlementPosture::StableReused
                | WorthQueryOutputSettlementPosture::RecoveredStableReused
        )
    }

    pub const fn posture(&self) -> WorthQueryOutputSettlementPosture {
        self.posture
    }

    /// The settled output roles, read as the output contract `Contract`.
    pub fn outputs_of<Contract: 'static>(
        &self,
    ) -> Result<
        WorthQueryApplicationTypedOutputCorrespondence<'_, Contract>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        self.output_correspondence.outputs_of()
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

    /// Number of installed producer executions initiated by this admitted demand.
    /// This is not the producing commit's historical readiness evidence.
    pub const fn producer_contacts_in_this_demand(&self) -> usize {
        self.producer_contacts_in_this_demand
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
            posture: restored.settlement_posture(),
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: restored.checkpoint.producer.clone(),
            output_family_identity: output_family_identity.to_owned(),
            receipt: None,
            stable: None,
            output_correspondence: Arc::clone(&restored.correspondence),
            restored_source: Some(restored.into()),
            readiness_delivery: None,
            producer_contacts_in_this_demand: 0,
            observation: WorthQueryApplicationReadObservation::from_product(
                runtime,
                product.read_lease(),
            ),
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn from_stable<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        stable: &super::super::output_lineage::PublishedStableLineage,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        producer_identity: &str,
        output_family_identity: &str,
    ) -> Arc<Self>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        Arc::new(Self {
            posture: WorthQueryOutputSettlementPosture::StableReused,
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: producer_identity.to_owned(),
            output_family_identity: output_family_identity.to_owned(),
            receipt: None,
            stable: Some(stable.clone()),
            output_correspondence: Arc::clone(stable.output_correspondence()),
            restored_source: None,
            readiness_delivery: Some(
                WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
            ),
            producer_contacts_in_this_demand: 0,
            observation: WorthQueryApplicationReadObservation::from_product(
                runtime,
                selected.product().read_lease(),
            ),
        })
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
    let original_publication_is_current = committed.is_selected_at(observation);
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
        if settlement.posture == WorthQueryOutputSettlementPosture::RecoveredStableReused {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "restored stable output requires native re-evidence",
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
            None => match settlement.stable.as_ref() {
                Some(stable) => lineage.source_facts_for_stable_output(
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    selected.product().observation(),
                    stable,
                    usize::MAX,
                ),
                None => lineage.source_facts_for_restored_output(
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    selected.product().observation(),
                    settlement,
                    usize::MAX,
                ),
            },
        };
        facts
            .map_err(|()| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
                    "settled output currentness exceeded its lineage work budget",
                )
            })?
            .map(|read| read.facts)
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "settled output no longer names the current output lineage",
                )
            })
    }
}

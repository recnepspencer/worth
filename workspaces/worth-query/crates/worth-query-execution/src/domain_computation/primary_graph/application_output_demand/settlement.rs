mod authority;
pub(crate) mod producer_identity;
mod route;
use producer_identity::SettlementProducerIdentity;
mod current_commit;
use current_commit::reacquire_current_committed_output;
mod current_accepted;
pub(crate) mod observation;
pub(crate) mod producing_publication;
pub(in crate::domain_computation::primary_graph) use producing_publication::OriginalProducingPublication;
pub(crate) mod receipt_custody;
mod reused;
use receipt_custody::SettlementReceiptCustody;
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::ReadyCompletion;
use super::WorthQueryAcceptedOutputAuthority;
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::output_lineage::BoundCurrentAcceptedOutput;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputCorrespondence;
use crate::domain_computation::primary_graph::WorthQueryApplicationOutputProjectionDenial;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;
use crate::domain_computation::primary_graph::WorthQueryApplicationTypedOutputCorrespondence;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;

use super::WorthQueryOutputReadinessDeliveryEvidence;

/// Owner-retained result of one settled output demand.
///
/// The retained World observation is the authority for exact reads while a
/// settlement is held. After compaction, the receipt identifies the same
/// committed lineage so the owner can reacquire its still-current occurrence.
pub struct WorthQueryOutputDemandSettlement {
    _original_delivery_memory: Option<worth_execution::ExecutionMemoryReservation>,
    posture: WorthQueryOutputSettlementPosture,
    runtime_authority: u64,
    schema_binding: ApplicationSchemaBindingIdentity,
    producer_identity: SettlementProducerIdentity,
    output_family_identity: String,
    receipt: Option<SettlementReceiptCustody>,
    pub stable: Option<super::super::output_lineage::PublishedStableLineage>,
    pub output_correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub restored_source: Option<super::WorthQueryRestoredAcceptedOutput>,
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
    HistoricallySelected,
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

    pub(in crate::domain_computation::primary_graph) fn from_committed_ready<Schema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        producer_identity: &str,
        output_family_identity: &str,
        producer_contacts_in_this_demand: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
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
        Ok(Arc::new(Self {
            _original_delivery_memory: None,
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
                observation::OriginalSettledProducerObservation::ready(
                    execution,
                    runtime,
                    completion,
                    &observation,
                )?,
                observation,
            ),
        }))
    }

    /// A second immutable delivery header is retained in addition to final
    /// Ready. Fund that actual extra allocation for its entire shared lifetime.
    pub(in crate::domain_computation::primary_graph) fn from_original_program_ready<Schema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        completion: &ReadyCompletion,
        producer: &str,
        family: &str,
        contacts: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
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
            ._original_delivery_memory = memory;
        Ok(original)
    }

    pub(in crate::domain_computation::primary_graph) fn publication_view(
        &self,
    ) -> Option<super::OutputPublicationView<'_>> {
        self.receipt
            .as_ref()
            .and_then(SettlementReceiptCustody::publication_view)
    }
    pub fn application_commit_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        self.receipt
            .as_ref()
            .and_then(SettlementReceiptCustody::as_receipt)
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

    /// Retired descendant witnesses used only to admit this performed succession.
    /// They are not canonical roles returned by `outputs_of`.
    pub fn closure_evidence_of<Contract: 'static>(
        &self,
    ) -> Result<
        super::super::WorthQueryApplicationTypedClosureEvidence<'_, Contract>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        self.output_correspondence
            .outputs_of::<Contract>()
            .map(|roles| roles.closure_evidence())
    }

    pub fn producer_identity(&self) -> &str {
        self.producer_identity.as_str()
    }

    pub fn output_family_identity(&self) -> &str {
        &self.output_family_identity
    }

    pub fn readiness_delivery(&self) -> Option<&WorthQueryOutputReadinessDeliveryEvidence> {
        self.readiness_delivery.as_ref()
    }

    /// Executions of this demand's own producer over the handle's lifetime,
    /// including an execution cancelled after entering its handler. Advancing
    /// a settled handle reports the same lifetime count. An upstream execution
    /// run by another caller's advance is counted by no demand handle.
    pub const fn producer_contacts_in_this_demand(&self) -> usize {
        self.producer_contacts_in_this_demand
    }

    #[doc(hidden)]
    pub fn retained_read(&self) -> Arc<WorthQueryApplicationReadObservation> {
        Arc::clone(&self.observation)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub fn select_output_demand_settlement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        self.select_application_read_observation_with_request(execution, &settlement.observation)
            .map_err(|error| match error {
                stopped @ crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(_) => {
                    WorthQueryOutputDemandDenial::product_selection(
                        stopped,
                        "retained output basis",
                    )
                }
                error @ crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                        format!("retained output basis unavailable: {error:?}"),
                    )
                }
            })
    }

    pub(in crate::domain_computation::primary_graph) fn current_output_source_facts(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<
        crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
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
        let selected = match settlement.publication_view() {
            Some(receipt) => self
                .on_branch(receipt.product_branch())
                .select_with_request(execution)
                .map_err(|error| {
                    WorthQueryOutputDemandDenial::product_selection(
                        error,
                        "settled output currentness could not select its product occurrence",
                    )
                })?,
            None => self.select_output_demand_settlement(execution, settlement)?,
        };
        let lineage = self
            .runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let facts = match settlement.publication_view() {
            Some(receipt) => lineage.source_facts_for_publication(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                selected.product().observation(),
                receipt,
                execution,
            ),
            None => match settlement.stable.as_ref() {
                Some(stable) => lineage.source_facts_for_stable_output(
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    selected.product().observation(),
                    stable,
                    execution,
                ),
                None => lineage.source_facts_for_restored_output(
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    selected.product().observation(),
                    settlement,
                    execution,
                ),
            },
        };
        facts
            .map_err(|stop| {
                crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                    crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
                    "settled output lineage read unavailable",
                )
            })?
            .and_then(|read| read.facts)
            .filter(|facts| !facts.is_empty())
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "settled output no longer names the current output lineage",
                )
            })
    }
}

mod court_doors_for_test;

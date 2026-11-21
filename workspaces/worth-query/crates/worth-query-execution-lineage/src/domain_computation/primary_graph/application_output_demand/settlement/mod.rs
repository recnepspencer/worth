//! Owner-retained result of one settled output demand (pure core + accessors).

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use super::{OutputPublicationView, ReadyCompletion, WorthQueryAcceptedOutputAuthority, WorthQueryOutputReadinessDeliveryEvidence, WorthQueryRestoredAcceptedOutput};
use crate::domain_computation::primary_graph::{
    output_lineage::PublishedStableLineage,
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationOutputCorrespondence,
    WorthQueryApplicationOutputProjectionDenial, WorthQueryApplicationReadObservation,
    WorthQueryApplicationTypedClosureEvidence, WorthQueryApplicationTypedOutputCorrespondence,
};

pub mod authority;
pub mod observation;
pub mod posture;
pub mod producer_identity;
pub mod producing_publication;
pub mod receipt_custody;
pub mod route;

pub use observation::{OriginalSettledProducerObservation, OriginalSettledProducerSource};
pub use posture::WorthQueryOutputSettlementPosture;
pub use producer_identity::SettlementProducerIdentity;
pub use producing_publication::OriginalProducingPublication;
pub use receipt_custody::SettlementReceiptCustody;

pub struct WorthQueryOutputDemandSettlement {
    pub _original_delivery_memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub posture: WorthQueryOutputSettlementPosture,
    pub runtime_authority: u64,
    pub schema_binding: ApplicationSchemaBindingIdentity,
    pub producer_identity: SettlementProducerIdentity,
    pub output_family_identity: String,
    pub receipt: Option<SettlementReceiptCustody>,
    pub stable: Option<PublishedStableLineage>,
    pub output_correspondence: Arc<WorthQueryApplicationOutputCorrespondence>,
    pub restored_source: Option<WorthQueryRestoredAcceptedOutput>,
    pub readiness_delivery: Option<WorthQueryOutputReadinessDeliveryEvidence>,
    pub producer_contacts_in_this_demand: usize,
    pub observation: Arc<WorthQueryApplicationReadObservation>,
}

impl WorthQueryOutputDemandSettlement {
    pub fn matches_runtime_owner(
        &self,
        runtime_authority: u64,
        schema_binding: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime_authority == runtime_authority && &self.schema_binding == schema_binding
    }

    pub fn publication_view(&self) -> Option<OutputPublicationView<'_>> {
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

    pub fn outputs_of<Contract: 'static>(
        &self,
    ) -> Result<
        WorthQueryApplicationTypedOutputCorrespondence<'_, Contract>,
        WorthQueryApplicationOutputProjectionDenial,
    > {
        self.output_correspondence.outputs_of()
    }

    pub fn closure_evidence_of<Contract: 'static>(
        &self,
    ) -> Result<
        WorthQueryApplicationTypedClosureEvidence<'_, Contract>,
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

    pub const fn producer_contacts_in_this_demand(&self) -> usize {
        self.producer_contacts_in_this_demand
    }

    #[doc(hidden)]
    pub fn retained_read(&self) -> Arc<WorthQueryApplicationReadObservation> {
        Arc::clone(&self.observation)
    }
}

mod court_doors_for_test;

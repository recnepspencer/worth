use std::sync::Arc;

use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryObservedSource, WorthQueryOutputDemandSettlement,
};

pub struct WorthQueryApplicationOutputDemandSettlement<Query> {
    retained: Arc<WorthQueryOutputDemandSettlement>,
    observation: super::super::WorthQueryApplicationReadObservation,
    source: WorthQueryObservedSource<Query>,
}

impl<Query> WorthQueryApplicationOutputDemandSettlement<Query> {
    pub(in crate::application_entry) fn new(
        retained: Arc<WorthQueryOutputDemandSettlement>,
        source: WorthQueryObservedSource<Query>,
    ) -> Self {
        let observation =
            super::super::WorthQueryApplicationReadObservation::new(retained.retained_read());
        Self {
            retained,
            observation,
            source,
        }
    }

    pub fn application_commit_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        self.retained.application_commit_receipt()
    }

    pub fn output_correspondence(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryApplicationOutputCorrespondence
    {
        self.retained.output_correspondence()
    }

    pub(in crate::application_entry) fn retained(&self) -> &WorthQueryOutputDemandSettlement {
        self.retained.as_ref()
    }

    pub const fn observed_source(&self) -> &WorthQueryObservedSource<Query> {
        &self.source
    }

    pub const fn observation(&self) -> &super::super::WorthQueryApplicationReadObservation {
        &self.observation
    }

    pub fn readiness_delivery(
        &self,
    ) -> Option<
        &worth_query_execution::facade::primary_graph::WorthQueryOutputReadinessDeliveryEvidence,
    > {
        self.retained.readiness_delivery()
    }

    /// Producer executions initiated by this demand, not by an earlier output.
    pub fn producer_contacts_in_this_demand(&self) -> usize {
        self.retained.producer_contacts_in_this_demand()
    }

    pub(in crate::application_entry) fn retained_settlement(
        &self,
    ) -> &WorthQueryOutputDemandSettlement {
        self.retained.as_ref()
    }

    pub(in crate::application_entry) fn retain_settlement(
        &self,
    ) -> Arc<WorthQueryOutputDemandSettlement> {
        Arc::clone(&self.retained)
    }
}

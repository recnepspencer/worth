use worth_relational::facade::history::CommitId;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch;
#[cfg(test)]
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationEmission;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn admit_application_commit_causality(
        &self,
        commit_id: CommitId,
    ) -> Result<(), &'static str> {
        self.live_delivery.admit_publication(commit_id)
    }

    pub(in crate::domain_computation::primary_graph) fn publish_application_commit_causality(
        &self,
        commit_id: CommitId,
        emissions: WorthQueryAdmittedApplicationEmissionBatch,
    ) -> Result<usize, &'static str> {
        self.live_delivery.publish(commit_id, emissions)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn committed_application_emissions(
        &self,
        commit_id: CommitId,
    ) -> Vec<WorthQueryApplicationEmission> {
        self.live_delivery.emissions(commit_id)
    }

    #[cfg(test)]
    pub(crate) fn published_application_commit_count(&self) -> usize {
        self.live_delivery.published_commit_count()
    }

    #[cfg(test)]
    pub(crate) fn retained_application_emission_bytes(&self) -> u64 {
        self.live_delivery.retained_payload_bytes()
    }
}

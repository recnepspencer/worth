//! The runtime's consumer methods over `retained_read` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn select_application_read_observation(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::RetainedReadOps<
            Schema,
        >>::select_application_read_observation(self, phase, observation)
    }
}

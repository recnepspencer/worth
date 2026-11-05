//! The runtime's consumer methods over `batch` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::batch::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchResourceDenial;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

impl<Schema: ApplicationSchema>
    super::super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    pub fn plan_application_query_batch_reads<B: ApplicationQueryBinding<Schema>>(
        &self,
        binding: &WorthQueryInstalledApplicationQueryBinding<Schema, B>,
        count: usize,
    ) -> Result<
        WorthQueryApplicationQueryBatchReadPlan,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::plan_application_query_batch_reads(self, binding, count)
    }

    pub fn plan_installed_query_batch_reads<Q, P, R, S>(
        &self,
        query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
            Schema,
            Q,
            P,
            R,
            S,
        >,
        count: usize,
        maximum_results: std::num::NonZeroUsize,
    ) -> Result<
        WorthQueryApplicationQueryBatchReadPlan,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::plan_installed_query_batch_reads(self, query, count, maximum_results)
    }

    pub fn application_query_batch_from_installed_reads(
        &self,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<
        WorthQueryApplicationQueryBatchAdmission,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::application_query_batch_from_installed_reads(self, reads)
    }

    pub fn extend_application_query_batch_once(
        &self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::extend_application_query_batch_once(self, batch, reads)
    }
}

//! Sealed test doors over this module's private members.
use super::*;

pub trait SchedulingProgressionDoorsOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static, {
#[doc(hidden)]
    fn schedule_selected_output_producer_for_test<Query>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedApplicationProducer,
        branch: crate::basis::WorthQueryProductBranch,
        observed_source: &WorthQueryObservedSource<Query>,
        performed_source: Option<
            &crate::domain_computation::primary_graph::application_output_demand::WorthQueryPerformedOutputDemandSource,
        >,
    ) -> Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial> {
        Self::schedule_selected_output_producer::<Query>(
            self,
            execution,
            phase,
            selected,
            branch,
            observed_source,
            performed_source,
        )
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> SchedulingProgressionDoorsOps<Schema> for HostRuntime where
    Schema: worth_query_installation::facade::ApplicationSchema + 'static, {}

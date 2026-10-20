use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission,
    output_reuse::{
        compare_retained_output_dependencies, compare_retained_output_witness,
        require_installed_output_dependencies, retained_output_settlement_is_verified,
        OutputDependencySelection,
    },
    WorthQueryApplicationBasisSelectionIdentity, WorthQueryPrimaryGraphApplicationRuntime,
};

type SelectedWithEntry<'a, Schema> = (
    WorthQuerySelectedApplicationProducer,
    &'a std::sync::Arc<super::super::registry::InstalledProducerProvider<Schema>>,
);

mod native_selection;
mod recovered_candidates;
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) mod retained_liveness;
mod selected_basis;
pub(super) use selected_basis::source_basis_is_admitted;
use selected_basis::{charge, selection_budget_denial};

#[cfg(test)]
#[path = "selection/retained_basis_tests.rs"]
mod retained_basis_tests;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn select_output_producer<Family>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
        maximum_work: Option<std::num::NonZeroUsize>,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| WorthQueryOutputDemandDenial::request_admission(cause.into()))?;
        let limits = self.output_demand_resource_profile().limits();
        let maximum_work = maximum_work
            .map(std::num::NonZeroUsize::get)
            .into_iter()
            .chain(limits.source_currentness_work())
            .min();
        let mut admission = self
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner
            .edit_admission_with_work_limit(maximum_work);
        self.select_output_producer_with_remaining::<Family>(
            &execution,
            source,
            profile_kind,
            &mut admission,
            None,
        )
        .map(|(selected, _)| selected)
    }

    /// The admitted demand continues on this request's meter.
    pub(super) fn select_output_producer_with_remaining<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
        admission: &mut InvalidationEditAdmission,
        retained_program_basis: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_remaining_core::<Family>(
            execution,
            source,
            profile_kind,
            admission,
            retained_program_basis,
            None,
            &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire(),
        )
    }
}

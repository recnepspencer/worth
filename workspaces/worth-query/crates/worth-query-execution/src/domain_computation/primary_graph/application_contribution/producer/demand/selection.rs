use super::*;
use crate::domain_computation::primary_graph::output_reuse::compare_retained_output_dependencies;
use crate::domain_computation::primary_graph::output_reuse::compare_retained_output_witness;
use crate::domain_computation::primary_graph::output_reuse::require_installed_output_dependencies;
use crate::domain_computation::primary_graph::output_reuse::retained_output_settlement_is_verified;
use crate::domain_computation::primary_graph::output_reuse::OutputDependencySelection;
use crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

type SelectedWithEntry<'a, Schema> = (
    WorthQuerySelectedApplicationProducer,
    &'a std::sync::Arc<super::super::registry::InstalledProducerProvider<Schema>>,
);

mod native_selection;
mod recovered_candidates;
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) mod retained_liveness;
mod selected_basis;

pub(super) use selected_basis::source_basis_is_admitted;

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
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let execution = phase
            .execution_request_for(&self.product_runtime)
            .map_err(|cause| WorthQueryOutputDemandDenial::request_admission(cause.into()))?;
        self.select_output_producer_with_remaining::<Family>(&execution, source, profile_kind, None)
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

        retained_program_basis: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_remaining_core::<Family>(execution, source, profile_kind, retained_program_basis, None, &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }
}

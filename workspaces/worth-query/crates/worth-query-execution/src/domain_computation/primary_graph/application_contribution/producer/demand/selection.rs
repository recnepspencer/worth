use crate::domain_computation::primary_graph::application_contribution::producer::demand::selection::native_selection::NativeSelectionParentOps;
use super::*;
use crate::domain_computation::primary_graph::output_reuse::compare_retained_output_dependencies;
use crate::domain_computation::primary_graph::output_reuse::compare_retained_output_witness;
use crate::domain_computation::primary_graph::output_reuse::require_installed_output_dependencies;
use crate::domain_computation::primary_graph::output_reuse::retained_output_settlement_is_verified;
use crate::domain_computation::primary_graph::output_reuse::OutputDependencySelection;
use crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use {
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind,
    crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandRecoveryPosture,
};

type SelectedWithEntry<'a, Schema> = (
    WorthQuerySelectedApplicationProducer,
    &'a std::sync::Arc<super::super::registry::InstalledProducerProvider<Schema>>,
);

// Lifecycle inspection selects the producer for live output. Demand admission
// may instead rejoin an already committed Ready, which performs no producer run.
enum SelectionPurpose {
    Lifecycle,
    OutputDelivery,
}

mod native_selection;
mod recovered_candidates;
pub(crate) mod retained_liveness;
pub(crate) mod selected_basis;

pub(super) use selected_basis::source_basis_is_admitted;

pub trait SelectionOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn select_output_producer<Family>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let execution = phase
            .execution_request_for(self.product_runtime())
            .map_err(|cause| WorthQueryOutputDemandDenial::request_admission(cause.into()))?;
        self.select_output_producer_with_remaining_core::<Family>(
            &execution,
            source,
            profile_kind,
            None,
            None,
            SelectionPurpose::Lifecycle,
            &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire(),
        )
        .map(|(selected, _)| selected)
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> SelectionOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}

pub(super) trait SelectionParentOps<Schema>: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> where
    Schema: ApplicationSchema + 'static, {
fn select_output_producer_with_remaining<Family>(
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
        self.select_output_producer_with_remaining_core::<Family>(execution, source, profile_kind, retained_program_basis, None, SelectionPurpose::OutputDelivery, &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }
}

impl<Schema, HostRuntime: crate::domain_computation::primary_graph::application_contribution::host::ApplicationContributionHost<Schema> + ?Sized> SelectionParentOps<Schema> for HostRuntime where
    Schema: ApplicationSchema + 'static, {}
pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) use selected_basis::SelectedBasisScopedOps;
pub(in crate::domain_computation::primary_graph::application_contribution::producer) use selected_basis::SelectionSelectedBasisScopedOps;

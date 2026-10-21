//! Exact selected Product checks for a fresh required-output continuation.

use super::*;
use crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation;
use crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// The required wave has already selected and retained this Product.
    /// Its fresh disclosed source must name that exact read; predecessor
    /// program custody may name an older head of the same occurrence.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn select_output_producer_with_remaining_on_selected<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,

        retained_program_basis: Option<&WorthQueryApplicationReadObservation>,
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_remaining_core::<Family>(execution, source, profile_kind, retained_program_basis, Some(shared), &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess::acquire())
    }
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn select_output_producer_for_observation<
        Family,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,

        shared: &SharedSelectedProductOperation<'_, Schema>,
        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_remaining_core::<Family>(
            execution,
            source,
            profile_kind,
            None,
            Some(shared),
            native,
        )
    }
}

/// Historical program custody constrains the application and occurrence. It
/// does not assert that its earlier commit equals the fresh selected head.
pub(super) fn historical_basis_matches<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryApplicationReadObservation,
    shared: &SharedSelectedProductOperation<'_, Schema>,
) -> Result<bool, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let selected = shared.selected();
    let selected_observation = selected.product().observation();
    Ok(std::ptr::eq(runtime, selected.application())
        && retained.belongs_to_selected_occurrence(runtime, selected_observation))
}

/// Check the selected read before candidate enumeration, including the empty
/// candidate path. A failed comparison never authorizes another Product read.
pub(super) fn require_fresh_source<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    source: &crate::basis::WorthQueryProductBranchReadIdentity,
    family: &str,
    shared: &SharedSelectedProductOperation<'_, Schema>,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    let selected = shared.selected();
    let selected_observation = selected.product().observation();
    if std::ptr::eq(runtime, selected.application())
        && source.matches_observation(selected_observation)
    {
        Ok(())
    } else {
        Err(WorthQueryOutputDemandDenial::new(
            WorthQueryOutputDemandDenialKind::Superseded,
            family.to_owned(),
        )
        .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable))
    }
}

pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn source_basis_is_admitted(
    source: &crate::basis::WorthQueryProductBranchReadIdentity,
    current: &worth_runtime_world::facade::ProductBranchObservation,
    owner_retained_program_basis: bool,
) -> bool {
    if owner_retained_program_basis {
        source.same_branch_occurrence_observation(current)
    } else {
        source.matches_observation(current)
    }
}

/// Candidate lookup uses the caller's selection admission. Its optional work
/// ceiling can be changed on a fresh selection; recorded-fact bounds and every
/// other native stop keep their own meaning.
pub(super) fn selection_lookup_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
    family: &str,
) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::output_reuse;
    use worth_relational::facade::mvcc::CompanionPreflightStop;
    match stop {
        CompanionPreflightStop::WorkExhausted { .. }
        | CompanionPreflightStop::WorkExhaustedWithoutRequirement { .. } => {
            selection_budget_denial(family)
        }
        other => output_reuse::retained_fact_denial(
            output_reuse::preflight_stop(other),
            "native retained output selection unavailable",
        ),
    }
}

pub(super) fn selection_budget_denial(family: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        family.to_owned(),
    )
    .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable)
}

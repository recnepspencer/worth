//! Exact selected Product checks for a fresh required-output continuation.

use super::*;
use crate::domain_computation::primary_graph::{
    product_operation::SharedSelectedProductOperation, WorthQueryApplicationReadObservation,
};

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
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
        remaining_work: &mut usize,
        retained_program_basis: Option<&WorthQueryApplicationReadObservation>,
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_remaining_core::<Family>(
            source,
            profile_kind,
            remaining_work,
            retained_program_basis,
            Some(shared),
        )
    }
}

/// Historical program custody constrains the application and occurrence. It
/// does not assert that its earlier commit equals the fresh selected head.
pub(super) fn historical_basis_matches<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &WorthQueryApplicationReadObservation,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    remaining_work: &mut usize,
    family: &str,
) -> Result<bool, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    // Admit the owner and observation metadata visit before reading widths.
    *remaining_work = remaining_work
        .checked_sub(4)
        .ok_or_else(|| selection_budget_denial(family))?;
    let selected = shared.selected();
    let selected_observation = selected.product().observation();
    let selected_width = selected_observation.branch_identity().name().as_str().len();
    let retained_width = retained.branch_identity().name().as_str().len();
    let comparison = selected_width
        .checked_add(retained_width)
        // The schema binding compares its package and schema digests.
        .and_then(|work| work.checked_add(64 + 7))
        .ok_or_else(|| selection_budget_denial(family))?;
    *remaining_work = remaining_work
        .checked_sub(comparison)
        .ok_or_else(|| selection_budget_denial(family))?;
    Ok(std::ptr::eq(runtime, selected.application())
        && retained.belongs_to_selected_occurrence(runtime, selected_observation))
}

/// Check the selected read before candidate enumeration, including the empty
/// candidate path. A failed comparison never authorizes another Product read.
pub(super) fn require_fresh_source<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    source: &crate::basis::WorthQueryProductBranchReadIdentity,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    remaining_work: &mut usize,
    family: &str,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    // These metadata reads precede the variable-width branch comparison.
    *remaining_work = remaining_work
        .checked_sub(4)
        .ok_or_else(|| selection_budget_denial(family))?;
    let selected = shared.selected();
    let selected_observation = selected.product().observation();
    let source_width = source.branch_identity().name().as_str().len();
    let selected_width = selected_observation.branch_identity().name().as_str().len();
    let comparison = source_width
        .checked_add(selected_width)
        // Generation, commit, three composite basis identities, and owner
        // axes use fixed-width comparisons after the two branch texts.
        .and_then(|work| work.checked_add(160))
        .ok_or_else(|| selection_budget_denial(family))?;
    *remaining_work = remaining_work
        .checked_sub(comparison)
        .ok_or_else(|| selection_budget_denial(family))?;
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

pub(super) fn source_basis_is_admitted(
    source: &crate::basis::WorthQueryProductBranchReadIdentity,
    current: &crate::basis::WorthQueryProductBranchReadIdentity,
    owner_retained_program_basis: bool,
) -> bool {
    if owner_retained_program_basis {
        source.same_branch_occurrence(current)
    } else {
        source == current
    }
}

pub(super) fn selection_budget_denial(family: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        family.to_owned(),
    )
    .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable)
}

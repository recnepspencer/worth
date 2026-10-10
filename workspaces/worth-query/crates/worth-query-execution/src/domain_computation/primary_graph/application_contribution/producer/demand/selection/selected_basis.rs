//! Exact selected Product checks for a fresh required-output continuation.

use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission,
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
        admission: &mut InvalidationEditAdmission,
        retained_program_basis: Option<&WorthQueryApplicationReadObservation>,
        shared: &SharedSelectedProductOperation<'_, Schema>,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.select_output_producer_with_remaining_core::<Family>(
            source,
            profile_kind,
            admission,
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
    admission: &mut InvalidationEditAdmission,
    family: &str,
) -> Result<bool, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    // Admit the owner and observation metadata visit before reading widths.
    charge(admission, 4, family)?;
    let selected = shared.selected();
    let selected_observation = selected.product().observation();
    let selected_width = selected_observation.branch_identity().name().as_str().len();
    let retained_width = retained.branch_identity().name().as_str().len();
    let comparison = selected_width
        .checked_add(retained_width)
        // The schema binding compares its package and schema digests.
        .and_then(|work| work.checked_add(64 + 7))
        .ok_or_else(|| selection_budget_denial(family))?;
    charge(admission, comparison, family)?;
    Ok(std::ptr::eq(runtime, selected.application())
        && retained.belongs_to_selected_occurrence(runtime, selected_observation))
}

/// Check the selected read before candidate enumeration, including the empty
/// candidate path. A failed comparison never authorizes another Product read.
pub(super) fn require_fresh_source<Schema>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    source: &crate::basis::WorthQueryProductBranchReadIdentity,
    shared: &SharedSelectedProductOperation<'_, Schema>,
    admission: &mut InvalidationEditAdmission,
    family: &str,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    // These metadata reads precede the variable-width branch comparison.
    charge(admission, 4, family)?;
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
    charge(admission, comparison, family)?;
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

/// Selection work on the request's meter.
pub(super) fn charge(
    admission: &mut InvalidationEditAdmission,
    work: usize,
    family: &str,
) -> Result<(), WorthQueryOutputDemandDenial> {
    admission
        .charge_external_work(work as u64)
        .map_err(|_| selection_budget_denial(family))
}

pub(super) fn selection_budget_denial(family: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        family.to_owned(),
    )
    .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable)
}

//! Typed entry points into the shared validated progression owner.

use super::*;

pub(super) fn validate_progression_resources<Schema>(
    retained: &mut Option<crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerDemandResources>,
    validated: &mut bool,
    disclosed_value: &dyn std::any::Any,
    entry: &InstalledProducerProvider<Schema>,
    producer_identity: &str,
    limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    if *validated {
        return Ok(());
    }
    let resources = resources::validate_demand_resources(
        entry.executor.as_ref(),
        disclosed_value,
        producer_identity,
        limits,
    )?;
    *retained = Some(resources);
    *validated = true;
    Ok(())
}

/// Copy the actual executed mode into the Ready backing. A required successor
/// already paid its separate admission copy before this effect boundary.
pub(super) fn clone_published_mode(
    registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
    interest: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest,
    mode: &WorthQueryProducerCommitAuthority,
    selected: bool,
    producer_identity: &str,
    admission: &mut InvalidationEditAdmission,
) -> Result<WorthQueryProducerCommitAuthority, WorthQueryOutputDemandDenial> {
    let copy = if selected {
        std::mem::size_of::<WorthQueryProducerCommitAuthority>()
    } else {
        0
    };
    let work = u64::try_from(copy)
        .ok()
        .and_then(|copy| copy.checked_add(3));
    if work.is_none_or(|work| admission.charge_external_work(work).is_err()) {
        registry.relinquish_execution(interest);
        return Err(denial(
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
            producer_identity.to_owned(),
        ));
    }
    Ok(mode.clone())
}

pub(super) fn validate_resources_after_begin<Schema>(
    registry: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry,
    interest: &crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandInterest,
    retained: &mut Option<crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryProducerDemandResources>,
    validated: &mut bool,
    disclosed_value: &dyn std::any::Any,
    entry: &InstalledProducerProvider<Schema>,
    producer_identity: &str,
    limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
) -> Result<(), WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
{
    if let Err(denial) = validate_progression_resources(
        retained,
        validated,
        disclosed_value,
        entry,
        producer_identity,
        limits,
    ) {
        registry.relinquish_execution(interest);
        return Err(denial);
    }
    Ok(())
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn advance_validated_output_demand<
        Family,
    >(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        entry: &InstalledProducerProvider<Schema>,
        commit_authority: WorthQueryProducerCommitAuthority,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        self.advance_validated_output_demand_with_schedule(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority,
            ScheduleProgression::ReturnPending,
            request_admission,
        )
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn advance_validated_required_fresh_on_selected<
        Family,
    >(
        &self,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        entry: &InstalledProducerProvider<Schema>,
        commit_authority: WorthQueryProducerCommitAuthority,
        shared: &crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation<'_, Schema>,
        matched_predecessor: Option<MatchedRequiredPredecessor<'_>>,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        self.advance_validated_output_demand_with_schedule(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority,
            ScheduleProgression::ContinueScheduled {
                shared,
                matched_predecessor,
            },
            request_admission,
        )
    }
}

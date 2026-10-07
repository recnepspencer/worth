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
    ) -> Result<CallerPass, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandAdvanceAdmission as Admission;
        const WAITING: CallerPass = CallerPass::Answer(WorthQueryOutputDemandAdvance::Pending);
        let wave_authority = commit_authority.clone();
        match self.advance_validated_output_demand_with_schedule(
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority,
            ScheduleProgression::Ordinary,
            request_admission,
        )? {
            OwnStages::Answer(advance) => return Ok(CallerPass::Answer(advance)),
            OwnStages::Refreshed => return Ok(CallerPass::RetryDisclosure),
            OwnStages::Checkpoint => {}
        }
        // The checkpoint this call published or moved is its own to finish.
        // A stage that did not move waits on a delivery outside this call.
        let completion = loop {
            let interest = demand
                .interest
                .as_ref()
                .expect("a published row retains its live Interest");
            match self.output_demands.begin_published(interest) {
                Admission::AdvanceCheckpoint { claim, checkpoint } => {
                    if !self.advance_output_checkpoint(
                        interest,
                        &demand.selected.identity,
                        claim,
                        checkpoint,
                    )? {
                        return Ok(WAITING);
                    }
                }
                Admission::Ready(completion) => break completion,
                Admission::Failed(denial) => return Err(denial),
                _ => return Ok(WAITING),
            }
        };
        // The Ready this call reached is certified as any Ready is: by the
        // required wave when it has one, and otherwise on its own proof.
        if let Some(advance) = super::super::required_wave::advance_required_before_caller(
            self,
            demand,
            principal,
            request_scope,
            delivery_branch,
            &wave_authority,
            request_admission,
        )? {
            return Ok(CallerPass::Answer(advance));
        }
        self.settle_own_ready(demand, &completion, delivery_branch)
            .map(CallerPass::Answer)
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
        matched_predecessors: Option<MatchedRequiredPredecessors<'_>>,
        request_admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        // The wave rejoins the successor this call published and drives its
        // checkpoint itself.
        Ok(
            match self.advance_validated_output_demand_with_schedule(
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
                entry,
                commit_authority,
                ScheduleProgression::Selected {
                    shared,
                    matched_predecessors,
                },
                request_admission,
            )? {
                OwnStages::Answer(advance) => advance,
                // A selected pass never refreshes: its Ready waits on the
                // wave that selected it.
                OwnStages::Checkpoint | OwnStages::Refreshed => {
                    WorthQueryOutputDemandAdvance::Pending
                }
            },
        )
    }
}

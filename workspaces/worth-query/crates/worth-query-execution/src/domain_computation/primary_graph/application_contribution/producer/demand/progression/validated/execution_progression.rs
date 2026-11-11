//! Admit or execute one typed producer progression at a time.
use super::*;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand::progression) fn prepare_validated_output_demand_with_schedule<
        'owner,
        Family,
    >(
        &'owner self,
        phase: &'owner WorthQueryAdvancementPhase<'owner>,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: ValidatedOutputDisclosure<
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        entry: &InstalledProducerProvider<Schema>,
        commit_authority: ProducerInvocationAuthority<'owner>,
        schedule_progression: ScheduleProgression<'owner, '_, '_, Schema>,

        performed: &'owner std::cell::RefCell<PerformedMembers>,
    ) -> Result<
        ExecutionPreparation<
            'owner,
            FamilySourceQuery<Schema, Family>,
            FamilySourceValue<Schema, Family>,
        >,
        WorthQueryOutputDemandDenial,
    >
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        FamilySourceValue<Schema, Family>: 'static,
        FamilySourceQuery<Schema, Family>: 'static,
    {
        let prepared = self.prepare_running_output_with_schedule(
            phase,
            demand,
            principal,
            request_scope,
            delivery_branch,
            disclosure,
            entry,
            commit_authority,
            &schedule_progression,
            performed,
        )?;
        match prepared {
            OutputPreparation::Reached(stage) => Ok(ExecutionPreparation::Reached(stage)),
            OutputPreparation::Running(running) => self.execute_running_output(
                phase,
                demand,
                principal,
                request_scope,
                delivery_branch,
                entry,
                schedule_progression,
                performed,
                running,
            ),
        }
    }
}

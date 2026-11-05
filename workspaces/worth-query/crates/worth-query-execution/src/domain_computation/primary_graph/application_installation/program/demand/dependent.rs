use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;
mod parent;
mod source_basis;

impl<Schema, Program> WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn admit_program_dependent_output<ParentDemand, Connection>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        parent: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        basis: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        minimum_observation: &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, ConnectionDemand<Schema, Connection>>,
            SourceValue<Schema, ConnectionDemand<Schema, Connection>>,
        >,
        limits: crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits,
    ) -> Result<
        WorthQueryAdmittedProgramOutput<Schema, Program, ConnectionDemand<Schema, Connection>>,
        WorthQueryOutputDemandDenial,
    >
    where
        ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
        Connection: ApplicationConnectionShape<Schema>,
        ConnectionBinding<Schema, Connection>:
            WorthQueryApplicationDependentOutputConnection<Schema, RootDemand = ParentDemand>,
    {
        let phase_request = match phase.execution_request_for(&self.backing.runtime.runtime.product_runtime) {
            Ok(request) => request,
            Err(cause) => return Err(crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial::new(crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()), "output-demand-phase")),
        };
        let execution = &phase_request;

        let artifact = self.validate_dependent_output_parent::<ParentDemand, Connection>(parent)?;
        let limits = self
            .backing
            .runtime
            .output_demand_resource_profile()
            .constrain(limits);
        let selected_source = source
            .observed_sources()
            .first()
            .filter(|_| source.rows().len() == 1 && source.observed_sources().len() == 1);
        let parent_occurrence = parent.retained.retained_read().branch_incarnation();
        let retained_basis =
            self.select_dependent_output_basis(execution, parent, basis, minimum_observation)?;
        if selected_source.is_none_or(|observed| {
            observed.selected_product_commit() != Some(basis.selected_commit())
                || observed.selected_product_occurrence() != Some(parent_occurrence)
        }) {
            return Err(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "dependent output source was not discovered at its Query-owned continuation observation",
            ));
        }
        self.backing
            .runtime
            .admit_dependent_output_demand::<Family<Schema, ConnectionDemand<Schema, Connection>>>(
                execution,
                source,
                limits,
                retained_basis.retain_application_read(),
            )
            .map(|admitted| WorthQueryAdmittedProgramOutput {
                admitted,
                original_root_completion: None,
                target_feature: TypeId::of::<Connection::TargetFeature>(),
                artifact,
                marker: std::marker::PhantomData,
            })
    }
}

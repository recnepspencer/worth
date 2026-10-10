use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::{
    WorthQueryAdmittedProgramOutput, WorthQueryProgramApplicationRuntime,
    WorthQueryProgramOutputAdvance, WorthQuerySettledProgramOutput,
};
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;

use super::{WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandSettlement};

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> =
    <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub(in crate::application_entry) enum WorthQueryApplicationProgramDemandProgress<
    Schema,
    Program,
    Demand,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    Pending,
    Settled {
        settlement: WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, Demand>>,
        authority: WorthQuerySettledProgramOutput<Schema, Program, Demand>,
    },
}

pub(in crate::application_entry) struct WorthQueryApplicationProgramDemandHandle<
    Schema,
    Program,
    Demand,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    admitted: WorthQueryAdmittedProgramOutput<Schema, Program, Demand>,
    demand: Demand,
    source_observation: Option<
        std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
        >,
    >,
}

impl<Schema, Program, Demand> WorthQueryApplicationProgramDemandHandle<Schema, Program, Demand>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, Demand>>,
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(in crate::application_entry) fn new(
        admitted: WorthQueryAdmittedProgramOutput<Schema, Program, Demand>,
        demand: Demand,
        source_observation: Option<
            std::sync::Arc<
                worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
            >,
        >,
    ) -> Self {
        Self {
            admitted,
            demand,
            source_observation,
        }
    }

    pub(in crate::application_entry) fn notifications(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications,
        WorthQueryApplicationOutputDemandDenial,
    > {
        self.admitted
            .notifications()
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)
    }

    fn disclose(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
        WorthQueryApplicationOutputDemandDenial,
    > {
        Ok(if let Some(observation) = &self.source_observation {
            request
                .at(
                    &crate::application_entry::WorthQueryApplicationReadObservation::new(
                        std::sync::Arc::clone(observation),
                    ),
                )
                .query(self.demand.source_intent())
                .execute_in_advancement(phase)
        } else {
            request
                .query(self.demand.source_intent())
                .execute_in_advancement(phase)
        }
        .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
        .into_output_demand_source())
    }

    pub(in crate::application_entry) fn finish_unavailable<Root>(
        &mut self, phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>, application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        prepared: &worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
    ) -> Result<worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial, WorthQueryApplicationOutputDemandDenial>
    where Root: worth_query_declaration::facade::application_program::ApplicationOutputGraphShape<Schema, Dependents = worth_query_declaration::facade::application_program::ApplicationOutputLeaf>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        <<Root as worth_query_declaration::facade::application_program::ApplicationOutputGraphShape<Schema>>::RootConnection as worth_query_declaration::facade::application_program::ApplicationConnectionShape<Schema>>::Binding:
    worth_query_execution::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>{
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let disclosure = self.disclose(phase, request)?;
        application
            .finish_unavailable_program_output::<Root>(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                &mut self.admitted,
                prepared,
                request.principal,
                request.scope,
                request.branch,
                disclosure,
            )
            .map_err(map_progress_denial)
    }

    pub(in crate::application_entry) fn advance(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationProgramDemandProgress<Schema, Program, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let disclosure = self.disclose(phase, request)?;
        match application
            .advance_program_output(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                &mut self.admitted,
                request.principal,
                request.scope,
                request.branch,
                disclosure,
            )
            .map_err(map_progress_denial)?
        {
            WorthQueryProgramOutputAdvance::Pending => {
                Ok(WorthQueryApplicationProgramDemandProgress::Pending)
            }
            WorthQueryProgramOutputAdvance::Settled(authority) => {
                let settlement = WorthQueryApplicationOutputDemandSettlement::new(
                    authority.retained(),
                    self.admitted.observed_source().clone(),
                    self.admitted.checkpoint_readmission_work_units(),
                    self.admitted.checkpoint_readmission_work_bound(),
                    self.admitted
                        .checkpoint_readmission_charged_preparation_bytes(),
                );
                Ok(WorthQueryApplicationProgramDemandProgress::Settled {
                    settlement,
                    authority,
                })
            }
        }
    }
}

fn map_progress_denial(
    denial: worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial,
) -> WorthQueryApplicationOutputDemandDenial {
    if denial.kind()
        == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded
    {
        WorthQueryApplicationOutputDemandDenial::Superseded
    } else {
        WorthQueryApplicationOutputDemandDenial::Demand(denial)
    }
}

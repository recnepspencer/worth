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
    'application,
    Schema,
    Program,
    Demand,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    admitted: WorthQueryAdmittedProgramOutput<Schema, Program, Demand>,
    demand: Demand,
}

impl<'application, Schema, Program, Demand>
    WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>
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
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        admitted: WorthQueryAdmittedProgramOutput<Schema, Program, Demand>,
        demand: Demand,
    ) -> Self {
        Self {
            application,
            admitted,
            demand,
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

    pub(in crate::application_entry) fn advance(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationProgramDemandProgress<Schema, Program, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let disclosure = request
            .query(self.demand.source_intent())
            .execute()
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
            .into_output_demand_disclosure();
        match self
            .application
            .advance_program_output(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &self.admitted,
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

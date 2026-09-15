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
    WorthQueryAdmittedProgramDependentOutput, WorthQueryAdmittedProgramRootOutput,
    WorthQueryProgramApplicationRuntime, WorthQueryProgramRootOutputAdvance,
    WorthQuerySettledProgramRootOutput,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryOutputDemandAdvance,
};

use super::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandProgress,
    WorthQueryApplicationOutputDemandSettlement,
};

type RootDemand<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::Connections as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type DependentDemand<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::DependentConnection as WorthQueryApplicationDependentOutputConnection<Schema>>::Demand;
type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> =
    <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub(in crate::application_entry) enum WorthQueryApplicationProgramRootDemandProgress<
    Schema,
    Program,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    Pending,
    Settled {
        settlement: WorthQueryApplicationOutputDemandSettlement<
            SourceQuery<Schema, RootDemand<Schema, Program>>,
        >,
        authority: WorthQuerySettledProgramRootOutput<Schema, Program>,
    },
}

pub(in crate::application_entry) struct WorthQueryApplicationProgramRootDemandHandle<
    'application,
    Schema,
    Program,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    admitted: WorthQueryAdmittedProgramRootOutput<Schema, Program>,
    demand: RootDemand<Schema, Program>,
}

pub(in crate::application_entry) struct WorthQueryApplicationProgramDependentDemandHandle<
    'application,
    Schema,
    Program,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    admitted: WorthQueryAdmittedProgramDependentOutput<Schema, Program>,
    demand: DependentDemand<Schema, Program>,
}

impl<'application, Schema, Program>
    WorthQueryApplicationProgramRootDemandHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<
        Schema,
        RootDemand = RootDemand<Schema, Program>,
    >,
    SourceValue<Schema, RootDemand<Schema, Program>>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, RootDemand<Schema, Program>>>
            + Clone,
    <Source<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, RootDemand<Schema, Program>>,
        >,
    <Source<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(in crate::application_entry) fn new(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        admitted: WorthQueryAdmittedProgramRootOutput<Schema, Program>,
        demand: RootDemand<Schema, Program>,
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
        WorthQueryApplicationProgramRootDemandProgress<Schema, Program>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let disclosure = request
            .query(self.demand.source_intent())
            .execute()
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
            .into_output_demand_disclosure();
        match self.application.advance_program_root_output(
            &self.admitted,
            request.principal,
            request.scope,
            request.branch,
            disclosure,
        )
        .map_err(map_progress_denial)?
        {
            WorthQueryProgramRootOutputAdvance::Pending => {
                Ok(WorthQueryApplicationProgramRootDemandProgress::Pending)
            }
            WorthQueryProgramRootOutputAdvance::Settled(authority) => {
                let settlement = WorthQueryApplicationOutputDemandSettlement::new(
                    authority.retained(),
                    self.admitted.observed_source().clone(),
                );
                Ok(WorthQueryApplicationProgramRootDemandProgress::Settled {
                    settlement,
                    authority,
                })
            }
        }
    }
}

impl<'application, Schema, Program>
    WorthQueryApplicationProgramDependentDemandHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<
        Schema,
        RootDemand = RootDemand<Schema, Program>,
    >,
    SourceValue<Schema, DependentDemand<Schema, Program>>: WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, DependentDemand<Schema, Program>>,
        > + Clone,
    <Source<Schema, DependentDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, DependentDemand<Schema, Program>>,
        >,
    <Source<Schema, DependentDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, DependentDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(in crate::application_entry) fn new(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        admitted: WorthQueryAdmittedProgramDependentOutput<Schema, Program>,
        demand: DependentDemand<Schema, Program>,
    ) -> Self {
        Self {
            application,
            admitted,
            demand,
        }
    }

    pub(in crate::application_entry) fn advance(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationOutputDemandProgress<
            SourceQuery<Schema, DependentDemand<Schema, Program>>,
        >,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let disclosure = request
            .query(self.demand.source_intent())
            .execute()
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
            .into_output_demand_disclosure();
        match self.application.advance_program_dependent_output(
            &self.admitted,
            request.principal,
            request.scope,
            request.branch,
            disclosure,
        )
        .map_err(map_progress_denial)?
        {
            WorthQueryOutputDemandAdvance::Pending => {
                Ok(WorthQueryApplicationOutputDemandProgress::Pending)
            }
            WorthQueryOutputDemandAdvance::Settled(retained) => {
                Ok(WorthQueryApplicationOutputDemandProgress::Settled(
                    WorthQueryApplicationOutputDemandSettlement::new(
                        retained,
                        self.admitted.observed_source().clone(),
                    ),
                ))
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

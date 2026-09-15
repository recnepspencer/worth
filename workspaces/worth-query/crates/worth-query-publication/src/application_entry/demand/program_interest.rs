use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramInventoryIdentity,
};
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

pub(in crate::application_entry) enum WorthQueryProgramNodeProgress<
    Schema,
    Program,
    Inventory,
    Demand,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    Pending,
    Settled {
        settlement: WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, Demand>>,
        authority: WorthQuerySettledProgramOutput<Schema, Program, Inventory, Demand>,
    },
}

pub(in crate::application_entry) struct WorthQueryProgramNodeHandle<
    'application,
    Schema,
    Program,
    Inventory,
    Demand,
> where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    admitted: WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>,
    demand: Demand,
    closed: bool,
}

impl<'application, Schema, Program, Inventory, Demand>
    WorthQueryProgramNodeHandle<'application, Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
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
        admitted: WorthQueryAdmittedProgramOutput<Schema, Program, Inventory, Demand>,
        demand: Demand,
    ) -> Self {
        Self {
            application,
            admitted,
            demand,
            closed: false,
        }
    }

    pub(in crate::application_entry) fn demand(&self) -> &Demand {
        &self.demand
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
        WorthQueryProgramNodeProgress<Schema, Program, Inventory, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let disclosure = request
            .query(self.demand.source_intent())
            .execute()
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
            .into_output_demand_disclosure();
        match worth_query_execution::facade::publication_integration::program_execution_port(
            self.application,
        )
        .advance_program_output(
            &self.admitted,
            request.principal,
            request.scope,
            request.branch,
            disclosure,
        )
        .map_err(map_progress_denial)?
        {
            WorthQueryProgramOutputAdvance::Pending => Ok(WorthQueryProgramNodeProgress::Pending),
            WorthQueryProgramOutputAdvance::Settled(authority) => {
                let settlement = WorthQueryApplicationOutputDemandSettlement::new(
                    authority.retained(),
                    self.admitted.observed_source().clone(),
                );
                Ok(WorthQueryProgramNodeProgress::Settled {
                    settlement,
                    authority,
                })
            }
        }
    }

    pub(in crate::application_entry) fn close(&mut self) {
        if !self.closed {
            self.admitted.close();
            self.closed = true;
        }
    }
}

impl<Schema, Program, Inventory, Demand> Drop
    for WorthQueryProgramNodeHandle<'_, Schema, Program, Inventory, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Inventory: ApplicationProgramInventoryIdentity,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn drop(&mut self) {
        if !self.closed {
            self.admitted.close();
            self.closed = true;
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

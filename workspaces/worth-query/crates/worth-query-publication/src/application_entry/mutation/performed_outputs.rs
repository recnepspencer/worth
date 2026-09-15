use crate::application_entry::demand::{
    WorthQueryApplicationProgramDependentDemandHandle,
    WorthQueryApplicationProgramRootDemandHandle, WorthQueryApplicationProgramRootDemandProgress,
};
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
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection,
};

type RootDemand<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::Connections as WorthQueryApplicationRequiredOutputConnection<
        Schema,
    >>::Demand;
type DependentConnection<Schema, Program> =
    <Program as ApplicationProgramDefinition<Schema>>::DependentConnection;
type DependentIntent<Schema, Program> =
    <DependentConnection<Schema, Program> as WorthQueryApplicationDependentOutputConnection<
        Schema,
    >>::Discovery;
type DependentDemand<Schema, Program> =
    <DependentConnection<Schema, Program> as WorthQueryApplicationDependentOutputConnection<
        Schema,
    >>::Demand;
type DemandSource<Schema, Demand> = <<Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily as WorthQueryProducerOutputFamily<Schema>>::Source;
type DemandQuery<Schema, Demand> =
    <DemandSource<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type DemandValue<Schema, Demand> = <<DemandSource<Schema, Demand> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type DiscoveryBinding<Schema, Program> =
    <DependentIntent<Schema, Program> as ApplicationQueryIntent<Schema>>::Binding;
type DiscoveryValue<Schema, Program> =
    <<DiscoveryBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub enum WorthQueryApplicationProgramOutputProgress<RootQuery, DependentQuery, DependentDemand> {
    Pending,
    Settled(
        WorthQueryApplicationProgramOutputSettlement<RootQuery, DependentQuery, DependentDemand>,
    ),
}

pub struct WorthQueryApplicationProgramOutputSettlement<RootQuery, DependentQuery, DependentDemand>
{
    root: crate::application_entry::WorthQueryApplicationOutputDemandSettlement<RootQuery>,
    dependent: Vec<(
        DependentDemand,
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<DependentQuery>,
    )>,
}

impl<RootQuery, DependentQuery, DependentDemand>
    WorthQueryApplicationProgramOutputSettlement<RootQuery, DependentQuery, DependentDemand>
{
    pub fn root_observation(
        &self,
    ) -> &crate::application_entry::WorthQueryApplicationReadObservation {
        self.root.observation()
    }

    pub fn observation(&self) -> &crate::application_entry::WorthQueryApplicationReadObservation {
        self.dependent
            .iter()
            .map(|(_, settled)| settled.observation())
            .chain(std::iter::once(self.root.observation()))
            .max_by_key(|observation| observation.selected_commit().ordinal())
            .expect("one program settlement always retains its root output")
    }

    pub fn dependent_count(&self) -> usize {
        self.dependent.len()
    }

    pub fn dependents(
        &self,
    ) -> &[(
        DependentDemand,
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<DependentQuery>,
    )] {
        &self.dependent
    }
}

pub struct WorthQueryApplicationProgramOutputHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<
        Schema,
        RootDemand = RootDemand<Schema, Program>,
    >,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    root: Option<WorthQueryApplicationProgramRootDemandHandle<'application, Schema, Program>>,
    root_demand: RootDemand<Schema, Program>,
    root_settlement: Option<
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            DemandQuery<Schema, RootDemand<Schema, Program>>,
        >,
    >,
    dependent: Vec<
        Option<(
            DependentDemand<Schema, Program>,
            WorthQueryApplicationProgramDependentDemandHandle<'application, Schema, Program>,
        )>,
    >,
    dependent_settlements: Vec<(
        DependentDemand<Schema, Program>,
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            DemandQuery<Schema, DependentDemand<Schema, Program>>,
        >,
    )>,
    controls: crate::application_entry::WorthQueryOutputDemandControls,
    complete: bool,
}

impl<'application, Schema, Program>
    WorthQueryApplicationProgramOutputHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<
        Schema,
        RootDemand = RootDemand<Schema, Program>,
    >,
{
    pub(in crate::application_entry) fn new(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        root: WorthQueryApplicationProgramRootDemandHandle<'application, Schema, Program>,
        root_demand: RootDemand<Schema, Program>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Self {
        Self {
            application,
            root: Some(root),
            root_demand,
            root_settlement: None,
            dependent: Vec::new(),
            dependent_settlements: Vec::new(),
            controls,
            complete: false,
        }
    }
}

impl<'application, Schema, Program>
    WorthQueryApplicationProgramOutputHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<
        Schema,
        RootDemand = RootDemand<Schema, Program>,
    >,
    RootDemand<Schema, Program>: Clone,
    DependentDemand<Schema, Program>: Clone,
    DiscoveryValue<Schema, Program>: WorthQueryApplicationProjection<
            Schema,
            <DiscoveryBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::Query,
        > + Clone,
    <DiscoveryBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DiscoveryBinding<Schema, Program> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    DemandValue<Schema, RootDemand<Schema, Program>>:
        WorthQueryApplicationProjection<Schema, DemandQuery<Schema, RootDemand<Schema, Program>>>
            + Clone,
    <DemandSource<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = DemandSource<Schema, RootDemand<Schema, Program>>,
        >,
    <DemandSource<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    DemandValue<Schema, DependentDemand<Schema, Program>>:
        WorthQueryApplicationProjection<
                Schema,
                DemandQuery<Schema, DependentDemand<Schema, Program>>,
            > + Clone,
    <DemandSource<Schema, DependentDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = DemandSource<Schema, DependentDemand<Schema, Program>>,
        >,
    <DemandSource<Schema, DependentDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DemandSource<Schema, DependentDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn notifications(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        self.root
            .as_ref()
            .ok_or(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed)?
            .notifications()
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)
    }

    /// Exact settled root retained while transitive program outputs continue.
    pub fn settled_root_observation(
        &self,
    ) -> Option<&crate::application_entry::WorthQueryApplicationReadObservation> {
        self.root_settlement
            .as_ref()
            .map(crate::application_entry::WorthQueryApplicationOutputDemandSettlement::observation)
    }

    pub fn advance(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<
        WorthQueryApplicationProgramOutputProgress<
            DemandQuery<Schema, RootDemand<Schema, Program>>,
            DemandQuery<Schema, DependentDemand<Schema, Program>>,
            DependentDemand<Schema, Program>,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        if self.complete {
            return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed);
        }
        if let Some(root) = &mut self.root {
            match root
                .advance(request)
                .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?
            {
                WorthQueryApplicationProgramRootDemandProgress::Pending => {
                    return Ok(WorthQueryApplicationProgramOutputProgress::Pending)
                }
                WorthQueryApplicationProgramRootDemandProgress::Settled {
                    settlement,
                    authority,
                } => self.start_dependents(request, settlement, authority)?,
            }
        }
        for index in 0..self.dependent.len() {
            let Some((_, handle)) = &mut self.dependent[index] else {
                continue;
            };
            match handle
                .advance(request)
                .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?
            {
                crate::application_entry::WorthQueryApplicationOutputDemandProgress::Pending => {}
                crate::application_entry::WorthQueryApplicationOutputDemandProgress::Settled(
                    settled,
                ) => {
                    let (demand, _) = self.dependent[index]
                        .take()
                        .expect("the settled dependent retains its typed occurrence demand");
                    self.dependent_settlements.push((demand, settled));
                }
            }
        }
        if self.dependent.iter().any(Option::is_some) {
            return Ok(WorthQueryApplicationProgramOutputProgress::Pending);
        }
        self.complete = true;
        Ok(WorthQueryApplicationProgramOutputProgress::Settled(
            WorthQueryApplicationProgramOutputSettlement {
                root: self
                    .root_settlement
                    .take()
                    .expect("dependent progression retains its root settlement"),
                dependent: std::mem::take(&mut self.dependent_settlements),
            },
        ))
    }

    fn start_dependents(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
        settled: crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            DemandQuery<Schema, RootDemand<Schema, Program>>,
        >,
        authority: worth_query_execution::facade::application_installation::WorthQuerySettledProgramRootOutput<
            Schema,
            Program,
        >,
    ) -> Result<(), crate::application_entry::WorthQueryRequiredOutputPreparationDenial> {
        let discovery = Program::DependentConnection::discovery_from_root(&self.root_demand)
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Connection)?;
        let retained = request.at(settled.observation());
        let result = retained
            .query(discovery)
            .execute()
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
        if result.rows().len() != 1 {
            return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingSource);
        }
        let demands = Program::DependentConnection::demands_from_discovery(&result.rows()[0])
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Connection)?;
        self.dependent = demands
            .into_iter()
            .map(|demand| {
                let handle = retained
                    .demand(demand.clone())
                    .controls(self.controls)
                    .start_dependent(self.application, &authority)
                    .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?;
                Ok(Some((demand, handle)))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.root = None;
        self.root_settlement = Some(settled);
        Ok(())
    }
}

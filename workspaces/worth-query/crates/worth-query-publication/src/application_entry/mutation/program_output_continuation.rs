use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputEdge, ApplicationOutputEdgesShape,
    ApplicationOutputLeaf, ApplicationProgramDefinition,
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
    WorthQueryProgramApplicationRuntime, WorthQuerySettledProgramOutput,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
};

use super::program_output_settlement::ProgramOutputRecord;
use super::program_output_work::ProgramOutputTraversalWork;
use crate::application_entry::demand::{
    WorthQueryApplicationProgramDemandHandle, WorthQueryApplicationProgramDemandProgress,
};
use crate::application_entry::{
    WorthQueryApplicationOutputDemandSettlement, WorthQueryApplicationRequest,
    WorthQueryOutputDemandControls, WorthQueryRequiredOutputPreparationDenial,
};

mod branch;

mod sealed {
    pub trait Continuation {}
    pub trait Factory {}
}

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> =
    <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Binding<Schema, Connection> = <Connection as ApplicationConnectionShape<Schema>>::Binding;
type ChildDemand<Schema, Connection> =
    <Binding<Schema, Connection> as WorthQueryApplicationDependentOutputConnection<Schema>>::Demand;
type Discovery<Schema, Connection> =
    <Binding<Schema, Connection> as WorthQueryApplicationDependentOutputConnection<Schema>>::Discovery;
type DiscoveryBinding<Schema, Connection> =
    <Discovery<Schema, Connection> as ApplicationQueryIntent<Schema>>::Binding;
type DiscoveryValue<Schema, Connection> =
    <<DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

#[doc(hidden)]
pub enum ProgramOutputContinuationProgress {
    Pending,
    Settled {
        outputs: Vec<ProgramOutputRecord>,
        work: ProgramOutputTraversalWork,
    },
}

#[doc(hidden)]
pub trait ProgramOutputContinuation<'application, Schema>: sealed::Continuation
where
    Schema: ApplicationSchema,
{
    fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial>;
}

#[doc(hidden)]
pub trait ProgramOutputContinuationFactory<'application, Schema, Program, ParentDemand>:
    sealed::Factory
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn start(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        _parent_settlement: &WorthQueryApplicationOutputDemandSettlement<
            SourceQuery<Schema, ParentDemand>,
        >,
        parent_authority: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>,
        WorthQueryRequiredOutputPreparationDenial,
    >;
}

struct CompleteContinuation {
    open: bool,
}

impl sealed::Continuation for CompleteContinuation {}
impl sealed::Factory for ApplicationOutputLeaf {}

impl<'application, Schema> ProgramOutputContinuation<'application, Schema> for CompleteContinuation
where
    Schema: ApplicationSchema,
{
    fn advance(
        &mut self,
        _: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        if std::mem::take(&mut self.open) {
            Ok(ProgramOutputContinuationProgress::Settled {
                outputs: Vec::new(),
                work: ProgramOutputTraversalWork::default(),
            })
        } else {
            Err(WorthQueryRequiredOutputPreparationDenial::Closed)
        }
    }
}

impl<'application, Schema, Program, ParentDemand>
    ProgramOutputContinuationFactory<'application, Schema, Program, ParentDemand>
    for ApplicationOutputLeaf
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn start(
        _: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        _: &ParentDemand,
        _parent_settlement: &WorthQueryApplicationOutputDemandSettlement<
            SourceQuery<Schema, ParentDemand>,
        >,
        _: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        _: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
        _: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        Ok(Box::new(CompleteContinuation { open: true }))
    }
}

struct EdgeNode<'application, Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    demand: Demand,
    handle: Option<WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>>,
    continuation: Option<Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>>,
}

struct EdgeContinuation<'application, Schema, Program, Connection, Children>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<Schema>,
    Children: ApplicationOutputEdgesShape<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    nodes: Vec<EdgeNode<'application, Schema, Program, ChildDemand<Schema, Connection>>>,
    outputs: Vec<ProgramOutputRecord>,
    work: ProgramOutputTraversalWork,
    controls: WorthQueryOutputDemandControls,
    marker: std::marker::PhantomData<fn() -> Children>,
}

impl<'application, Schema, Program, Connection, Children> sealed::Continuation
    for EdgeContinuation<'application, Schema, Program, Connection, Children>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<Schema>,
    Children: ApplicationOutputEdgesShape<Schema>,
{
}

impl<Connection, Children> sealed::Factory for ApplicationOutputEdge<Connection, Children> {}

impl<'application, Schema, Program, ParentDemand, Connection, Children>
    ProgramOutputContinuationFactory<'application, Schema, Program, ParentDemand>
    for ApplicationOutputEdge<Connection, Children>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
    Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<
        Schema,
        RootDemand = ParentDemand,
    >,
    ChildDemand<Schema, Connection>: Clone,
    DiscoveryValue<Schema, Connection>: WorthQueryApplicationProjection<
            Schema,
            <DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::Query,
        > + Clone,
    <DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    SourceValue<Schema, ChildDemand<Schema, Connection>>: WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, ChildDemand<Schema, Connection>>,
        > + Clone,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, ChildDemand<Schema, Connection>>,
        >,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Children: ApplicationOutputEdgesShape<Schema>
        + ProgramOutputContinuationFactory<
            'application,
            Schema,
            Program,
            ChildDemand<Schema, Connection>,
        >,
{
    fn start(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        parent_settlement: &WorthQueryApplicationOutputDemandSettlement<
            SourceQuery<Schema, ParentDemand>,
        >,
        parent_authority: &WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        let discovery = Binding::<Schema, Connection>::discovery_from_root(parent_demand)
            .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
        let retained = request.at(parent_settlement.observation());
        let result = retained
            .query(discovery)
            .execute()
            .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
        if result.rows().len() != 1 {
            return Err(WorthQueryRequiredOutputPreparationDenial::MissingSource);
        }
        let discovery_rows = result.rows().len();
        let demands = Binding::<Schema, Connection>::demands_from_discovery(&result.rows()[0])
            .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
        let work = ProgramOutputTraversalWork::discovered(discovery_rows, demands.len());
        let nodes = demands
            .into_iter()
            .map(|demand| {
                let handle = retained
                    .demand(demand.clone())
                    .controls(controls)
                    .start_dependent::<Program, ParentDemand, Connection>(
                        application,
                        parent_authority,
                    )
                    .map_err(WorthQueryRequiredOutputPreparationDenial::Demand)?;
                Ok(EdgeNode {
                    demand,
                    handle: Some(handle),
                    continuation: None,
                })
            })
            .collect::<Result<Vec<_>, WorthQueryRequiredOutputPreparationDenial>>()?;
        Ok(Box::new(EdgeContinuation::<Schema, Program, Connection, Children> {
            application,
            nodes,
            outputs: Vec::new(),
            work,
            controls,
            marker: std::marker::PhantomData,
        }))
    }
}

impl<'application, Schema, Program, Connection, Children>
    ProgramOutputContinuation<'application, Schema>
    for EdgeContinuation<'application, Schema, Program, Connection, Children>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<Schema>,
    ChildDemand<Schema, Connection>: Clone,
    SourceValue<Schema, ChildDemand<Schema, Connection>>: WorthQueryApplicationProjection<
            Schema,
            SourceQuery<Schema, ChildDemand<Schema, Connection>>,
        > + Clone,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, ChildDemand<Schema, Connection>>,
        >,
    <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, ChildDemand<Schema, Connection>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Children: ApplicationOutputEdgesShape<Schema>
        + ProgramOutputContinuationFactory<
            'application,
            Schema,
            Program,
            ChildDemand<Schema, Connection>,
        >,
{
    fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        for node in &mut self.nodes {
            if let Some(handle) = &mut node.handle {
                match handle
                    .advance(request)
                    .map_err(WorthQueryRequiredOutputPreparationDenial::Demand)?
                {
                    WorthQueryApplicationProgramDemandProgress::Pending => {}
                    WorthQueryApplicationProgramDemandProgress::Settled {
                        settlement,
                        authority,
                    } => {
                        node.continuation = Some(Children::start(
                            self.application,
                            &node.demand,
                            &settlement,
                            &authority,
                            request,
                            self.controls,
                        )?);
                        self.outputs.push(ProgramOutputRecord::new::<
                            Schema,
                            Connection,
                        >(node.demand.clone(), settlement));
                        node.handle = None;
                    }
                }
            }
            if let Some(continuation) = &mut node.continuation {
                match continuation.advance(request)? {
                    ProgramOutputContinuationProgress::Pending => {}
                    ProgramOutputContinuationProgress::Settled { outputs, work } => {
                        self.outputs.extend(outputs);
                        self.work.include(work);
                        node.continuation = None;
                    }
                }
            }
        }
        if self
            .nodes
            .iter()
            .any(|node| node.handle.is_some() || node.continuation.is_some())
        {
            return Ok(ProgramOutputContinuationProgress::Pending);
        }
        Ok(ProgramOutputContinuationProgress::Settled {
            outputs: std::mem::take(&mut self.outputs),
            work: self.work,
        })
    }
}

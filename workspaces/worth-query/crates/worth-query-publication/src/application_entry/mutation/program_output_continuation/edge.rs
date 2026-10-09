use super::*;
use std::sync::Arc;

struct EdgeNode<Schema, Program, Demand>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    demand: Demand,
    handle: Option<WorthQueryApplicationProgramDemandHandle<Schema, Program, Demand>>,
    settled: Option<(
        WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, Demand>>,
        Arc<WorthQuerySettledProgramOutput<Schema, Program, Demand>>,
    )>,
    continuation: Option<Box<dyn ProgramOutputContinuation<Schema, Program>>>,
}

struct EdgeContinuation<Schema, Program, ParentDemand, Connection, Children>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
    Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<Schema>,
    Children: ApplicationOutputEdgesShape<Schema>,
{
    parent_demand: ParentDemand,
    parent_authority: Arc<WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>>,
    nodes: Option<Vec<EdgeNode<Schema, Program, ChildDemand<Schema, Connection>>>>,
    next_admission: usize,
    outputs: Vec<ProgramOutputRecord>,
    work: ProgramOutputTraversalWork,
    controls: WorthQueryOutputDemandControls,
    basis: crate::application_entry::WorthQueryApplicationReadObservation,
    minimum_observation: crate::application_entry::WorthQueryApplicationReadObservation,
    marker: std::marker::PhantomData<fn() -> Children>,
}

impl<Schema, Program, ParentDemand, Connection, Children> sealed::Continuation
    for EdgeContinuation<Schema, Program, ParentDemand, Connection, Children>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
    Connection: ApplicationConnectionShape<Schema>,
    Binding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<Schema>,
    Children: ApplicationOutputEdgesShape<Schema>,
{
}

impl<Connection, Children> sealed::Factory for ApplicationOutputEdge<Connection, Children> {}

impl< Schema, Program, ParentDemand, Connection, Children>
    ProgramOutputContinuationFactory<Schema, Program, ParentDemand>
    for ApplicationOutputEdge<Connection, Children>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema> + Clone,
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
        + ProgramOutputContinuationFactory<Schema,
            Program,
            ChildDemand<Schema, Connection>,
        >,
{
    fn start(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        parent_settlement: &WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, ParentDemand>>,
        parent_authority: &Arc<WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>>,
        minimum_observation: &crate::application_entry::WorthQueryApplicationReadObservation,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<Box<dyn ProgramOutputContinuation<Schema, Program>>, WorthQueryRequiredOutputPreparationDenial> {
        let basis = if parent_settlement.observation().selected_commit().ordinal()
            >= minimum_observation.selected_commit().ordinal() {
            parent_settlement.observation()
        } else { minimum_observation };
        // Setup is effect-free. Discovery/admission belongs to the retained state,
        // so a later sibling refusal cannot drop an earlier admitted child.
        Ok(Box::new(EdgeContinuation::<Schema, Program, ParentDemand, Connection, Children> {
            parent_demand: parent_demand.clone(),
            parent_authority: Arc::clone(parent_authority),
            nodes: None,
            next_admission: 0,
            outputs: Vec::new(),
            work: ProgramOutputTraversalWork::default(),
            controls,
            basis: basis.retained_clone(),
            minimum_observation: minimum_observation.retained_clone(),
            marker: std::marker::PhantomData,
        }))
    }
}

impl< Schema, Program, ParentDemand, Connection, Children>
    ProgramOutputContinuation<Schema, Program>
    for EdgeContinuation<Schema, Program, ParentDemand, Connection, Children>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema> + Clone,
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
        + ProgramOutputContinuationFactory<Schema,
            Program,
            ChildDemand<Schema, Connection>,
        >,
{
    fn advance(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryRequiredOutputPreparationDenial::ForeignProgram);
        }
        if self.nodes.is_none() {
            let discovery = Binding::<Schema, Connection>::discovery_from_root(&self.parent_demand)
                .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
            let result = request.at(&self.basis).query(discovery).execute_in_advancement(phase)
                .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
            if result.rows().len() != 1 {
                return Err(WorthQueryRequiredOutputPreparationDenial::MissingSource);
            }
            let demands = Binding::<Schema, Connection>::demands_from_discovery(&result.rows()[0])
                .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
            self.work = ProgramOutputTraversalWork::discovered(result.rows().len(), demands.len());
            self.nodes = Some(demands.into_iter().map(|demand| EdgeNode {
                demand, handle: None, settled: None, continuation: None,
            }).collect());
        }
        let nodes = self.nodes.as_mut().expect("discovery retains its exact child list");
        while let Some(node) = nodes.get_mut(self.next_admission) {
            let handle = request.at(&self.basis).demand(node.demand.clone()).controls(self.controls)
                .start_dependent::<Program, ParentDemand, Connection>(
                    phase,
                    application, &self.parent_authority, &self.basis, &self.minimum_observation)
                .map_err(WorthQueryRequiredOutputPreparationDenial::Demand)?;
            node.handle = Some(handle);
            self.next_admission += 1;
        }
        for node in nodes.iter_mut() {
            if let Some(handle) = &mut node.handle {
                match handle.advance(phase, application, request).map_err(WorthQueryRequiredOutputPreparationDenial::Demand)? {
                    WorthQueryApplicationProgramDemandProgress::Pending => continue,
                    WorthQueryApplicationProgramDemandProgress::Settled { settlement, authority } => {
                        node.settled = Some((settlement, Arc::new(authority)));
                        node.handle = None;
                    }
                }
            }
            if let Some((settlement, authority)) = node.settled.as_ref() {
                node.continuation = Some(Children::start(application, &node.demand,
                    settlement, authority, &self.basis, request, self.controls)?);
            }
            if let Some((settlement, _)) = node.settled.take() {
                self.outputs.push(ProgramOutputRecord::new::<Schema, Connection>(node.demand.clone(), settlement));
            }
            if let Some(continuation) = &mut node.continuation {
                match continuation.advance(phase, application, request)? {
                    ProgramOutputContinuationProgress::Pending => {},
                    ProgramOutputContinuationProgress::Settled { outputs, work } => {
                        self.outputs.extend(outputs);
                        self.work.include(work);
                        node.continuation = None;
                    }
                }
            }
        }
        if nodes.iter().any(|node| node.handle.is_some() || node.settled.is_some() || node.continuation.is_some()) {
            return Ok(ProgramOutputContinuationProgress::Pending);
        }
        Ok(ProgramOutputContinuationProgress::Settled {
            outputs: std::mem::take(&mut self.outputs), work: self.work,
        })
    }
}

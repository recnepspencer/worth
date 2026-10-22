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
    _retained_handle: Option<WorthQueryApplicationProgramDemandHandle<Schema, Program, Demand>>,
    continuation_complete: bool,
    settled: Option<(
        WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, Demand>>,
        Arc<WorthQuerySettledProgramOutput<Schema, Program, Demand>>,
    )>,
    continuation: Option<Box<dyn ProgramOutputContinuation<Schema, Program>>>,
    dependencies: Option<WorthQueryRetainedQueryDependencies>,
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
    discovery_source: Option<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationOutputDemandSource<
            <DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::Query,
            DiscoveryValue<Schema, Connection>,
        >,
    >,
    dependencies: Option<WorthQueryRetainedQueryDependencies>,
    next_admission: usize,
    next_advance: usize,
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
            discovery_source: None,
            dependencies: None,
            next_admission: 0,
            next_advance: 0,
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
    fn close_read_request(&mut self, phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>, application: &WorthQueryProgramApplicationRuntime<Schema, Program>, branch: worth_query_execution::facade::product::WorthQueryProductBranch) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryOutputReadCloseDenial> { if let Some(nodes) = self.nodes.as_mut() {
            for node in nodes {
                if let Some(children) = node.continuation.as_mut() { children.close_read_request(phase, application, branch)?; }
                if let Some(handle) = node.handle.as_mut() { handle.close_read_request(phase, application, branch)?; }
                if let Some(handle) = node._retained_handle.as_mut() { handle.close_read_request(phase, application, branch)?; }
            }
        }
        Ok(()) }
    fn advance(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial> {
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryRequiredOutputPreparationDenial::ForeignProgram);
        }
        if self.discovery_source.is_none() && self.dependencies.is_none() {
            let discovery = Binding::<Schema, Connection>::discovery_from_root(&self.parent_demand)
                .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
            let result = request.at(&self.basis).query(discovery).execute_in_advancement(phase)
                .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
            self.discovery_source = Some(result.into_output_demand_source());
        }
        if self.nodes.is_none() {
            let result = self.discovery_source.as_ref().expect("original discovery pairing remains owned");
            if result.rows().len() != 1 { return Err(WorthQueryRequiredOutputPreparationDenial::MissingSource); }
            let demands = Binding::<Schema, Connection>::demands_from_discovery(&result.rows()[0])
                .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
            self.work = ProgramOutputTraversalWork::discovered(result.rows().len(), demands.len());
            self.nodes = Some(demands.into_iter().map(|demand| EdgeNode {
                demand, handle: None, _retained_handle: None, continuation_complete: false,
                settled: None, continuation: None, dependencies: None,
            }).collect());
        }
        if self.dependencies.is_none() {
            let proof = request.at(&self.basis).retain_query_dependencies(phase,
                self.discovery_source.as_ref().expect("capture borrows original pairing"))?;
            self.dependencies = Some(proof);
            self.discovery_source = None;
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
        let mut runnable = false;
        for index in (self.next_advance..nodes.len()).chain(0..self.next_advance) {
            // A native handoff returns to the caller. Resume with the next
            // sibling so an unfinished predecessor does not monopolize progress.
            let next = if index + 1 == nodes.len() { 0 } else { index + 1 };
            let node = &mut nodes[index];
            if let Some(handle) = &mut node.handle {
                match handle.advance(phase, application, request).map_err(WorthQueryRequiredOutputPreparationDenial::Demand)? {
                    WorthQueryApplicationProgramDemandProgress::Pending => continue,
                    WorthQueryApplicationProgramDemandProgress::Progressed => {
                        self.next_advance = next;
                        return Ok(ProgramOutputContinuationProgress::Progressed);
                    },
                    WorthQueryApplicationProgramDemandProgress::Settled { settlement, authority } => {
                        node.settled = Some((settlement, Arc::new(authority)));
                        // Descendants and later siblings can still consume this
                        // exact posting. Retain its real admitted interest.
                        node._retained_handle = node.handle.take();
                        self.next_advance = next;
                        // Keep settlement and posting custody before starting
                        // another independent discovery/admission stage.
                        return Ok(ProgramOutputContinuationProgress::Progressed);
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
            if !node.continuation_complete {
                let Some(continuation) = &mut node.continuation else { continue; };
                match continuation.advance(phase, application, request)? {
                    ProgramOutputContinuationProgress::Pending => {},
                    ProgramOutputContinuationProgress::Progressed => {
                        self.next_advance = next;
                        return Ok(ProgramOutputContinuationProgress::Progressed);
                    },
                    ProgramOutputContinuationProgress::Settled { outputs, work, dependencies } => {
                        node.dependencies = dependencies;
                        self.outputs.extend(outputs);
                        self.work.include(work);
                        node.continuation_complete = true;
                        runnable = true;
                    }
                }
            }
        }
        if nodes.iter().any(|node| node.handle.is_some() || node.settled.is_some() || !node.continuation_complete) {
            return Ok(if runnable { ProgramOutputContinuationProgress::Progressed } else { ProgramOutputContinuationProgress::Pending });
        }
        let dependencies = request.at(&self.basis).combine_query_dependencies(phase,
            self.dependencies.iter().chain(nodes.iter().filter_map(|node| node.dependencies.as_ref())))?;
        // No original evidence or closed prefix moves before the seal succeeds.
        self.dependencies = None;
        for node in nodes { node.dependencies = None; }
        Ok(ProgramOutputContinuationProgress::Settled {
            dependencies: Some(dependencies),
            outputs: std::mem::take(&mut self.outputs), work: self.work,
        })
    }
}

//! One retained nominal demand dispatches preparation and its genuine continuations.
use super::*;

pub(in super::super) trait RequiredTree<S: ApplicationSchema>:
    Send + 'static
{
    fn family(&self) -> &'static str;
    fn order(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder;
    fn seed(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        wave: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn occurrence(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<(occurrences::OccurrenceKey, std::any::TypeId)>, SynchronousOutputFailure>;
    fn has_static_continuation(&self) -> bool;
    fn prerequisites(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn prepare(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<AppliedOutputParent>, SynchronousOutputFailure>;
    fn discover(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn discover_static(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn discover_occurrences(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
}
pub(super) struct Tree<D, K> {
    pub(super) connection: &'static str,
    pub(super) position:
        worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
    pub(super) order:
        worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    pub(super) demand: D,
    pub(super) continuation: K,
}
impl<S, D, K> RequiredTree<S> for Tree<D, K>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S> + Clone + Send,
    K: Continuation<S, D>,
    SourceInput<S, D>: ApplicationQueryIntent<S, Binding = Source<S, D>>,
    Scope<S, D>: ApplicationQueryScopeResolution<S, Principal<S, D>>,
{
    fn family(&self) -> &'static str {
        <D::OutputFamily as worth_query_declaration::facade::application_program::WorthQueryProducerOutputFamily<S>>::IDENTITY
    }
    fn order(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder
    {
        &self.order
    }
    fn seed(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        wave: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        wave.push(
            context,
            self.connection,
            &self.position,
            size_of::<Self>(),
            || {
                Box::new(Self {
                    connection: self.connection,
                    position: self.position.clone(),
                    order: self.order.clone(),
                    demand: self.demand.clone(),
                    continuation: self.continuation.clone(),
                })
            },
        )
    }
    fn occurrence(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<(occurrences::OccurrenceKey, std::any::TypeId)>, SynchronousOutputFailure>
    {
        let Some(component) = self
            .continuation
            .occurrence_context(std::any::TypeId::of::<D::OutputFamily>())?
        else {
            return Ok(None);
        };
        let demand = source::capture::<S, D>(self.connection, self.demand.clone());
        let principal = context.owner.principal.ok_or_else(|| {
            occurrences::refusal(self.connection, "current principal unavailable")
        })?;
        let reader = context
            .owner
            .application
            .installed_producers
            .declared_coupled_source(&demand)
            .map_err(CoupledSourceReadStop::from)?;
        let route = reader.coupled_executor().read_coupled_source(
            context.execution,
            context.owner.application,
            principal,
            context.owner.request,
            context.owner.before,
            context.before,
            &demand,
            candidate,
            true,
        )?;
        match route {
            super::super::execution::CoupledSourceRead::Route { scope, partition } => Ok(Some((
                occurrences::OccurrenceKey {
                    family: demand.family,
                    source_binding: demand.source_binding_type,
                    scope,
                    partition,
                },
                component,
            ))),
            _ => Err(occurrences::refusal(
                self.connection,
                "route-only source returned a result",
            )),
        }
    }
    fn has_static_continuation(&self) -> bool {
        self.continuation.has_static_continuation()
    }
    fn prerequisites(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        self.continuation
            .prerequisites(context, pin, source, &self.demand, next)
    }
    fn prepare(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<AppliedOutputParent>, SynchronousOutputFailure> {
        context.checkpoint(self.connection)?;
        let demand = source::capture::<S, D>(self.connection, self.demand.clone());
        Ok(context
            .owner
            .prepare_one(context.execution, context.before, &demand, candidate)?)
    }
    fn discover(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        self.continuation
            .assemble(context, parent, &self.demand, next)
    }
    fn discover_static(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        self.continuation
            .assemble_static(context, parent, &self.demand, next)
    }
    fn discover_occurrences(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        self.continuation
            .assemble_occurrences(context, parent, &self.demand, next)
    }
}

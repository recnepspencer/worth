//! Immutable, sendable required demand carried into its source's assembly.
use super::*;
use source::{Source, SourceInput};
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_program::WorthQueryApplicationRequiredOutputConnection,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
};
type Scope<S, D> = <Source<S, D> as ApplicationQueryBinding<S>>::ScopeBinding;
type Principal<S, D> = <Source<S, D> as ApplicationQueryBinding<S>>::PrincipalIdentity;

pub(super) trait RequiredTree<S: ApplicationSchema>: Send + 'static {
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
    ) -> Result<Option<occurrences::OccurrenceKey>, SynchronousOutputFailure>;
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
}
struct Tree<D, E> {
    connection: &'static str,
    position:
        worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
    order: worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    demand: D,
    marker: PhantomData<fn() -> E>,
}
impl<S, D, E> RequiredTree<S> for Tree<D, E>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S> + Clone + Send,
    E: SynchronousOutputEdges<S, D> + 'static,
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
                    marker: PhantomData,
                })
            },
        )
    }
    fn occurrence(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<Option<occurrences::OccurrenceKey>, SynchronousOutputFailure> {
        if !E::occurrence_component() {
            return Ok(None);
        }
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
            super::super::execution::CoupledSourceRead::Route { scope, partition } => {
                Ok(Some(occurrences::OccurrenceKey {
                    family: demand.family,
                    source_binding: demand.source_binding_type,
                    scope,
                    partition,
                }))
            }
            _ => Err(occurrences::refusal(
                self.connection,
                "route-only source returned a result",
            )),
        }
    }
    fn prerequisites(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        E::prerequisites(context, pin, source, &self.demand, next)
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
        E::assemble(context, parent, &self.demand, next)
    }
}
pub(super) fn schedule<S, D, E>(
    context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
    wave: &mut SynchronousOutputWave<S>,
    connection: &'static str,
    position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
    demand: D,
) -> Result<(), SynchronousOutputFailure>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S> + Clone + Send,
    E: SynchronousOutputEdges<S, D> + 'static,
    SourceInput<S, D>: ApplicationQueryIntent<S, Binding = Source<S, D>>,
    Scope<S, D>: ApplicationQueryScopeResolution<S, Principal<S, D>>,
{
    wave.push(
        context,
        connection,
        &position,
        std::mem::size_of::<Tree<D, E>>(),
        || {
            Box::new(Tree::<D, E> {
                connection,
                position: position.clone(),
                order: context.order.clone(),
                demand,
                marker: PhantomData,
            })
        },
    )
}
impl CapturedSynchronousOutputs {
    pub(in crate::domain_computation::primary_graph::application_contribution) fn fixed<S, D, E>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        connection: &'static str,
        demand: D,
        position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        order: &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    ) -> Self
    where
        S: ApplicationSchema,
        D: WorthQueryApplicationOutputDemand<S> + Clone + Send,
        E: SynchronousOutputEdges<S, D> + 'static,
        SourceInput<S, D>: ApplicationQueryIntent<S, Binding = Source<S, D>>,
        Scope<S, D>: ApplicationQueryScopeResolution<S, Principal<S, D>>,
    {
        Self::capture(
            execution,
            connection,
            size_of::<Tree<D, E>>() + size_of::<root::Root<S>>(),
            || {
                Box::new(root::Root::Fixed(Box::new(Tree::<D, E> {
                    connection,
                    position,
                    order: order.clone(),
                    demand,
                    marker: PhantomData,
                })))
            },
        )
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn assemble<
        S: ApplicationSchema,
    >(
        roots: &[&Self],
        owner: &CoupledApplicationAssembly<'_, '_, '_, S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        let mut wave = SynchronousOutputWave::new();
        let order = Self::seed(roots, owner, execution, before, candidate, &mut wave)?;
        let Some(order) = order else { return Ok(()) };
        let context = SynchronousOutputAssembly {
            owner,
            execution,
            before,
            order: &order,
        };
        wave.drive(&context, candidate, None)
            .map_err(|cause| cause.0)
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn seed<
        S: ApplicationSchema,
    >(
        roots: &[&Self],
        owner: &CoupledApplicationAssembly<'_, '_, '_, S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        wave: &mut SynchronousOutputWave<S>,
    ) -> Result<
        Option<
            worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
        >,
        CoupledSourceReadStop,
    > {
        let mut order = None;
        for captured in roots {
            let Self::Prepared { root, _memory } = captured else {
                let Self::Refused(cause) = captured else {
                    unreachable!()
                };
                return Err(CoupledSourceReadStop::capture(cause.denial()));
            };
            let tree = root.downcast_ref::<root::Root<S>>().ok_or_else(|| {
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::ForeignDemand,
                    "required graph schema",
                )
            })?;
            retention_observation::record(
                execution.has_accounting(),
                tree.container_bytes(),
                _memory
                    .as_ref()
                    .map(worth_execution::ExecutionMemoryReservation::bytes),
            );
            let context = SynchronousOutputAssembly {
                owner,
                execution,
                before,
                order: tree.order(),
            };
            order = Some(tree.order().clone());
            tree.seed(&context, candidate, wave)
                .map_err(|cause| cause.0)?;
        }
        Ok(order)
    }
}
impl<S: ApplicationSchema, O, I, Scope>
    crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<S, O, I, Scope>
{
    pub fn with_synchronous_required_outputs<C, E>(
        mut self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        demand: C::Demand,
        position: worth_query_declaration::facade::application_program::ApplicationConnectionDeclaration,
        order: &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder,
    ) -> Self
    where
        C: WorthQueryApplicationRequiredOutputConnection<S>,
        C::Demand: Clone + Send,
        E: SynchronousOutputEdges<S, C::Demand> + 'static,
        SourceInput<S, C::Demand>: ApplicationQueryIntent<S, Binding = Source<S, C::Demand>>,
        <Source<S, C::Demand> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <Source<S, C::Demand> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
    {
        self.synchronous_outputs
            .push(CapturedSynchronousOutputs::capture(
                execution,
                C::IDENTITY,
                std::mem::size_of::<Tree<C::Demand, E>>() + std::mem::size_of::<root::Root<S>>(),
                || {
                    let tree: Box<dyn RequiredTree<S>> = Box::new(Tree::<C::Demand, E> {
                        connection: C::IDENTITY,
                        position,
                        order: order.clone(),
                        demand,
                        marker: PhantomData,
                    });
                    Box::new(root::Root::Fixed(tree))
                },
            ));
        self
    }
}

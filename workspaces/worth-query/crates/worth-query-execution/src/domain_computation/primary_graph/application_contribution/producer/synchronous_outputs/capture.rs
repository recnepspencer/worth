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
    fn order(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationProgramConnectionOrder;
    fn seed(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        wave: &mut SynchronousOutputWave<S>,
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
            order = Some(tree.order());
            tree.seed(&context, candidate, &mut wave)
                .map_err(|cause| cause.0)?;
        }
        let Some(order) = order else { return Ok(()) };
        let context = SynchronousOutputAssembly {
            owner,
            execution,
            before,
            order,
        };
        wave.drive(&context, candidate).map_err(|cause| cause.0)
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

//! Immutable, sendable required demand carried into its source's assembly.
use super::*;
use source::{Source, SourceInput};
use std::{any::Any, marker::PhantomData};
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
                order: context.order.clone(),
                demand,
                marker: PhantomData,
            })
        },
    )
}
/// Private erasure follows the effect; only its schema-typed owner opens it.
pub(in crate::domain_computation::primary_graph) struct CapturedSynchronousOutputs(
    Box<dyn Any + Send>,
);
impl CapturedSynchronousOutputs {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn assemble<
        S: ApplicationSchema,
    >(
        &self,
        owner: &DeclaredProducerAssembly<'_, '_, '_, S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        before: &RelationalPinnedRead,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        let tree = self
            .0
            .downcast_ref::<Box<dyn RequiredTree<S>>>()
            .ok_or_else(|| {
                super::super::WorthQueryOutputDemandDenial::new(
                    super::super::WorthQueryOutputDemandDenialKind::ForeignDemand,
                    "required graph schema",
                )
            })?;
        let order = tree.order();
        let context = SynchronousOutputAssembly {
            owner,
            execution,
            before,
            order,
        };
        let mut wave = SynchronousOutputWave::new();
        if let Some(parent) = tree.prepare(&context, candidate).map_err(|cause| cause.0)? {
            tree.discover(&context, &parent, &mut wave)
                .map_err(|cause| cause.0)?;
        }
        wave.drive(&context, candidate).map_err(|cause| cause.0)
    }
}
impl<S: ApplicationSchema, O, I, Scope>
    crate::domain_computation::primary_graph::WorthQueryApplicationEffectProgram<S, O, I, Scope>
{
    pub fn with_synchronous_required_outputs<C, E>(
        mut self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: C::Demand,
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
        let tree: Box<dyn RequiredTree<S>> = Box::new(Tree::<C::Demand, E> {
            connection: C::IDENTITY,
            order: order.clone(),
            demand,
            marker: PhantomData,
        });
        self.synchronous_outputs = Some(CapturedSynchronousOutputs(Box::new(tree)));
        self
    }
}

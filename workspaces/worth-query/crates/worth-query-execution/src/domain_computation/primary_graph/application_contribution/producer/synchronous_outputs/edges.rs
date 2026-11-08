//! The sealed declared topology dispatches discovery; it never supplies children.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
};
use worth_query_declaration::facade::{
    application_program::{ApplicationConnectionShape, ApplicationOutputEdge},
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
type Binding<S, C> = <C as ApplicationConnectionShape<S>>::Binding;
type Discovery<S, C> =
    <Binding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::Discovery;
type QueryBinding<S, C> = <Discovery<S, C> as ApplicationQueryIntent<S>>::Binding;
type Row<S,C> = <<QueryBinding<S,C> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Child<S, C> = <Binding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::Demand;
impl<C, E> sealed::Edges for ApplicationOutputEdge<C, E> {}
impl<S, D, C, E> SynchronousOutputEdges<S, D> for ApplicationOutputEdge<C, E>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
    C: ApplicationConnectionShape<S>,
    Binding<S, C>: WorthQueryApplicationDependentOutputConnection<S, RootDemand = D>,
    Child<S, C>: Clone + Send,
    E: SynchronousOutputEdges<S, Child<S, C>> + 'static,
    <QueryBinding<S, C> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <QueryBinding<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    Row<S, C>: WorthQueryApplicationProjection<
        S,
        <QueryBinding<S, C> as ApplicationQueryBinding<S>>::Query,
    >,
    source::SourceInput<S, Child<S, C>>:
        ApplicationQueryIntent<S, Binding = source::Source<S, Child<S, C>>>,
    <source::Source<S, Child<S, C>> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <source::Source<S, Child<S, C>> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
{
    fn assemble(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        let connection =
            <Binding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::IDENTITY;
        let refused = |cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::Query(
                super::super::execution::denial::failed(connection, cause).into(),
            ))
        };
        context.checkpoint(connection)?;
        let intent = Binding::<S, C>::discovery_from_root(demand)
            .map_err(|cause| refused(format!("{cause:?}")))?;
        let discovered = context
            .discover::<QueryBinding<S, C>>(parent, intent)
            .map_err(SynchronousOutputFailure)?;
        let children = Binding::<S, C>::demands_from_discovery(&discovered)
            .map_err(|cause| refused(format!("{cause:?}")))?;
        for child in children {
            context.checkpoint(connection)?;
            capture::schedule::<S, Child<S, C>, E>(
                context,
                next,
                connection,
                C::declaration(),
                child,
            )?;
        }
        Ok(())
    }
}
impl<L, R> sealed::Edges for (L, R) {}
impl<S, D, L, R> SynchronousOutputEdges<S, D> for (L, R)
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
    L: SynchronousOutputEdges<S, D>,
    R: SynchronousOutputEdges<S, D>,
{
    fn assemble(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        L::assemble(context, parent, demand, next)?;
        R::assemble(context, parent, demand, next)
    }
}

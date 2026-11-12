//! A recorded source recipe discovers roots at D; only current C grants access.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationProjection,
};
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_program::{
        ApplicationConnectionShape, ApplicationDiscoveredOutputGraph,
        ApplicationDiscoveredOutputRoot, ApplicationOutputGraphShape,
        ApplicationProgramConnectionOrder,
    },
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
type Binding<S, C> = <C as ApplicationConnectionShape<S>>::Binding;
type Recipe<S, C> =
    <Binding<S, C> as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery;
type Discovery<S, C> = <Recipe<S, C> as ApplicationQueryIntent<S>>::Binding;
type Row<S,C> = <<Discovery<S,C> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Demand<S, C> = <Binding<S, C> as WorthQueryApplicationDiscoveredOutputConnection<S>>::Demand;
type Source<S, C> = source::Source<S, Demand<S, C>>;
mod sealed {
    pub trait Root {}
}
impl<C, E> sealed::Root for ApplicationDiscoveredOutputGraph<C, E> {}

/// Executable discovered roots carry their exact typed recipe into the candidate.
/// The sealed implementation owns query admission and graph scheduling; this is
/// never a dependent-set certificate or a producer settlement.
pub trait SynchronousDiscoveredOutputRoot<S>:
    ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot + sealed::Root
where
    S: ApplicationSchema,
    <Self::RootConnection as ApplicationConnectionShape<S>>::Binding:
        WorthQueryApplicationDiscoveredOutputConnection<S>,
{
    fn attach<O, I, Scope>(
        program: WorthQueryApplicationEffectProgram<S, O, I, Scope>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recipe: std::sync::Arc<<<Self::RootConnection as ApplicationConnectionShape<S>>::Binding as WorthQueryApplicationDiscoveredOutputConnection<S>>::Discovery>,
        order: &ApplicationProgramConnectionOrder,
    ) -> WorthQueryApplicationEffectProgram<S, O, I, Scope>;
}
struct Discovered<C, E, Recipe> {
    recipe: std::sync::Arc<Recipe>,
    order: ApplicationProgramConnectionOrder,
    marker: PhantomData<fn() -> (C, E)>,
}
impl<S, C, E> root::Discovered<S> for Discovered<C, E, Recipe<S, C>>
where
    S: ApplicationSchema,
    C: ApplicationConnectionShape<S>,
    Binding<S, C>: WorthQueryApplicationDiscoveredOutputConnection<S>,
    Demand<S, C>: Clone + Send,
    E: SynchronousOutputEdges<S, Demand<S, C>> + 'static,
    <Discovery<S, C> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Discovery<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    Row<S, C>:
        WorthQueryApplicationProjection<S, <Discovery<S, C> as ApplicationQueryBinding<S>>::Query>,
    <Source<S, C> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, C>>,
    <Source<S, C> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
{
    fn order(&self) -> &ApplicationProgramConnectionOrder {
        &self.order
    }
    fn seed(
        &self,
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        wave: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        let connection = Binding::<S, C>::IDENTITY;
        context.checkpoint(connection)?;
        let authored = AppliedAuthoredDiscovery {
            source: candidate.authored_source(),
            pin: candidate
                .pin_source(context.execution)
                .map_err(CoupledSourceReadStop::from)?,
        };
        let discovered = context
            .discover_authored::<Discovery<S, C>>(&authored, self.recipe.as_ref().clone())
            .map_err(SynchronousOutputFailure)?;
        let demands = Binding::<S, C>::demands_from_discovery(&discovered).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::from(
                super::super::WorthQueryOutputDemandDenial::discovery_connection_denied(
                    connection, cause,
                ),
            ))
        })?;
        for demand in demands {
            context.checkpoint(connection)?;
            capture::schedule::<S, Demand<S, C>, E>(
                context,
                wave,
                connection,
                C::declaration(),
                demand,
            )?;
        }
        Ok(())
    }
}
impl<S, C, E> SynchronousDiscoveredOutputRoot<S> for ApplicationDiscoveredOutputGraph<C, E>
where
    S: ApplicationSchema,
    C: ApplicationConnectionShape<S>,
    ApplicationDiscoveredOutputGraph<C, E>:
        ApplicationOutputGraphShape<S, RootConnection = C, Dependents = E>,
    Binding<S, C>: WorthQueryApplicationDiscoveredOutputConnection<S>,
    Demand<S, C>: Clone + Send,
    E: SynchronousOutputEdges<S, Demand<S, C>> + 'static,
    <Discovery<S, C> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Discovery<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    Row<S, C>:
        WorthQueryApplicationProjection<S, <Discovery<S, C> as ApplicationQueryBinding<S>>::Query>,
    <Source<S, C> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, C>>,
    <Source<S, C> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
{
    fn attach<O, I, Scope>(
        mut program: WorthQueryApplicationEffectProgram<S, O, I, Scope>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        recipe: std::sync::Arc<Recipe<S, C>>,
        order: &ApplicationProgramConnectionOrder,
    ) -> WorthQueryApplicationEffectProgram<S, O, I, Scope> {
        program
            .synchronous_outputs
            .push(CapturedSynchronousOutputs::capture(
                execution,
                Binding::<S, C>::IDENTITY,
                std::mem::size_of::<Discovered<C, E, Recipe<S, C>>>()
                    + std::mem::size_of::<root::Root<S>>(),
                || {
                    let root: root::Root<S> =
                        root::Root::Discovered(Box::new(Discovered::<C, E, Recipe<S, C>> {
                            recipe,
                            order: order.clone(),
                            marker: PhantomData,
                        }));
                    Box::new(root)
                },
            ));
        program
    }
}

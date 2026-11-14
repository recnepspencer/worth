//! Actual prerequisite closure within the existing funded output wave.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationPrerequisiteOutputConnection, WorthQueryApplicationProjection,
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
    WorthQueryPrerequisiteSelection,
};
use std::any::Any;
use worth_query_declaration::facade::{
    application_program::{ApplicationConnectionShape, ApplicationOutputOccurrences},
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct OccurrenceKey {
    pub(super) family: &'static str,
    pub(super) source_binding: crate::equality_type_id::EqualityTypeId,
    pub(super) scope: worth_relational::facade::identity::EntityId,
    pub(super) partition: [u8; 32],
}
pub(super) fn refusal(subject: &str, detail: &str) -> SynchronousOutputFailure {
    SynchronousOutputFailure(CoupledSourceReadStop::from(
        super::super::WorthQueryOutputDemandDenial::new(
            super::super::WorthQueryOutputDemandDenialKind::ForeignDemand,
            format!("{subject}: {detail}"),
        ),
    ))
}

type Binding<S, C> = <C as ApplicationConnectionShape<S>>::Binding;
type Parent<S, C> =
    <Binding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::RootDemand;
type Child<S, C> = <Binding<S, C> as WorthQueryApplicationDependentOutputConnection<S>>::Demand;
type Discovery<S,C> = <<Binding<S,C> as WorthQueryApplicationDependentOutputConnection<S>>::Discovery as ApplicationQueryIntent<S>>::Binding;
type Prerequisites<S, C> = <<Binding<S, C> as WorthQueryApplicationPrerequisiteOutputConnection<
    S,
>>::Prerequisites as ApplicationQueryIntent<S>>::Binding;
type Row<S,C> = <<Binding<S,C> as WorthQueryApplicationPrerequisiteOutputConnection<S>>::PrerequisiteRow as ApplicationStructuredValueBinding>::Value;
type DeclaredRow<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

struct PrerequisiteRows<S: ApplicationSchema, C: ApplicationConnectionShape<S>>(Vec<Row<S, C>>)
where
    Binding<S, C>: WorthQueryApplicationPrerequisiteOutputConnection<S>;
impl<S, C>
    WorthQueryApplicationProjection<S, <Prerequisites<S, C> as ApplicationQueryBinding<S>>::Query>
    for PrerequisiteRows<S, C>
where
    S: ApplicationSchema,
    C: ApplicationConnectionShape<S>,
    Binding<S, C>: WorthQueryApplicationPrerequisiteOutputConnection<S>,
    Row<S, C>: WorthQueryApplicationProjection<
        S,
        <Prerequisites<S, C> as ApplicationQueryBinding<S>>::Query,
    >,
    <Binding<S, C> as WorthQueryApplicationPrerequisiteOutputConnection<S>>::Selection:
        WorthQueryPrerequisiteSelection<
            S,
            <Prerequisites<S, C> as ApplicationQueryBinding<S>>::Query,
        >,
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            S,
            <Prerequisites<S, C> as ApplicationQueryBinding<S>>::Query,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self(Binding::<S, C>::prerequisite_rows().project_all(row)?))
    }
}

trait ConnectionExecution<S: ApplicationSchema>: ApplicationConnectionShape<S> {
    fn families(visit: &mut impl FnMut(std::any::TypeId));
    fn outgoing<T: Component<S>>(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn incoming<T: Component<S>>(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
}
impl<S, C> ConnectionExecution<S> for C
where
    S: ApplicationSchema,
    C: ApplicationConnectionShape<S>,
    Binding<S, C>: WorthQueryApplicationPrerequisiteOutputConnection<S>,
    Parent<S, C>: Clone + Send,
    Child<S, C>: Clone + Send,
    <Discovery<S, C> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Discovery<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
    <Prerequisites<S, C> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <Prerequisites<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    DeclaredRow<S, Discovery<S, C>>:
        WorthQueryApplicationProjection<S, <Discovery<S, C> as ApplicationQueryBinding<S>>::Query>,
    DeclaredRow<S, Prerequisites<S, C>>: WorthQueryApplicationProjection<
        S,
        <Prerequisites<S, C> as ApplicationQueryBinding<S>>::Query,
    >,
    Row<S, C>: WorthQueryApplicationProjection<
        S,
        <Prerequisites<S, C> as ApplicationQueryBinding<S>>::Query,
    >,
    <Binding<S, C> as WorthQueryApplicationPrerequisiteOutputConnection<S>>::Selection:
        WorthQueryPrerequisiteSelection<
            S,
            <Prerequisites<S, C> as ApplicationQueryBinding<S>>::Query,
        >,
    source::SourceInput<S, Parent<S, C>>:
        ApplicationQueryIntent<S, Binding = source::Source<S, Parent<S, C>>>,
    source::SourceInput<S, Child<S, C>>:
        ApplicationQueryIntent<S, Binding = source::Source<S, Child<S, C>>>,
    <source::Source<S, Parent<S, C>> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <source::Source<S, Parent<S, C>> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
    <source::Source<S, Child<S, C>> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <source::Source<S, Child<S, C>> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
{
    fn families(visit: &mut impl FnMut(std::any::TypeId)) {
        visit(
            crate::equality_type_id::EqualityTypeId::of::<
                <Parent<S, C> as WorthQueryApplicationOutputDemand<S>>::OutputFamily,
            >()
            .into_external(),
        );
        visit(
            crate::equality_type_id::EqualityTypeId::of::<
                <Child<S, C> as WorthQueryApplicationOutputDemand<S>>::OutputFamily,
            >()
            .into_external(),
        );
    }
    fn outgoing<T: Component<S>>(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        let Some(demand) = demand.downcast_ref::<Parent<S, C>>() else {
            return Ok(());
        };
        let connection = Binding::<S, C>::IDENTITY;
        let intent = Binding::<S, C>::discovery_from_root(demand).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::from(
                super::super::WorthQueryOutputDemandDenial::discovery_connection_denied(
                    connection, cause,
                ),
            ))
        })?;
        let rows = context.discover::<Discovery<S, C>>(parent, intent)?;
        let children = Binding::<S, C>::demands_from_discovery(&rows).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::from(
                super::super::WorthQueryOutputDemandDenial::discovery_connection_denied(
                    connection, cause,
                ),
            ))
        })?;
        for child in children {
            capture::schedule::<S, Child<S, C>, ApplicationOutputOccurrences<T>>(
                context,
                next,
                connection,
                C::declaration(),
                child,
            )?;
        }
        Ok(())
    }
    fn incoming<T: Component<S>>(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        let Some(demand) = demand.downcast_ref::<Child<S, C>>() else {
            return Ok(());
        };
        let connection = Binding::<S, C>::IDENTITY;
        let intent = Binding::<S, C>::prerequisites_from_target(demand).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::from(
                super::super::WorthQueryOutputDemandDenial::discovery_connection_denied(
                    connection, cause,
                ),
            ))
        })?;
        let rows = context
            .discover_as_at::<Prerequisites<S, C>, PrerequisiteRows<S, C>>(pin, source, intent)?;
        for row in rows.0 {
            let parent = Binding::<S, C>::parent_from_prerequisite(&row).map_err(|cause| {
                SynchronousOutputFailure(CoupledSourceReadStop::from(
                    super::super::WorthQueryOutputDemandDenial::discovery_connection_denied(
                        connection, cause,
                    ),
                ))
            })?;
            capture::schedule::<S, Parent<S, C>, ApplicationOutputOccurrences<T>>(
                context,
                next,
                connection,
                C::declaration(),
                parent,
            )?;
        }
        Ok(())
    }
}
trait Component<S: ApplicationSchema>:
    worth_query_declaration::facade::application_program::ApplicationOccurrenceConnections<S>
{
    fn families(visit: &mut impl FnMut(std::any::TypeId));
    fn outgoing(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
    fn incoming(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure>;
}
macro_rules! tuple {
    ($($c:ident),+) => {
        impl<S:ApplicationSchema,$($c:ConnectionExecution<S>),+> Component<S> for ($($c,)+) {
            fn families(visit:&mut impl FnMut(std::any::TypeId)) { $($c::families(visit);)+ }
            fn outgoing(context:&SynchronousOutputAssembly<'_, '_, '_, '_, S>,parent:&AppliedOutputParent,demand:&dyn Any,next:&mut SynchronousOutputWave<S>)->Result<(),SynchronousOutputFailure> { $($c::outgoing::<Self>(context,parent,demand,next)?;)+ Ok(()) }
            fn incoming(context:&SynchronousOutputAssembly<'_, '_, '_, '_, S>,pin:&RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,source:worth_relational::facade::identity::EntityId,demand:&dyn Any,next:&mut SynchronousOutputWave<S>)->Result<(),SynchronousOutputFailure> { $($c::incoming::<Self>(context,pin,source,demand,next)?;)+ Ok(()) }
        }
    }
}
tuple!(A);
tuple!(A, B);
tuple!(A, B, C);
tuple!(A, B, C, D);
tuple!(A, B, C, D, E);
tuple!(A, B, C, D, E, F);
tuple!(A, B, C, D, E, F, G);
tuple!(A, B, C, D, E, F, G, H);
impl<T> sealed::Edges for ApplicationOutputOccurrences<T> {}
impl<S: ApplicationSchema, D: WorthQueryApplicationOutputDemand<S>, T: Component<S>>
    SynchronousOutputEdges<S, D> for ApplicationOutputOccurrences<T>
{
    fn occurrence_families(visit: &mut impl FnMut(std::any::TypeId)) {
        T::families(visit);
    }
    fn occurrence_component() -> bool {
        true
    }
    fn declared_families(
        _: std::any::TypeId,
        _: &mut impl FnMut(std::any::TypeId, std::any::TypeId),
    ) { /* Internal arcs have actual occurrence identities, not a declaration DAG. */
    }
    fn prerequisites(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        T::incoming(context, pin, source, demand, next)
    }
    fn assemble(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        parent: &AppliedOutputParent,
        demand: &D,
        next: &mut SynchronousOutputWave<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        T::outgoing(context, parent, demand, next)
    }
}

//! Stored incoming queries retain their complete selected rows and support.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationStoredPrerequisites;

type Prerequisites<S, C> = <<Binding<S, C> as WorthQueryApplicationPrerequisiteOutputConnection<
    S,
>>::Prerequisites as ApplicationQueryIntent<S>>::Binding;
type Row<S,C> = <<Binding<S,C> as WorthQueryApplicationPrerequisiteOutputConnection<S>>::PrerequisiteRow as ApplicationStructuredValueBinding>::Value;

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

pub(super) trait PrerequisiteExecution<S: ApplicationSchema, C: ApplicationConnectionShape<S>> {
    fn incoming(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
        whole: &'static ComponentExecution<S>,
    ) -> Result<(), SynchronousOutputFailure>;
}
impl<S, C> PrerequisiteExecution<S, C> for ApplicationStoredPrerequisites
where
    S: ApplicationSchema,
    C: ApplicationConnectionShape<S>,
    Binding<S, C>: WorthQueryApplicationPrerequisiteOutputConnection<S>,
    Parent<S, C>: Clone + Send,
    <Prerequisites<S, C> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <Prerequisites<S, C> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
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
    <source::Source<S, Parent<S, C>> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <source::Source<S, Parent<S, C>> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
{
    fn incoming(
        context: &SynchronousOutputAssembly<'_, '_, '_, '_, S>,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        source: worth_relational::facade::identity::EntityId,
        demand: &dyn Any,
        next: &mut SynchronousOutputWave<S>,
        whole: &'static ComponentExecution<S>,
    ) -> Result<(), SynchronousOutputFailure> {
        let Some(demand) = demand.downcast_ref::<Child<S, C>>() else {
            return Ok(());
        };
        let connection = Binding::<S, C>::IDENTITY;
        let intent = Binding::<S, C>::prerequisites_from_target(demand).map_err(|cause| {
            SynchronousOutputFailure(CoupledSourceReadStop::from(
                super::super::super::WorthQueryOutputDemandDenial::discovery_connection_denied(
                    connection, cause,
                ),
            ))
        })?;
        let rows = context.discover_as_at::<Prerequisites<S, C>, PrerequisiteRows<S, C>>(
            pin,
            source,
            intent,
            crate::domain_computation::primary_graph::provider::CandidateReadSupport::Query,
        )?;
        for row in rows.0 {
            let parent = Binding::<S, C>::parent_from_prerequisite(&row).map_err(|cause| {
                SynchronousOutputFailure(CoupledSourceReadStop::from(
                    super::super::super::WorthQueryOutputDemandDenial::discovery_connection_denied(
                        connection, cause,
                    ),
                ))
            })?;
            capture::schedule_continued(
                context,
                next,
                connection,
                C::declaration(),
                parent,
                whole,
            )?;
        }
        Ok(())
    }
}

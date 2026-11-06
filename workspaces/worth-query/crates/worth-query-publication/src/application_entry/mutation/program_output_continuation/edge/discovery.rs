//! Discovery retains its exact Native kind and original C-query description.
use super::*;

pub(super) enum EdgeDiscoveryCustody<Q, R> {
    Ordinary(worth_query_execution::facade::primary_graph::WorthQueryApplicationOutputDemandSource<Q, R>),
    History(worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputDiscovery<Q, R>),
}
impl<Q, R> EdgeDiscoveryCustody<Q, R> {
    pub(super) fn rows(&self) -> &[R] {
        match self {
            Self::Ordinary(source) => source.rows(),
            Self::History(source) => source.rows(),
        }
    }
}
impl<Schema, Program, ParentDemand, Connection, Children>
    EdgeContinuation<Schema, Program, ParentDemand, Connection, Children>
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
    pub(super) fn read_discovery(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        history: Option<ProgramOutputHistoryFrame<'_, '_, '_>>,
    ) -> Result<EdgeDiscoveryCustody<<DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::Query, DiscoveryValue<Schema, Connection>>, WorthQueryRequiredOutputPreparationDenial> {
        let intent = Binding::<Schema, Connection>::discovery_from_root(&self.parent_demand)
            .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
        match (self.parent_authority.kind(), history) {
            (ParentAuthorityKind::Ordinary(_), None) => request.at(&self.basis).query(intent)
                .execute_in_advancement(phase).map(|result| EdgeDiscoveryCustody::Ordinary(result.into_output_demand_source()))
                .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery),
            (ParentAuthorityKind::History(parent), Some(history)) if history.authority.uses_owned_model_graph() => application
                .execute_owned_history_discovery::<ParentDemand, Connection>(
                    phase, &worth_query_execution::publication_boundary::program_publication_access(),
                    parent, history.authority, &self.basis.retained, &self.minimum_observation.retained,
                    request.principal, request.scope, intent,
                ).map(EdgeDiscoveryCustody::History).map_err(history_query_denial),
            (ParentAuthorityKind::History(parent), Some(history)) => request.query(intent)
                .with_admitted_query(phase, |_, binding, external, control| {
                    let read = application.execute_history_dependent_output_query::<ParentDemand, Connection, DiscoveryBinding<Schema, Connection>>(
                        phase, &worth_query_execution::publication_boundary::program_publication_access(),
                        parent, history.authority, &self.basis.retained, &self.minimum_observation.retained, binding, external, control,
                    ).map_err(crate::application_entry::WorthQueryApplicationRequestQueryDenial::HistoryOutput)?;
                    read.into_discovery().map(EdgeDiscoveryCustody::History)
                        .map_err(crate::application_entry::WorthQueryApplicationRequestQueryDenial::HistoryOutput)
                }).map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery),
            (ParentAuthorityKind::Verified(parent), Some(history)) => application
                .execute_verified_history_discovery::<ParentDemand, Connection>(
                    phase, &worth_query_execution::publication_boundary::program_publication_access(),
                    parent, history.authority, request.principal, request.scope, intent,
                ).map(EdgeDiscoveryCustody::History).map_err(history_query_denial),
            _ => Err(history_mismatch()),
        }
    }

    pub(super) fn retain_discovery(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        history: Option<ProgramOutputHistoryFrame<'_, '_, '_>>,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredOutputPreparationDenial> {
        let source = self.discovery_source.as_ref().expect("original discovery remains owned");
        match (source, self.parent_authority.kind(), history) {
            (EdgeDiscoveryCustody::Ordinary(source), ParentAuthorityKind::Ordinary(_), None) =>
                request.at(&self.basis).retain_query_dependencies(phase, source),
            (EdgeDiscoveryCustody::History(source), ParentAuthorityKind::History(parent), Some(history)) =>
                application.retain_history_discovery_dependencies::<ParentDemand, Connection>(
                    phase, &worth_query_execution::publication_boundary::program_publication_access(),
                    parent, history.authority, source, request.principal, request.scope,
                ).map_err(history_read_denial),
            (EdgeDiscoveryCustody::History(source), ParentAuthorityKind::Verified(parent), Some(history)) =>
                application.retain_verified_history_discovery_dependencies::<ParentDemand, Connection>(
                    phase, &worth_query_execution::publication_boundary::program_publication_access(),
                    parent, history.authority, source, request.principal, request.scope,
                ).map_err(history_read_denial),
            _ => Err(history_mismatch()),
        }
    }
}

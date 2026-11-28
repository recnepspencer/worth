//! Read discovery at the program basis and retain its dependencies.
use super::*;

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
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<worth_query_execution::facade::primary_graph::WorthQueryApplicationOutputDemandSource<<DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::Query, DiscoveryValue<Schema, Connection>>, WorthQueryRequiredOutputPreparationDenial> {
        let intent = Binding::<Schema, Connection>::discovery_from_root(&self.parent_demand)
            .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?;
        request.at(&self.basis).query(intent)
            .execute_in_advancement(phase)
            .map(|result| result.into_output_demand_source())
            .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)
    }

    pub(super) fn retain_discovery(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<WorthQueryRetainedQueryDependencies, WorthQueryRequiredOutputPreparationDenial> {
        let source = self.discovery_source.as_ref().expect("original discovery remains owned");
        request.at(&self.basis).retain_query_dependencies(phase, source)
    }
}

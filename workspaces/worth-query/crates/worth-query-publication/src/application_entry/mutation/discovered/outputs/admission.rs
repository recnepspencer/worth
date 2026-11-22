//! Incremental discovery and root admission at one retained source observation.
use super::*;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationOutputDemandSource, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryOutputSourceEvidence,
    WorthQueryPreparedRequiredOutputSource,
};

mod stages;

pub(super) type DiscoveryBinding<Schema, Root> =
    <Discovery<Schema, Root> as ApplicationQueryIntent<Schema>>::Binding;
pub(super) type DiscoveryQuery<Schema, Root> =
    <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
pub(super) type DiscoveryValue<Schema, Root> =
    <<DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type SourceInput<Schema, Root> =
    WorthQueryApplicationOutputDemandSource<Query<Schema, Root>, Value<Schema, Root>>;

struct RootInput<Schema, Root>
where
    Schema: ApplicationSchema,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    source: Option<SourceInput<Schema, Root>>,
    current: Option<SourceInput<Schema, Root>>,
    superseded: bool,
}

pub(super) struct RootAdmission<Schema, Root>
where
    Schema: ApplicationSchema,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    discovery: worth_query_execution::facade::primary_graph::WorthQueryOutputDiscoveryRecipe<Discovery<Schema,Root>>,
    discovery_source: Option<WorthQueryApplicationOutputDemandSource<DiscoveryQuery<Schema, Root>, DiscoveryValue<Schema, Root>>>,
    pub(super) dependencies: Option<worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies>,
    kind: DiscoveredRootStartKind,
    demands: Option<Vec<Demand<Schema, Root>>>,
    sources: Vec<SourceInput<Schema, Root>>,
    current: Vec<Option<SourceInput<Schema, Root>>>,
    stale: Vec<usize>,
    inputs: Vec<RootInput<Schema, Root>>,
    bound: bool,
    next: usize,
    first_denial: Option<WorthQueryOutputDemandDenial>,
}

impl<Schema, Root> RootAdmission<Schema, Root>
where
    Schema: ApplicationSchema,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    pub(super) fn new(
        discovery: impl Into<
            worth_query_execution::facade::primary_graph::WorthQueryOutputDiscoveryRecipe<
                Discovery<Schema, Root>,
            >,
        >,
        kind: DiscoveredRootStartKind,
    ) -> Self {
        Self {
            discovery: discovery.into(),
            discovery_source: None,
            dependencies: None,
            kind,
            demands: None,
            sources: Vec::new(),
            current: Vec::new(),
            stale: Vec::new(),
            inputs: Vec::new(),
            bound: false,
            next: 0,
            first_denial: None,
        }
    }

    pub(super) fn first_denial(&self) -> Option<&WorthQueryOutputDemandDenial> {
        self.first_denial.as_ref()
    }
}

impl<Schema, Root> RootAdmission<Schema, Root>
where
    Schema: ApplicationSchema + 'static,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    Demand<Schema, Root>: Clone,
    DiscoveryValue<Schema, Root>:
        WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
    <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Value<Schema, Root>: WorthQueryApplicationProjection<Schema, Query<Schema, Root>> + Clone,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, Root>>,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(super) fn admit<Program>(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        receipt: &WorthQueryOutputSourceEvidence,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        retained: &WorthQueryApplicationReadObservation,
        controls: WorthQueryOutputDemandControls,
        roots: &mut Vec<RootNode<Schema, Program, Root>>,
        superseded: &mut Vec<Demand<Schema, Root>>,
    ) -> Result<(), WorthQueryRequiredOutputPreparationDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        self.discover_demands(phase, application, request, retained)?;
        if !self.bound {
            self.bind_root_sources(phase, application, request, prepared, retained)?;
        }
        let demand_count = self
            .demands
            .as_ref()
            .expect("discovery retains its root list")
            .len();
        while self.next < demand_count {
            self.admit_next_root(
                phase,
                application,
                request,
                receipt,
                prepared,
                retained,
                controls,
                roots,
                superseded,
            )?;
        }
        Ok(())
    }
}

#[inline(never)]
fn read_source<Schema, Root>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    retained: &WorthQueryApplicationReadObservation,
    demand: &Demand<Schema, Root>,
) -> Result<SourceInput<Schema, Root>, WorthQueryRequiredOutputPreparationDenial>
where
    Schema: ApplicationSchema + 'static,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    Value<Schema, Root>: WorthQueryApplicationProjection<Schema, Query<Schema, Root>> + Clone,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, Root>>,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    let result = request
        .at(retained)
        .query(demand.source_intent())
        .execute_in_advancement(phase)
        .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
    if result.rows().len() != 1 || result.observed_sources().len() != 1 {
        return Err(WorthQueryRequiredOutputPreparationDenial::MissingSource);
    }
    Ok(result.into_output_demand_source())
}

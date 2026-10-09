//! Incremental discovery and root admission at one retained source observation.
use super::*;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationOutputDemandSource,
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryPreparedRequiredOutputSource,
};

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
    discovery: Discovery<Schema, Root>,
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
    pub(super) fn new(discovery: Discovery<Schema, Root>, kind: DiscoveredRootStartKind) -> Self {
        Self {
            discovery,
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
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        receipt: &WorthQueryApplicationCommitReceipt,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        retained: &WorthQueryApplicationReadObservation,
        controls: WorthQueryOutputDemandControls,
        roots: &mut Vec<RootNode<Schema, Program, Root>>,
        superseded: &mut Vec<Demand<Schema, Root>>,
    ) -> Result<(), WorthQueryRequiredOutputPreparationDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        if self.demands.is_none() {
            let discovered = request
                .at(retained)
                .query(self.discovery.clone())
                .execute()
                .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
            let row = discovered
                .rows()
                .first()
                .filter(|_| discovered.rows().len() == 1)
                .ok_or(WorthQueryRequiredOutputPreparationDenial::MissingSource)?;
            self.demands = Some(
                RootConnection::<Schema, Root>::demands_from_discovery(row)
                    .map_err(WorthQueryRequiredOutputPreparationDenial::Connection)?,
            );
        }
        let demands = self
            .demands
            .as_ref()
            .expect("discovery retains its root list");
        if !self.bound {
            while self.current.len() < demands.len() {
                let index = self.current.len();
                if self.sources.len() == index {
                    self.sources.push(read_source::<Schema, Root>(
                        request,
                        retained,
                        &demands[index],
                    )?);
                }
                let current = if matches!(self.kind, DiscoveredRootStartKind::Recovery) {
                    let result = request
                        .query(demands[index].source_intent())
                        .execute()
                        .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
                    let current = result.into_output_demand_source();
                    match application
                        .validate_recovered_discovered_program_root_currentness::<Root>(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        prepared,
                        &self.sources[index],
                        &current,
                    ) {
                        Ok(()) => {}
                        Err(denial)
                            if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded =>
                        {
                            self.stale.push(index);
                        }
                        Err(denial) => {
                            self.first_denial.get_or_insert_with(|| denial.clone());
                            return Err(
                                WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial),
                            );
                        }
                    }
                    Some(current)
                } else {
                    None
                };
                self.current.push(current);
            }
            application
                .bind_prepared_discovered_program_root_sources::<Root>(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    prepared,
                    &self.sources,
                )
                .map_err(|denial| {
                    self.first_denial.get_or_insert_with(|| denial.clone());
                    WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial)
                })?;
            self.inputs = std::mem::take(&mut self.sources)
                .into_iter()
                .zip(std::mem::take(&mut self.current))
                .enumerate()
                .map(|(index, (source, current))| RootInput {
                    source: Some(source),
                    current,
                    superseded: self.stale.contains(&index),
                })
                .collect();
            self.bound = true;
        }
        while self.next < demands.len() {
            let demand = demands[self.next].clone();
            let input = &mut self.inputs[self.next];
            if input.superseded {
                superseded.push(demand);
                self.next += 1;
                continue;
            }
            application
                .validate_discovered_program_source::<Root>(
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    prepared,
                    receipt,
                    &retained.retained,
                    request.principal,
                    request.scope,
                    request.branch,
                )
                .map_err(WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
            // Admission consumes a linear disclosure even on refusal. Only the
            // unadmitted root reacquires it, at the original retained observation.
            let source = match input.source.take() {
                Some(source) => source,
                None => read_source::<Schema, Root>(request, retained, &demand)?,
            };
            let limits = controls.resolve(application.runtime().output_demand_resource_profile());
            let admitted = match self.kind {
                DiscoveredRootStartKind::Performed => application
                    .admit_performed_discovered_program_root_output::<Root>(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        source,
                        limits,
                        prepared,
                    ),
                DiscoveredRootStartKind::Recovery => {
                    let current = match input.current.take() {
                        Some(current) => current,
                        None => request
                            .query(demand.source_intent())
                            .execute()
                            .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?
                            .into_output_demand_source(),
                    };
                    application.validate_recovered_discovered_program_root_currentness::<Root>(
                        &worth_query_execution::publication_boundary::program_publication_access(), prepared, &source, &current)
                        .and_then(|()| application.recover_discovered_program_root_output::<Root>(
                            &worth_query_execution::publication_boundary::program_publication_access(), source, current, limits, receipt))
                }
            };
            match admitted {
                Ok(admitted) => roots.push(RootNode {
                    demand: demand.clone(),
                    handle: Some(WorthQueryApplicationProgramDemandHandle::new(
                        admitted, demand, None,
                    )),
                    settlement: None,
                    pending_authority: None,
                    continuation: None,
                    outputs: Vec::new(),
                    work: ProgramOutputTraversalWork::default(),
                }),
                Err(denial) if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded => {
                    superseded.push(demand)
                }
                Err(denial) => {
                    self.first_denial.get_or_insert_with(|| denial.clone());
                    return Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(
                        denial,
                    ));
                }
            }
            self.next += 1;
        }
        Ok(())
    }
}

fn read_source<Schema, Root>(
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
        .execute()
        .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?;
    if result.rows().len() != 1 || result.observed_sources().len() != 1 {
        return Err(WorthQueryRequiredOutputPreparationDenial::MissingSource);
    }
    Ok(result.into_output_demand_source())
}

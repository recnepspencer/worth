//! Phase-isolated discovery, source-binding, and root-admission stages.
use super::*;

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
    #[inline(never)]
    pub(super) fn discover_demands<Program>(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        retained: &WorthQueryApplicationReadObservation,
    ) -> Result<(), WorthQueryRequiredOutputPreparationDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        if self.discovery_source.is_none() && self.dependencies.is_none() {
            let discovered = match self.discovery.live() {
                Some(live) => request
                    .at(retained)
                    .query(live.clone())
                    .execute_in_advancement(phase)
                    .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?
                    .into_output_demand_source(),
                None => application
                    .runtime()
                    .read_recorded_output_discovery::<DiscoveryBinding<Schema, Root>>(
                        phase,
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        &self.discovery,
                        request.principal,
                        request.scope,
                        &retained.retained,
                    )
                    .map_err(WorthQueryRequiredOutputPreparationDenial::DemandExecution)?,
            };
            self.discovery_source = Some(discovered);
        }
        if self.demands.is_none() {
            let discovered = self
                .discovery_source
                .as_ref()
                .expect("original discovery pairing remains owned");
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
        if self.dependencies.is_none() {
            let proof = request.at(retained).retain_query_dependencies(
                phase,
                self.discovery_source
                    .as_ref()
                    .expect("capture borrows original pairing"),
            )?;
            self.dependencies = Some(proof);
            self.discovery_source = None;
        }
        Ok(())
    }

    #[inline(never)]
    pub(super) fn bind_root_sources<Program>(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        prepared: &WorthQueryPreparedRequiredOutputSource,
        retained: &WorthQueryApplicationReadObservation,
    ) -> Result<(), WorthQueryRequiredOutputPreparationDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let demands = self
            .demands
            .as_ref()
            .expect("discovery retains its root list");
        while self.current.len() < demands.len() {
            let index = self.current.len();
            if self.sources.len() == index {
                self.sources.push(read_source::<Schema, Root>(
                    phase,
                    request,
                    retained,
                    &demands[index],
                )?);
            }
            let current = if matches!(self.kind, DiscoveredRootStartKind::Recovery) {
                let result = request
                    .query(demands[index].source_intent())
                    .execute_in_advancement(phase)
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
                        return Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(
                            denial,
                        ));
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
                phase,
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
        Ok(())
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn admit_next_root<Program>(
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
        let demand = self
            .demands
            .as_ref()
            .expect("discovery retains its root list")[self.next]
            .clone();
        let input = &mut self.inputs[self.next];
        if input.superseded {
            superseded.push(demand);
            self.next += 1;
            return Ok(());
        }
        application
            .validate_discovered_program_source::<Root>(
                phase,
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
            None => read_source::<Schema, Root>(phase, request, retained, &demand)?,
        };
        let limits = controls.resolve(application.runtime().output_demand_resource_profile());
        let admitted = match self.kind {
            DiscoveredRootStartKind::Performed => application
                .admit_performed_discovered_program_root_output::<Root>(
                    phase,
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    source,
                    limits,
                    prepared,
                    std::sync::Arc::clone(&retained.retained),
                ),
            DiscoveredRootStartKind::Recovery => {
                let current = match input.current.take() {
                    Some(current) => current,
                    None => request
                        .query(demand.source_intent())
                        .execute_in_advancement(phase)
                        .map_err(WorthQueryRequiredOutputPreparationDenial::SourceQuery)?
                        .into_output_demand_source(),
                };
                application
                    .validate_recovered_discovered_program_root_currentness::<Root>(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        prepared,
                        &source,
                        &current,
                    )
                    .and_then(|()| {
                        application.recover_discovered_program_root_output::<Root>(
                            phase,
                            &worth_query_execution::publication_boundary::program_publication_access(
                            ),
                            source,
                            current,
                            limits,
                            receipt,
                        )
                    })
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
                dependencies: None,
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
        Ok(())
    }
}

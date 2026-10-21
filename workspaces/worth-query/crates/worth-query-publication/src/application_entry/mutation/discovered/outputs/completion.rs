use super::*;
impl<Schema, Program, Root> WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    Demand<Schema, Root>: Clone,
    Value<Schema, Root>: WorthQueryApplicationProjection<Schema, Query<Schema, Root>> + Clone,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, Root>>,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    DiscoveryValue<Schema, Root>:
        WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
    <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, Demand<Schema, Root>>,
{
    pub(super) fn complete_outputs(
        &mut self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputProgress<Query<Schema, Root>, Demand<Schema, Root>>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        let dependencies = request.at(&self.source).combine_query_dependencies(
            phase,
            std::iter::once(
                self.admission
                    .dependencies
                    .as_ref()
                    .expect("root admission retains mandatory discovery"),
            )
            .chain(
                self.roots
                    .iter()
                    .filter_map(|root| root.dependencies.as_ref()),
            ),
        )?;
        self.admission.dependencies = None;
        application.complete_program_output_source(
            &worth_query_execution::publication_boundary::program_publication_access(),
            &self.source_receipt,
        );
        self.source_lease.take();
        self.complete = true;
        let settled_count = self
            .roots
            .iter()
            .filter(|root| root.settlement.is_some())
            .count();
        let mut traversal = ProgramOutputTraversalWork::discovered(1, settled_count);
        let mut roots = Vec::with_capacity(settled_count);
        let mut outputs = Vec::new();
        for mut root in std::mem::take(&mut self.roots) {
            traversal.include(root.work);
            if let Some(settlement) = root.settlement.take() {
                roots.push((root.demand, settlement));
            }
            outputs.append(&mut root.outputs);
        }
        let work = crate::application_entry::mutation::WorthQueryApplicationProgramWork::from_all_settlements(
            traversal,
            roots.iter().map(|(_, settled)| (
                settled.application_commit_receipt(),
                settled.readiness_delivery(),
            ))
                .chain(outputs.iter().map(|output| (
                    output.receipt(),
                    output.readiness_delivery(),
                ))),
        );
        Ok(WorthQueryDiscoveredProgramOutputProgress::Settled(
            WorthQueryDiscoveredProgramOutputSettlement::new(
                self.source.retained_clone(),
                roots,
                std::mem::take(&mut self.superseded),
                outputs,
                work,
                dependencies,
            ),
        ))
    }
}

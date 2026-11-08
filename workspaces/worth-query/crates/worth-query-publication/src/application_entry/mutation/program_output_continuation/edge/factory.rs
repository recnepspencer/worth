//! Typed edge construction and root recipe enrollment.
use super::*;

impl< Schema, Program, ParentDemand, Connection, Children>
    ProgramOutputContinuationFactory<Schema, Program, ParentDemand>
    for ApplicationOutputEdge<Connection, Children>
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
    fn validate_history_group_outputs(
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<(), worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial> {
        application.validate_history_group_output::<ChildDemand<Schema, Connection>>(
            &worth_query_execution::publication_boundary::program_publication_access(),
        )?;
        Children::validate_history_group_outputs(application)
    }

    fn capture_history_root_recipes<H: worth_query_declaration::facade::application_history::ApplicationHistoryBinding<Schema>>(
        document: &worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistory<'_, Schema, H>,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        profile: &worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputProfile,
        parent_demand: &ParentDemand,
    ) -> Result<(), crate::application_entry::WorthQueryHistoryOutputDenial> {
        let _ = application;
        let intent = Binding::<Schema, Connection>::discovery_from_root(parent_demand)
            .map_err(|cause| crate::application_entry::WorthQueryHistoryOutputDenial::Preparation(WorthQueryRequiredOutputPreparationDenial::Connection(cause)))?;
        document.capture_history_root_discovery_recipe::<Connection, DiscoveryBinding<Schema, Connection>>(
            phase, &worth_query_execution::publication_boundary::program_publication_access(), profile, intent,
        ).map_err(crate::application_entry::WorthQueryHistoryOutputDenial::Source)
    }

    fn start_verified(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>, parent_demand: &ParentDemand,
        parent: &worth_query_execution::publication_boundary::document_history::WorthQueryVerifiedHistoryParent<Schema, Program, ParentDemand>,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, controls: WorthQueryOutputDemandControls,
    ) -> Result<Box<dyn ProgramOutputContinuation<Schema, Program>>, WorthQueryRequiredOutputPreparationDenial> {
        // The Query-owned parent cannot be cloned into a different posture.
        // Clone the SDK carrier only after Query shares this exact read custody.
        Ok(Box::new(EdgeContinuation::<Schema, Program, ParentDemand, Connection, Children> {
            recording_recipes: None,
            parent_demand: parent_demand.clone(),
            parent_authority: ProgramOutputParentAuthority::verified(parent.clone()),
            nodes: None, discovery_source: None, dependencies: None, next_admission: 0, next_advance: 0,
            outputs: Vec::new(), work: ProgramOutputTraversalWork::default(), controls,
            basis: crate::application_entry::WorthQueryApplicationReadObservation::new(std::sync::Arc::clone(parent.observation())),
            minimum_observation: crate::application_entry::WorthQueryApplicationReadObservation::new(std::sync::Arc::clone(parent.observation())),
            marker: std::marker::PhantomData,
        }))
    }

    fn start(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        parent_settlement: &WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, ParentDemand>>,
        parent_authority: &ProgramOutputParentAuthority<Schema, Program, ParentDemand>,
        minimum_observation: &crate::application_entry::WorthQueryApplicationReadObservation,
        _: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<Box<dyn ProgramOutputContinuation<Schema, Program>>, WorthQueryRequiredOutputPreparationDenial> {
        let basis = if parent_settlement.observation().selected_commit().ordinal()
            >= minimum_observation.selected_commit().ordinal() {
            parent_settlement.observation()
        } else { minimum_observation };
        // Setup is effect-free. Discovery/admission belongs to the retained state,
        // so a later sibling refusal cannot drop an earlier admitted child.
        Ok(Box::new(EdgeContinuation::<Schema, Program, ParentDemand, Connection, Children> {
            recording_recipes: None,
            parent_demand: parent_demand.clone(),
            parent_authority: parent_authority.clone(),
            nodes: None,
            discovery_source: None,
            dependencies: None,
            next_admission: 0,
            next_advance: 0,
            outputs: Vec::new(),
            work: ProgramOutputTraversalWork::default(),
            controls,
            basis: basis.retained_clone(),
            minimum_observation: minimum_observation.retained_clone(),
            marker: std::marker::PhantomData,
        }))
    }
}

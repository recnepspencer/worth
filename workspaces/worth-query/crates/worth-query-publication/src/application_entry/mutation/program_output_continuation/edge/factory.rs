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



    fn start(
        _: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        parent_settlement: &WorthQueryApplicationOutputDemandSettlement<SourceQuery<Schema, ParentDemand>>,
        parent_authority: &std::sync::Arc<WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>>,
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

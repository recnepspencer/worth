//! The single private assembly of an admitted Home source and installed Query meaning.
use super::*;
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) struct HomeQueryPreparation<
    Schema: ApplicationSchemaComposition,
    Program,
> {
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) state:
        super::program_state::HomeServingProgramState<Schema, Program>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) bootstrap:
        WorthQueryPrimaryGraphBootstrap<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) services: crate::domain_computation::primary_graph::application_runtime::installation::home_attachment::HomeApplicationServices<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) producers: crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationProducerRegistry<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) conditionals: crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledApplicationConditionalRegistry<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) producer_routes: crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledOutputProducerRoutes,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) definition_routes: crate::domain_computation::primary_graph::conditional_operation::definition_declarations::ConditionalDefinitionDeclarations,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) readiness_routes: crate::domain_computation::primary_graph::application_contribution::WorthQueryInstalledOutputReadinessRoutes,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) bridge:
        super::super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
}
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn prepare<
    Schema: ApplicationSchemaComposition,
    Program,
>(
    mut state: HomeApplicationState<Schema, Program>,
    schema: InstalledNativeSchemaMeaning,
    restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    owner: WorthQueryRelationalSourceOwner,
) -> (
    HomeApplicationState<Schema, Program>,
    WorthQueryPrimaryGraphBootstrap<Schema>,
) {
    let mut graph = super::super::super::super::WorthQueryPrimaryGraph::from_installed_source(
        state.runtime.authority_identity(),
        state.installed.binding_identity(),
        Arc::new(schema.layout),
        owner,
    );
    graph.application_publication = Some(Arc::new(
        super::super::super::super::root::application_publication::ApplicationPublication::installed(
            &schema.receipt,
        ),
    ));
    let meaning =
        super::super::super::super::bootstrap::prepared_native::PreparedPrimaryGraphMeaning {
            resource_support:
                super::super::super::super::provider::WorthQueryPrimaryGraphResourceSupport::install(
                    state
                        .runtime
                        .application_query_resource_profile()
                        .maximum_concurrent_graph_work(),
                ),
            recovered_publication: None,
            recovered_relational_authority: None,
            recovered_checkpoint_restore_work: restore_work,
            invariant_installation_receipt: schema.receipt,
            expected_invariant_inventory_digest: schema.expected_invariant_inventory_digest,
            typed_rows: Default::default(),
            source_initializer: schema.source_initializer,
        };
    let mut bootstrap = meaning.into_bootstrap(
        &state.runtime,
        &state.installed,
        graph,
        state.limits.world.clone(),
    );
    bootstrap.mutation_handlers = std::mem::take(&mut state.pending.handlers);
    bootstrap.resources = std::mem::take(&mut state.pending.resources);
    bootstrap.managed_indexes = std::mem::take(&mut state.pending.managed_indexes);
    bootstrap.program_activation = Some(state.activation.clone());
    (state, bootstrap)
}

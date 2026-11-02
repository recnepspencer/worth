//! The single private assembly of an admitted Home source and installed Query meaning.
use super::*;
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) struct HomeQueryPreparation<
    Schema: ApplicationSchemaComposition,
    Program,
> {
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) state:
        HomeApplicationState<Schema, Program>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) bootstrap:
        WorthQueryPrimaryGraphBootstrap<Schema>,
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) bridge:
        super::super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
}
pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn assemble<
    Schema: ApplicationSchemaComposition,
    Program,
>(
    mut state: HomeApplicationState<Schema, Program>,
    schema: InstalledNativeSchemaMeaning,
    restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
    owner: WorthQueryRelationalSourceOwner,
    bridge: RuntimeBridge,
    conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
) -> HomeQueryPreparation<Schema, Program> {
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
    HomeQueryPreparation { state, bootstrap, bridge: super::super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge::attach_home_bridge(bridge, conditional) }
}

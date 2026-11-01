//! Finished installed meaning attaches only Store's actual registered source.
use super::super::super::{
    WorthQueryBootstrapAdvancementPhase, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::*;
use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;
use std::sync::{Arc, RwLock};
use worth_runtime_bridge::facade::{
    BridgeSealedRuntimeAssembly, RuntimeBridge, RuntimeBridgeRelationalSource,
};

pub struct PreparedHomeQueryRuntime<Schema: ApplicationSchemaComposition, Program> {
    pub(super) state: HomeApplicationState<Schema, Program>,
    pub(super) publication: Option<super::super::super::WorthQueryPrimaryGraphPublication>,
    pub(super) bootstrap: WorthQueryPrimaryGraphBootstrap<Schema>,
    pub(super) bridge: super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge,
}

pub struct QueryHomeSourceAttachmentRefused<Schema: ApplicationSchemaComposition, Program> {
    _prepared: ReadmittedHomeNativeMeaning<Schema, Program>,
    _source: RuntimeBridgeRelationalSource,
    _bridge: RuntimeBridge,
    _conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryHomeSourceAttachmentRefused<Schema, Program>
{
    pub fn reason(&self) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
    }
}

impl<Schema: ApplicationSchemaComposition, Program> ReadmittedHomeNativeMeaning<Schema, Program> {
    pub fn attach_home_registered_source(
        self,
        _phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        source: RuntimeBridgeRelationalSource,
        bridge: RuntimeBridge,
        conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    ) -> Result<
        PreparedHomeQueryRuntime<Schema, Program>,
        QueryHomeSourceAttachmentRefused<Schema, Program>,
    > {
        let result = (|| {
            let profile = source.authoritative_source_profile();
            if bridge.authoritative_source_profile() != Some(&profile)
                || !conditional
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .readmits_authoritative_source_profile(&profile)
            {
                return Err(home_source_denial(
                    "Home Bridge does not admit its registered Native source",
                ));
            }
            WorthQueryRelationalSourceOwner::attach_home_source(
                source.clone(),
                &self.schema.receipt,
                self.state.limits.world.invalidation_resources(),
            )
            .map_err(|cause| {
                home_source_denial(format!("Home Native source attachment denied: {cause:?}"))
            })
        })();
        let owner = match result {
            Ok(owner) => owner,
            Err(cause) => {
                return Err(QueryHomeSourceAttachmentRefused {
                    _prepared: self,
                    _source: source,
                    _bridge: bridge,
                    _conditional: conditional,
                    cause,
                })
            }
        };
        let Self {
            mut state,
            schema,
            restore_work,
        } = self;
        let mut graph = super::super::super::WorthQueryPrimaryGraph::from_installed_source(
            state.runtime.authority_identity(),
            state.installed.binding_identity(),
            Arc::new(schema.layout),
            owner,
        );
        graph.application_publication = Some(Arc::new(
            super::super::super::root::application_publication::ApplicationPublication::installed(
                &schema.receipt,
            ),
        ));
        let meaning =
            super::super::super::bootstrap::prepared_native::PreparedPrimaryGraphMeaning {
                resource_support:
                    super::super::super::provider::WorthQueryPrimaryGraphResourceSupport::install(
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
        bootstrap.program_activation_seed = Some(
            super::super::super::bootstrap::WorthQueryProgramActivationSeed::for_initial_program(
                &state.support.initial_revision,
                state.activation.clone(),
            ),
        );
        Ok(PreparedHomeQueryRuntime { state, bootstrap, publication: None,
            bridge: super::super::super::managed_bridge::WorthQueryInstalledApplicationBridge::attach_home_bridge(bridge, conditional) })
    }
}

fn home_source_denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(
        WorthQueryPrimaryGraphInstallationDenialKind::RuntimeBridgeRejected,
        subject,
    )
}

#[derive(Debug)]
pub enum QueryHomeInitialStateDenial {
    Callback(WorthQueryPrimaryGraphInstallationDenial),
    Panicked,
}
impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeQueryRuntime<Schema, Program> {
    /// Store already consumed the actual new-Home opportunity. Only the user
    /// callback unwinds inside this catch; Home and Query preparation stay out.
    pub fn initialize_home_query_state(
        &mut self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        initial_state: impl FnOnce(
            &WorthQueryBootstrapAdvancementPhase<'_>,
            &mut WorthQueryPrimaryGraphBootstrap<Schema>,
            &WorthQueryInstalledApplicationSchema<Schema>,
        ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial>,
    ) -> Result<(), QueryHomeInitialStateDenial> {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            initial_state(phase, &mut self.bootstrap, &self.state.installed)
        }))
        .map_err(|_| QueryHomeInitialStateDenial::Panicked)?
        .map_err(QueryHomeInitialStateDenial::Callback)
    }

    pub fn publish_home_query_seed(
        &mut self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if self.publication.is_some() {
            return Ok(());
        }
        self.publication = Some(super::super::super::bootstrap::home_publication::publish(
            &mut self.bootstrap,
            phase,
            || {
                self.state
                    .admitted_home_binding()
                    .map(|admitted| admitted.consume(|owner| owner.into_description()))
                    .map_err(|cause| {
                        home_source_denial(format!("Home completion binding denied: {cause:?}"))
                    })
            },
        )?);
        Ok(())
    }
}

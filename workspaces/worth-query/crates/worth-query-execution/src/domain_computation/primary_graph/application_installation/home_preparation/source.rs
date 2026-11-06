//! Finished installed meaning attaches only Store's actual registered source.
use super::super::super::{
    WorthQueryBootstrapAdvancementPhase, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::*;
pub(super) mod assembly;
use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;
use std::sync::{Arc, RwLock};
use worth_runtime_bridge::facade::{
    BridgeSealedRuntimeAssembly, RuntimeBridge, RuntimeBridgeRelationalSource,
};

pub struct PreparedHomeQueryRuntime<Schema: ApplicationSchemaComposition, Program> {
    pub(super) prepared: assembly::HomeQueryPreparation<Schema, Program>,
    pub(super) publication: Option<super::super::super::WorthQueryPrimaryGraphPublication>,
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
        let result =
            admit_registered_source(&self.state, &self.schema, &source, &bridge, &conditional);
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
            state,
            schema,
            restore_work,
            activation_seed,
        } = self;
        let mut prepared =
            assembly::assemble(state, schema, restore_work, owner, bridge, conditional);
        prepared.bootstrap.program_activation_seed = Some(activation_seed);
        Ok(PreparedHomeQueryRuntime {
            prepared,
            publication: None,
        })
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
            initial_state(
                phase,
                &mut self.prepared.bootstrap,
                &self.prepared.state.installed,
            )
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
            &mut self.prepared.bootstrap,
            phase,
        )?);
        Ok(())
    }
}

pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn admit_registered_source<
    Schema: ApplicationSchemaComposition,
    Program,
>(
    state: &HomeApplicationState<Schema, Program>,
    schema: &InstalledNativeSchemaMeaning,
    source: &RuntimeBridgeRelationalSource,
    bridge: &RuntimeBridge,
    conditional: &Arc<RwLock<BridgeSealedRuntimeAssembly>>,
) -> Result<WorthQueryRelationalSourceOwner, WorthQueryPrimaryGraphInstallationDenial> {
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
        &schema.receipt,
        Arc::new(
            super::super::super::SourceInvalidationOwner::with_managed_views(
                state.limits.world.invalidation_resources(),
                schema.receipt.runtime_instance_id(),
                Arc::clone(&schema.managed_views),
            ),
        ),
    )
    .map_err(|cause| home_source_denial(format!("Home Native source attachment denied: {cause:?}")))
}

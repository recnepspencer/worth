//! Finished installed meaning attaches only Store's actual registered source.
use super::super::super::{
    WorthQueryBootstrapAdvancementPhase, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::*;
pub(super) mod assembly;
mod authored_bindings;
mod program_installation;
mod program_preparation_custody;
mod program_preparation_denial;
pub use program_preparation_denial::QueryHomeSignalProgramPreparationDenial;
pub(super) mod program_state;
pub use program_preparation_custody::QueryHomeSignalProgramPreparationRefused;
mod program_handoff;
use crate::domain_computation::execution_runtime::product_world::WorthQueryRelationalSourceOwner;
pub use authored_bindings::QueryHomeAuthoredBindings;
pub use program_handoff::{PreparedExistingHomeSignalProgram, PreparedHomeSignalProgram};
use std::sync::Arc;
use worth_runtime_bridge::facade::{
    BridgeReadmittedAuthoredAssembly, RuntimeBridge, RuntimeBridgeRelationalSource,
};

pub struct PreparedHomeQueryRuntime<Schema: ApplicationSchemaComposition, Program> {
    pub(super) prepared: assembly::HomeQueryPreparation<Schema, Program>,
    pub(super) publication: Option<super::super::super::WorthQueryPrimaryGraphPublication>,
}

pub struct QueryHomeSourceAttachmentRefused<Schema: ApplicationSchemaComposition, Program> {
    _prepared: Box<program_installation::HomeServingProgramPreparation<Schema, Program>>,
    _assembly: Arc<BridgeReadmittedAuthoredAssembly>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryHomeSourceAttachmentRefused<Schema, Program>
{
    pub fn reason(&self) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
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
) -> Result<WorthQueryRelationalSourceOwner, WorthQueryPrimaryGraphInstallationDenial> {
    let profile = source.authoritative_source_profile();
    if bridge.authoritative_source_profile() != Some(&profile) {
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

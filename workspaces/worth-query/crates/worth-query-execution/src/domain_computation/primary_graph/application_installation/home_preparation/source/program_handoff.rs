//! The real fresh programme is handed to Signal before any World attachment.
use super::program_installation::{HomeServingProgramPreparation, HomeSignalProgramParts};
use super::*;
use worth_runtime_bridge::facade::{
    BridgeConditionalRuntimeBuilder, BridgeReadmittedAuthoredAssembly,
};
pub struct PreparedHomeSignalProgram<Schema: ApplicationSchemaComposition, Program> {
    pub(super) prepared: Box<HomeServingProgramPreparation<Schema, Program>>,
}
pub struct PreparedExistingHomeSignalProgram<Schema: ApplicationSchemaComposition, Program> {
    pub(super) prepared: Box<HomeServingProgramPreparation<Schema, Program>>,
}
impl<Schema: ApplicationSchemaComposition, Program> ReadmittedHomeNativeMeaning<Schema, Program> {
    pub fn prepare_home_signal_program(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        source: RuntimeBridgeRelationalSource,
        ordinary: RuntimeBridge,
    ) -> Result<
        (
            PreparedHomeSignalProgram<Schema, Program>,
            Box<BridgeConditionalRuntimeBuilder>,
            QueryHomeAuthoredBindings,
        ),
        super::program_preparation_custody::QueryHomeSignalProgramPreparationRefused<
            Schema,
            Program,
        >,
    > {
        let Self {
            state,
            schema,
            restore_work,
            activation_seed,
        } = self;
        let HomeSignalProgramParts {
            prepared,
            builder,
            bindings,
        } = super::program_installation::prepare(
            phase,
            Box::new(state),
            Box::new(schema),
            restore_work,
            source,
            ordinary,
            Some(activation_seed),
        )?;
        Ok((
            PreparedHomeSignalProgram {
                prepared: Box::new(prepared),
            },
            builder,
            bindings,
        ))
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    super::super::ReadmittedExistingHomeNativeMeaning<Schema, Program>
{
    pub fn prepare_existing_home_signal_program(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        source: RuntimeBridgeRelationalSource,
        ordinary: RuntimeBridge,
    ) -> Result<
        (
            PreparedExistingHomeSignalProgram<Schema, Program>,
            Box<BridgeConditionalRuntimeBuilder>,
            QueryHomeAuthoredBindings,
        ),
        super::program_preparation_custody::QueryHomeSignalProgramPreparationRefused<
            Schema,
            Program,
        >,
    > {
        let super::super::ReadmittedExistingHomeNativeMeaning {
            state,
            schema,
            restore_work,
        } = self;
        let HomeSignalProgramParts {
            prepared,
            builder,
            bindings,
        } = super::program_installation::prepare(
            phase,
            Box::new(state),
            Box::new(schema),
            restore_work,
            source,
            ordinary,
            None,
        )?;
        Ok((
            PreparedExistingHomeSignalProgram {
                prepared: Box::new(prepared),
            },
            builder,
            bindings,
        ))
    }
}
impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeSignalProgram<Schema, Program> {
    pub fn attach_home_authored_signal(
        self,
        assembly: Arc<BridgeReadmittedAuthoredAssembly>,
    ) -> Result<
        PreparedHomeQueryRuntime<Schema, Program>,
        QueryHomeSourceAttachmentRefused<Schema, Program>,
    > {
        attach(self.prepared, assembly).map(|prepared| PreparedHomeQueryRuntime {
            prepared,
            publication: None,
        })
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    PreparedExistingHomeSignalProgram<Schema, Program>
{
    pub fn attach_existing_home_authored_signal(
        self,
        assembly: Arc<BridgeReadmittedAuthoredAssembly>,
    ) -> Result<
        super::super::PreparedExistingHomeQueryRuntime<Schema, Program>,
        super::super::QueryExistingHomeSourceAttachmentRefused<Schema, Program>,
    > {
        attach(self.prepared, assembly)
            .map(super::super::PreparedExistingHomeQueryRuntime::from_authored_preparation)
            .map_err(|inner| super::super::QueryExistingHomeSourceAttachmentRefused { inner })
    }
}
fn attach<Schema: ApplicationSchemaComposition, Program>(
    prepared: Box<HomeServingProgramPreparation<Schema, Program>>,
    assembly: Arc<BridgeReadmittedAuthoredAssembly>,
) -> Result<
    assembly::HomeQueryPreparation<Schema, Program>,
    QueryHomeSourceAttachmentRefused<Schema, Program>,
> {
    if assembly
        .admit_programme_binding(&prepared.installation)
        .is_err()
    {
        return Err(QueryHomeSourceAttachmentRefused {
            _prepared: prepared,
            _assembly: assembly,
            cause: home_source_denial("Home Signal does not admit this exact installed programme"),
        });
    }
    let HomeServingProgramPreparation {
        state,
        bootstrap,
        services,
        ordinary,
        installation: _,
        producers,
        conditionals,
        producer_routes,
        readiness_routes,
        definition_routes,
    } = *prepared;
    Ok(assembly::HomeQueryPreparation {state,bootstrap,services,producers,conditionals,producer_routes,readiness_routes,definition_routes,bridge:crate::domain_computation::primary_graph::managed_bridge::WorthQueryInstalledApplicationBridge::attach_authored_home_bridge(ordinary,assembly)})
}

impl<Schema: ApplicationSchemaComposition, Program>
    PreparedExistingHomeSignalProgram<Schema, Program>
{
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn predecessor_program_for_rendering(
        &self,
        rendering: &worth_foundational::facade::AspectValue,
    ) -> Option<worth_query_declaration::facade::application_program::ApplicationProgramRevision>
    {
        self.prepared
            .state
            .support
            .roster
            .entries()
            .iter()
            .find(|entry| {
                super::super::super::super::program_occurrence::program_revision_rendering(
                    entry.revision(),
                ) == *rendering
            })
            .map(|entry| *entry.revision())
    }
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn predecessor_activation(
        &self,
    ) -> (
        &worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        &WorthQueryProgramActivationCell,
    ) {
        (
            &self.prepared.state.support.initial_revision,
            &self.prepared.state.activation,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_installation::home_preparation) fn predecessor_native_meaning(
        &self,
    ) -> (
        &super::super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
        &RelationalInitialSchemaInstallationReceipt,
    ) {
        (
            &self.prepared.bootstrap.graph.layout,
            self.prepared.bootstrap.installed_native_receipt(),
        )
    }
}

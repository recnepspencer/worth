//! Genuine installation, source seeding, authenticated reads and checkpoint recovery.
use super::{model::NestedValue, producer, query::ReadValue, schema::*};
use crate::document_retention_model::{host::host_limits, operator_identity::*};
use worth_query_host::facade::{
    application_contribution::*,
    application_entry::*,
    application_installation::*,
    declaration::{application_program::*, authentication::*},
    primary_graph::*,
};

pub use super::program::Program;
impl WorthQueryApplicationContribution<CanonicalSchema> for CanonicalContribution {
    type Configuration = ();
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CanonicalSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        contracts.producer::<producer::Producer>()?;
        contracts.conditional::<super::readiness::DerivedReadiness>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, CanonicalSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<producer::ProduceBinding, _>(producer::Handler)?;
        setup.handler::<super::edit::EditBinding, _>(super::edit::Handler)?;
        setup.producer::<producer::Producer>(producer::Provider)?;
        setup.conditional::<super::readiness::DerivedReadiness>(())
    }
}

pub type Host = WorthQueryProgramApplicationRuntime<CanonicalSchema, Program>;

pub fn install(value: NestedValue) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    in_memory_program(
        ApplicationProgramAuthoring::<CanonicalSchema, Program>::begin()
            .validated_program()
            .unwrap(),
        CanonicalSchema::declaration().unwrap(),
        ((),),
        limits(),
        |_, graph, installed| {
            let binding = installed
                .principal_binding(DocumentPrincipalBinding::reference())
                .unwrap();
            graph.bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("canonical-value-operator").unwrap(),
                1_u64,
                operator_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Document::reference(),
                    WorthQueryApplicationEntityKey::new("nested-row").unwrap(),
                )
                .field(
                    DocumentIdentityField::reference(),
                    "authored-nested".to_owned(),
                )
                .field(DocumentValueField::reference(), value),
            )?;
            Ok(())
        },
    )
}

pub fn restore(checkpoint: WorthQueryApplicationCheckpoint) -> Host {
    in_memory_program_from_checkpoint(
        ApplicationProgramAuthoring::<CanonicalSchema, Program>::begin()
            .validated_program()
            .unwrap(),
        CanonicalSchema::declaration().unwrap(),
        ((),),
        limits(),
        checkpoint,
    )
    .expect("current-format native checkpoint restores")
}

pub fn read(host: &Host) -> NestedValue {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(ReadValue {
            identity: "authored-nested".to_owned(),
        })
        .execute()
        .expect("ordinary authenticated query decodes the nested value")
        .rows()[0]
        .clone()
}

pub fn limits() -> WorthQueryInMemoryApplicationLimits {
    host_limits()
}
pub fn program() -> ValidatedApplicationProgram<CanonicalSchema, Program> {
    ApplicationProgramAuthoring::<CanonicalSchema, Program>::begin()
        .validated_program()
        .unwrap()
}

pub fn read_derived(host: &Host) -> super::output::DerivedValue {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .on_branch(host.runtime().current_world())
        .query(super::derived_query::ReadValue {
            identity: "derived-report".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}

/// Descriptive native record coordinates and generation; grants no authority.
pub fn source_id(host: &Host) -> worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts {
    let request = request_scope();
    let runtime = host.runtime();
    runtime
        .with_application_advancement(&request, |phase| {
            runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap()
                .resolve_entity(
                    &phase,
                    DocumentIdentityField::reference(),
                    "authored-nested".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
                .relational_record_identity_parts()
        })
        .expect("source identity resolves under the actual native request")
}

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
        contracts.producer::<producer::preserve::Producer>()?;
        contracts.conditional::<super::readiness::DerivedReadiness>()?;
        contracts.conditional::<super::readiness::preserve::PreserveReadiness>()?;
        Ok(())
    }
    fn configure(
        (): (),
        setup: &mut WorthQueryApplicationContributionSetup<'_, CanonicalSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        setup.handler::<producer::ProduceBinding, _>(producer::Handler)?;
        setup.handler::<producer::preserve::Binding, _>(producer::preserve::Handler)?;
        setup.handler::<super::edit::EditBinding, _>(super::edit::Handler)?;
        setup.producer::<producer::Producer>(producer::Provider)?;
        setup.producer::<producer::preserve::Producer>(producer::preserve::Provider)?;
        setup.conditional::<super::readiness::DerivedReadiness>(())?;
        setup.conditional::<super::readiness::preserve::PreserveReadiness>(())
    }
}

pub type Host = WorthQueryProgramApplicationRuntime<CanonicalSchema, Program>;

pub fn install(value: NestedValue) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    install_owned(value, |_| {})
}

#[allow(
    dead_code,
    reason = "V1 retains native projection authority; V2 compiles this same host fixture"
)]
pub fn install_with_projection(
    value: NestedValue,
) -> Result<
    (
        Host,
        WorthQueryApplicationInvariantProjectionAuthority<CanonicalSchema>,
    ),
    WorthQueryInMemoryApplicationDenial,
> {
    let mut projection = None;
    let host = install_owned(value, |graph| {
        projection = Some(graph.retain_invariant_projection_authority());
    })?;
    Ok((
        host,
        projection.expect("the actual bootstrap issued its retained projection authority"),
    ))
}

fn install_owned(
    value: NestedValue,
    retain: impl FnOnce(&WorthQueryPrimaryGraphBootstrap<CanonicalSchema>),
) -> Result<Host, WorthQueryInMemoryApplicationDenial> {
    in_memory_program(
        ApplicationProgramAuthoring::<CanonicalSchema, Program>::begin()
            .validated_program()
            .unwrap(),
        CanonicalSchema::declaration().unwrap(),
        ((),),
        limits(),
        |_, graph, installed| {
            retain(graph);
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

pub fn edit_source(
    host: &Host,
    value: NestedValue,
    idempotency: u64,
) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(super::edit::Input {
            identity: "authored-nested".to_owned(),
            value,
        })
        .without_source()
        .idempotency(&idempotency)
        .execute_performed::<Program, super::program::Root>(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the native program-owned source edit executes");
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the source edit must retain its required-output custody");
    };
    let controls = WorthQueryOutputDemandControls::new(
        std::num::NonZeroUsize::new(4096).unwrap(),
        std::num::NonZeroUsize::new(8192).unwrap(),
    );
    let mut started = performed
        .start_required_outputs(host, &request, controls)
        .unwrap_or_else(|_| panic!("the source edit must start its fixed required output"));
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = started
        .required_output_mut()
        .settle(host, &request)
        .unwrap()
    else {
        panic!("the source edit's real required-output producer must settle");
    };
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    settled
        .root_receipt()
        .expect("the real regeneration has a producer receipt")
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

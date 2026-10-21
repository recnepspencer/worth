//! Genuine installation, source seeding, authenticated reads and checkpoint recovery.
use super::{model::NestedValue, query::ReadValue, schema::*};
use crate::document_retention_model::{host::host_limits, operator_identity::*};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_contribution::*,
    application_entry::*,
    application_installation::*,
    declaration::{application_program::*, authentication::*},
    primary_graph::*,
};

pub struct Program;
pub struct Feature;
impl ApplicationFeature<CanonicalSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "canonical-nested.feature.v1";
}
impl ApplicationProgramDefinition<CanonicalSchema> for Program {
    type Contributions = (CanonicalContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("canonical-nested.program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![ApplicationFeatureSpec::root::<CanonicalSchema, Feature>().finish()]
    }
}
impl WorthQueryApplicationContribution<CanonicalSchema> for CanonicalContribution {
    type Configuration = ();
    fn configure(
        (): (),
        _setup: &mut WorthQueryApplicationContributionSetup<'_, CanonicalSchema>,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        Ok(())
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
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
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
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        checkpoint,
    )
    .expect("current-format native checkpoint restores")
}

pub fn read(host: &Host) -> NestedValue {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    host.runtime()
        .request(&principal, &scope)
        .query(ReadValue {
            identity: "authored-nested".to_owned(),
        })
        .execute()
        .expect("ordinary authenticated query decodes the nested value")
        .rows()[0]
        .clone()
}

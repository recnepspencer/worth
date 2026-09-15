use std::marker::PhantomData;

use worth_query_declaration::facade::application_program::{
    ApplicationConnectionDeclaration, ApplicationFeatureDeclaration, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramRuleDeclaration, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};

/// Failure to bind validated program meaning to its exact installed schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationProgramInstallationDenial {
    subject: String,
}

impl WorthQueryApplicationProgramInstallationDenial {
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryApplicationProgramInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application program installation denied: {}",
            self.subject
        )
    }
}

impl std::error::Error for WorthQueryApplicationProgramInstallationDenial {}

/// Installed program meaning affine to one schema installation.
pub struct WorthQueryInstalledApplicationProgram<Schema, Program> {
    identity: ApplicationProgramIdentity,
    schema_binding: ApplicationSchemaBindingIdentity,
    features: Box<[ApplicationFeatureDeclaration]>,
    connections: Box<[ApplicationConnectionDeclaration]>,
    rules: Box<[ApplicationProgramRuleDeclaration]>,
    marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> WorthQueryInstalledApplicationProgram<Schema, Program> {
    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }
    pub fn features(&self) -> &[ApplicationFeatureDeclaration] {
        &self.features
    }
    pub fn connections(&self) -> &[ApplicationConnectionDeclaration] {
        &self.connections
    }

    pub fn contains_connection(&self, identity: &str) -> bool {
        self.connections
            .iter()
            .any(|connection| connection.identity() == identity)
    }
    pub fn rules(&self) -> &[ApplicationProgramRuleDeclaration] {
        &self.rules
    }
}

pub fn install_application_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    installed_schema: &crate::facade::WorthQueryInstalledApplicationSchema<Schema>,
) -> Result<
    WorthQueryInstalledApplicationProgram<Schema, Program>,
    WorthQueryApplicationProgramInstallationDenial,
>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    if program.features().is_empty()
        || program.connections().is_empty()
        || program.rules().is_empty()
    {
        return Err(WorthQueryApplicationProgramInstallationDenial {
            subject: program.identity().as_str().to_owned(),
        });
    }
    for rule in program.rules() {
        let installed = installed_schema
            .invariants()
            .descriptors()
            .any(|candidate| {
                candidate.identifier() == rule.identity()
                    && candidate.major() == rule.major()
                    && candidate.minor() == rule.minor()
                    && candidate.execution_point() == rule.execution_point()
            });
        if !installed {
            return Err(WorthQueryApplicationProgramInstallationDenial {
                subject: rule.identity().to_owned(),
            });
        }
    }
    Ok(WorthQueryInstalledApplicationProgram {
        identity: program.identity().clone(),
        schema_binding: installed_schema.binding_identity(),
        features: program.features().to_vec().into_boxed_slice(),
        connections: program.connections().to_vec().into_boxed_slice(),
        rules: program.rules().to_vec().into_boxed_slice(),
        marker: PhantomData,
    })
}

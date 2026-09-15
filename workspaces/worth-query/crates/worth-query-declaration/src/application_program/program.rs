use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramConnectionSet, ApplicationProgramFeatureSet, ApplicationProgramIdentity,
    ApplicationProgramInventoryDeclaration, ApplicationProgramInventorySet,
    ApplicationProgramRuleDeclaration, ApplicationProgramRuleSet,
};

pub trait ApplicationProgramDefinition<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Contributions;
    type Features: ApplicationProgramFeatureSet<Schema>;
    type Connections: ApplicationProgramConnectionSet<Schema>;
    type Rules: ApplicationProgramRuleSet<Schema>;
    type Inventories: ApplicationProgramInventorySet<Schema>;

    const IDENTITY: ApplicationProgramIdentity;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationProgramValidationDenialKind {
    InvalidIdentity,
    EmptyProgram,
    DuplicateFeature,
    DuplicatePort,
    DuplicateConnection,
    DanglingFeature,
    DanglingPort,
    ForeignFeatureType,
    ForeignPortType,
    MissingRequiredInput,
    DuplicateInputBinding,
    UnsupportedFanIn,
    DuplicateRule,
    DuplicateInventory,
    DuplicateInventoryOutput,
    EmptyInventory,
    UnknownInventoryOutput,
    IncompleteInventory,
    OrphanFeature,
    AvailabilityMismatch,
    Cycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramValidationDenial {
    kind: ApplicationProgramValidationDenialKind,
    subject: String,
}

impl ApplicationProgramValidationDenial {
    pub const fn kind(&self) -> ApplicationProgramValidationDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub(in super::super) fn new(
        kind: ApplicationProgramValidationDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for ApplicationProgramValidationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application program denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for ApplicationProgramValidationDenial {}

pub struct ValidatedApplicationProgram<Schema, Program> {
    identity: ApplicationProgramIdentity,
    features: Box<[ApplicationFeatureDeclaration]>,
    connections: Box<[ApplicationConnectionDeclaration]>,
    rules: Box<[ApplicationProgramRuleDeclaration]>,
    inventories: Box<[ApplicationProgramInventoryDeclaration]>,
    marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> ValidatedApplicationProgram<Schema, Program> {
    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }

    pub fn features(&self) -> &[ApplicationFeatureDeclaration] {
        &self.features
    }

    pub fn connections(&self) -> &[ApplicationConnectionDeclaration] {
        &self.connections
    }

    pub fn rules(&self) -> &[ApplicationProgramRuleDeclaration] {
        &self.rules
    }

    pub fn inventories(&self) -> &[ApplicationProgramInventoryDeclaration] {
        &self.inventories
    }

    pub(in super::super) fn from_parts(
        identity: ApplicationProgramIdentity,
        features: Vec<ApplicationFeatureDeclaration>,
        connections: Vec<ApplicationConnectionDeclaration>,
        rules: Vec<ApplicationProgramRuleDeclaration>,
        inventories: Vec<ApplicationProgramInventoryDeclaration>,
    ) -> Self {
        Self {
            identity,
            features: features.into_boxed_slice(),
            connections: connections.into_boxed_slice(),
            rules: rules.into_boxed_slice(),
            inventories: inventories.into_boxed_slice(),
            marker: PhantomData,
        }
    }
}

/// Builds and validates the complete graph from the program's typed members.
///
/// A string cannot stand in for a typed connection.
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_program::*;
/// type Invalid = ApplicationProgramRequiredConnection<&'static str>;
/// ```
pub fn validate_application_program<Schema, Program>(
) -> Result<ValidatedApplicationProgram<Schema, Program>, ApplicationProgramValidationDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    super::validation::validate::<Schema, Program>()
}

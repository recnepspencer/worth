use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationConnectionDeclaration, ApplicationFeature, ApplicationFeatureDeclaration,
    ApplicationInputPort, ApplicationLocalRuleRef, ApplicationOccurrenceConnectionBinding,
    ApplicationOutputPort, ApplicationProgramIdentity, ApplicationProgramRuleDeclaration,
    ApplicationSharedRuleRef,
};

/// Complete authored static program definition.
pub trait ApplicationProgramDefinition<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    /// Exact root contribution tuple whose generated slots implement this
    /// program. Installation must consume this tuple before exposing a root.
    type Contributions;
    /// Typed execution connections lowered by the host audience.
    type Connections;
    type DependentConnection;
    const IDENTITY: ApplicationProgramIdentity;

    fn features() -> &'static [ApplicationFeatureDeclaration];
}

pub struct ApplicationProgramConnectionRequired;
pub struct ApplicationProgramDependentConnectionRequired;
pub struct ApplicationProgramLocalRuleRequired;
pub struct ApplicationProgramSharedRuleRequired;
pub struct ApplicationProgramComplete;

/// Phase 1 authoring progression. Validation is unavailable until its typed
/// required connection has been supplied.
pub struct ApplicationProgramAuthoring<
    Schema,
    Program,
    State = ApplicationProgramConnectionRequired,
> {
    connections: Vec<ApplicationConnectionDeclaration>,
    local_rule: Option<ApplicationProgramRuleDeclaration>,
    shared_rule: Option<ApplicationProgramRuleDeclaration>,
    marker: PhantomData<fn() -> (Schema, Program, State)>,
}

impl<Schema, Program>
    ApplicationProgramAuthoring<Schema, Program, ApplicationProgramConnectionRequired>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub const fn begin() -> Self {
        Self {
            connections: Vec::new(),
            local_rule: None,
            shared_rule: None,
            marker: PhantomData,
        }
    }

    pub fn connect<SourceFeature, SourcePort, TargetFeature, TargetPort>(
        self,
        connection: super::ApplicationConnectionRef<
            Schema,
            SourceFeature,
            SourcePort,
            TargetFeature,
            TargetPort,
            Program::Connections,
        >,
    ) -> ApplicationProgramAuthoring<Schema, Program, ApplicationProgramDependentConnectionRequired>
    where
        SourceFeature: ApplicationFeature<Schema>,
        TargetFeature: ApplicationFeature<Schema>,
        SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
        TargetPort: ApplicationInputPort<
            Schema,
            TargetFeature,
            Value = <SourcePort as ApplicationOutputPort<Schema, SourceFeature>>::Value,
        >,
        Program::Connections:
            ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>,
    {
        ApplicationProgramAuthoring {
            connections: vec![connection.into_declaration()],
            local_rule: None,
            shared_rule: None,
            marker: PhantomData,
        }
    }
}

impl<Schema, Program>
    ApplicationProgramAuthoring<Schema, Program, ApplicationProgramDependentConnectionRequired>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn connect_dependent<SourceFeature, SourcePort, TargetFeature, TargetPort>(
        mut self,
        connection: super::ApplicationConnectionRef<
            Schema,
            SourceFeature,
            SourcePort,
            TargetFeature,
            TargetPort,
            Program::DependentConnection,
        >,
    ) -> ApplicationProgramAuthoring<Schema, Program, ApplicationProgramLocalRuleRequired>
    where
        SourceFeature: ApplicationFeature<Schema>,
        TargetFeature: ApplicationFeature<Schema>,
        SourcePort: ApplicationOutputPort<Schema, SourceFeature>,
        TargetPort: ApplicationInputPort<
            Schema,
            TargetFeature,
            Value = <SourcePort as ApplicationOutputPort<Schema, SourceFeature>>::Value,
        >,
        Program::DependentConnection:
            ApplicationOccurrenceConnectionBinding<Schema, SourceFeature, TargetFeature>,
    {
        self.connections.push(connection.into_declaration());
        ApplicationProgramAuthoring {
            connections: self.connections,
            local_rule: None,
            shared_rule: None,
            marker: PhantomData,
        }
    }
}

impl<Schema, Program>
    ApplicationProgramAuthoring<Schema, Program, ApplicationProgramLocalRuleRequired>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn local_rule<Feature, Invariant>(
        self,
        rule: ApplicationLocalRuleRef<Schema, Feature, Invariant>,
        execution_point: crate::application_schema::ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramAuthoring<Schema, Program, ApplicationProgramSharedRuleRequired>
    where
        Feature: ApplicationFeature<Schema>,
        Invariant: crate::application_schema::ApplicationInvariantMarkerIdentity<Schema>,
    {
        ApplicationProgramAuthoring {
            connections: self.connections,
            local_rule: Some(rule.declaration(execution_point)),
            shared_rule: None,
            marker: PhantomData,
        }
    }
}

impl<Schema, Program>
    ApplicationProgramAuthoring<Schema, Program, ApplicationProgramSharedRuleRequired>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn shared_rule<Invariant>(
        self,
        rule: ApplicationSharedRuleRef<Schema, Invariant>,
        execution_point: crate::application_schema::ApplicationInvariantExecutionPoint,
    ) -> ApplicationProgramAuthoring<Schema, Program, ApplicationProgramComplete>
    where
        Invariant: crate::application_schema::ApplicationInvariantMarkerIdentity<Schema>,
    {
        ApplicationProgramAuthoring {
            connections: self.connections,
            local_rule: self.local_rule,
            shared_rule: Some(rule.declaration(execution_point)),
            marker: PhantomData,
        }
    }
}

impl<Schema, Program> ApplicationProgramAuthoring<Schema, Program, ApplicationProgramComplete>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// ```compile_fail
    /// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
    /// fn missing_connection<Schema, Program>()
    /// where Schema: ApplicationSchema, Program: ApplicationProgramDefinition<Schema> {
    ///     let _ = ApplicationProgramAuthoring::<Schema, Program>::begin().validated_program();
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
    /// fn string_is_not_a_connection<Schema, Program>()
    /// where Schema: ApplicationSchema, Program: ApplicationProgramDefinition<Schema> {
    ///     let _ = ApplicationProgramAuthoring::<Schema, Program>::begin().connect("source -> target");
    /// }
    /// ```
    pub fn validated_program(
        self,
    ) -> Result<ValidatedApplicationProgram<Schema, Program>, ApplicationProgramValidationDenial>
    {
        validate::<Schema, Program>(
            self.connections,
            [
                self.local_rule
                    .expect("complete authoring retains its local rule"),
                self.shared_rule
                    .expect("complete authoring retains its shared rule"),
            ],
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationProgramValidationDenialKind {
    InvalidIdentity,
    DuplicateFeature,
    DuplicateConnection,
    DanglingFeature,
    MissingRequiredInput,
    DuplicateInputBinding,
    DuplicateRule,
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

/// Validated immutable program meaning consumed by installation.
pub struct ValidatedApplicationProgram<Schema, Program> {
    identity: ApplicationProgramIdentity,
    features: &'static [ApplicationFeatureDeclaration],
    connections: Box<[ApplicationConnectionDeclaration]>,
    rules: [ApplicationProgramRuleDeclaration; 2],
    marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> ValidatedApplicationProgram<Schema, Program> {
    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }
    pub const fn features(&self) -> &'static [ApplicationFeatureDeclaration] {
        self.features
    }
    pub fn connections(&self) -> &[ApplicationConnectionDeclaration] {
        &self.connections
    }
    pub fn rules(&self) -> &[ApplicationProgramRuleDeclaration] {
        &self.rules
    }
}

fn validate<Schema, Program>(
    connections: Vec<ApplicationConnectionDeclaration>,
    rules: [ApplicationProgramRuleDeclaration; 2],
) -> Result<ValidatedApplicationProgram<Schema, Program>, ApplicationProgramValidationDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    require_identity(Program::IDENTITY.as_str())?;
    let features = Program::features();
    let mut feature_ids = BTreeSet::new();
    for feature in features {
        require_identity(feature.identity())?;
        if !feature_ids.insert(feature.identity()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateFeature,
                feature.identity(),
            ));
        }
        for input in feature.required_inputs() {
            require_identity(input)?;
        }
    }
    let mut rule_ids = BTreeSet::new();
    for rule in &rules {
        require_identity(rule.identity())?;
        if !rule_ids.insert((
            rule.identity(),
            rule.major(),
            rule.minor(),
            rule.execution_point(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateRule,
                rule.identity(),
            ));
        }
        if rule
            .local_owner()
            .is_some_and(|owner| !feature_ids.contains(owner))
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                rule.identity(),
            ));
        }
    }
    let mut connection_ids = BTreeSet::new();
    let mut bound_inputs = BTreeSet::new();
    for connection in &connections {
        require_identity(connection.identity())?;
        require_identity(connection.source_port())?;
        require_identity(connection.target_port())?;
        if !connection_ids.insert(connection.identity()) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateConnection,
                connection.identity(),
            ));
        }
        if !feature_ids.contains(connection.source_feature())
            || !feature_ids.contains(connection.target_feature())
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                connection.identity(),
            ));
        }
        if !bound_inputs.insert((connection.target_feature(), connection.target_port())) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateInputBinding,
                connection.target_port(),
            ));
        }
    }
    for feature in features {
        for input in feature.required_inputs() {
            if !bound_inputs.contains(&(feature.identity(), *input)) {
                return Err(denial(
                    ApplicationProgramValidationDenialKind::MissingRequiredInput,
                    format!("{}.{input}", feature.identity()),
                ));
            }
        }
    }
    Ok(ValidatedApplicationProgram {
        identity: Program::IDENTITY.clone(),
        features,
        connections: connections.into_boxed_slice(),
        rules,
        marker: PhantomData,
    })
}

fn require_identity(identity: &str) -> Result<(), ApplicationProgramValidationDenial> {
    if identity.is_empty()
        || identity.trim() != identity
        || identity.chars().any(char::is_whitespace)
    {
        Err(denial(
            ApplicationProgramValidationDenialKind::InvalidIdentity,
            identity,
        ))
    } else {
        Ok(())
    }
}

fn denial(
    kind: ApplicationProgramValidationDenialKind,
    subject: impl Into<String>,
) -> ApplicationProgramValidationDenial {
    ApplicationProgramValidationDenial {
        kind,
        subject: subject.into(),
    }
}

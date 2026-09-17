use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationActionDeclaration, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramActionsShape, ApplicationProgramFeaturesShape, ApplicationProgramIdentity,
    ApplicationProgramOutputsShape, ApplicationProgramRuleDeclaration,
    ApplicationProgramRulesShape,
};

/// Complete authored static program definition.
pub trait ApplicationProgramDefinition<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    /// Exact root contribution tuple whose generated slots implement this
    /// program. Installation must consume this tuple before exposing a root.
    type Contributions;
    /// Complete feature-owned action inventory for this program.
    type Actions: ApplicationProgramActionsShape<Schema>;
    /// Complete feature inventory and each feature's authored input ports.
    type Features: ApplicationProgramFeaturesShape<Schema>;
    /// Complete transitive output topology used by the installed executor.
    type Outputs: super::ApplicationProgramOutputsShape<Schema>;
    /// Complete scoped invariant inventory owned by this composition.
    type Rules: ApplicationProgramRulesShape<Schema>;
    const IDENTITY: ApplicationProgramIdentity;
}

/// Phase 1 authoring progression. Validation is unavailable until its typed
/// required connection has been supplied.
pub struct ApplicationProgramAuthoring<Schema, Program> {
    marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> ApplicationProgramAuthoring<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub const fn begin() -> Self {
        Self {
            marker: PhantomData,
        }
    }
    /// ```compile_fail
    /// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
    /// fn missing_connection<Schema, Program>()
    /// where Schema: ApplicationSchema, Program: ApplicationProgramDefinition<Schema> {
    ///     type Missing = ApplicationOutputLeaf;
    ///     fn require_graph<S: ApplicationSchema, G: ApplicationProgramOutputsShape<S>>() {}
    ///     require_graph::<Schema, Missing>();
    /// }
    /// ```
    ///
    /// ```compile_fail
    /// use worth_query_declaration::facade::{application_program::*, application_schema::ApplicationSchema};
    /// fn string_is_not_a_connection<Schema>()
    /// where Schema: ApplicationSchema {
    ///     fn require_graph<S: ApplicationSchema, G: ApplicationProgramOutputsShape<S>>() {}
    ///     require_graph::<Schema, &'static str>();
    /// }
    /// ```
    pub fn validated_program(
        self,
    ) -> Result<ValidatedApplicationProgram<Schema, Program>, ApplicationProgramValidationDenial>
    {
        validate::<Schema, Program>(Program::Outputs::connections(), Program::Rules::rules())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationProgramValidationDenialKind {
    InvalidIdentity,
    DuplicateFeature,
    DuplicateAction,
    DuplicateConnection,
    DanglingFeature,
    MissingRequiredInput,
    DuplicateInputBinding,
    UndeclaredInput,
    UnexportedCrossInstanceConnection,
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
    features: Box<[ApplicationFeatureDeclaration]>,
    actions: Box<[ApplicationActionDeclaration]>,
    connections: Box<[ApplicationConnectionDeclaration]>,
    rules: Box<[ApplicationProgramRuleDeclaration]>,
    marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> ValidatedApplicationProgram<Schema, Program> {
    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }
    pub fn features(&self) -> &[ApplicationFeatureDeclaration] {
        &self.features
    }
    pub fn actions(&self) -> &[ApplicationActionDeclaration] {
        &self.actions
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
    rules: Vec<ApplicationProgramRuleDeclaration>,
) -> Result<ValidatedApplicationProgram<Schema, Program>, ApplicationProgramValidationDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    require_identity(Program::IDENTITY.as_str())?;
    let features = Program::Features::features();
    let mut feature_ids = BTreeSet::new();
    for feature in &features {
        require_identity(feature.composition_instance())?;
        require_identity(feature.identity())?;
        if !feature_ids.insert((feature.composition_instance(), feature.identity())) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateFeature,
                feature.identity(),
            ));
        }
        let mut input_ids = BTreeSet::new();
        for input in feature.inputs() {
            require_identity(input.identity())?;
            if !input_ids.insert(input.identity()) {
                return Err(denial(
                    ApplicationProgramValidationDenialKind::DuplicateInputBinding,
                    format!("{}.{}", feature.identity(), input.identity()),
                ));
            }
        }
    }
    let actions = Program::Actions::actions();
    let mut action_ids = BTreeSet::new();
    for action in &actions {
        require_identity(action.composition_instance())?;
        require_identity(action.feature())?;
        require_identity(action.binding())?;
        if !feature_ids.contains(&(action.composition_instance(), action.feature())) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                action.feature(),
            ));
        }
        if !action_ids.insert((
            action.composition_instance(),
            action.feature(),
            action.action_type(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateAction,
                action.binding(),
            ));
        }
    }
    let mut rule_ids = BTreeSet::new();
    for rule in &rules {
        require_identity(rule.composition_instance())?;
        require_identity(rule.identity())?;
        if !rule_ids.insert((
            rule.composition_instance(),
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
        let scope_exists = features
            .iter()
            .any(|feature| feature.composition_instance() == rule.composition_instance());
        let owner_exists = rule
            .local_owner()
            .is_none_or(|owner| feature_ids.contains(&(rule.composition_instance(), owner)));
        if !scope_exists || !owner_exists {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                rule.identity(),
            ));
        }
    }
    let mut connection_ids = BTreeSet::new();
    let mut bound_inputs = BTreeSet::new();
    for connection in &connections {
        require_identity(connection.source_instance())?;
        require_identity(connection.target_instance())?;
        require_identity(connection.identity())?;
        require_identity(connection.source_port())?;
        require_identity(connection.target_port())?;
        if !connection_ids.insert((
            connection.source_instance(),
            connection.target_instance(),
            connection.identity(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateConnection,
                connection.identity(),
            ));
        }
        if !feature_ids.contains(&(connection.source_instance(), connection.source_feature()))
            || !feature_ids.contains(&(connection.target_instance(), connection.target_feature()))
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DanglingFeature,
                connection.identity(),
            ));
        }
        if connection.source_instance() != connection.target_instance()
            && !connection.exports_across_instances()
        {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UnexportedCrossInstanceConnection,
                connection.identity(),
            ));
        }
        let target = features
            .iter()
            .find(|feature| {
                feature.composition_instance() == connection.target_instance()
                    && feature.identity() == connection.target_feature()
            })
            .expect("the target feature was proven present");
        if !target.inputs().iter().any(|input| {
            input.identity() == connection.target_port()
                && input.required() == connection.target_required()
        }) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::UndeclaredInput,
                format!(
                    "{}.{}",
                    connection.target_feature(),
                    connection.target_port()
                ),
            ));
        }
        if !bound_inputs.insert((
            connection.target_instance(),
            connection.target_feature(),
            connection.target_port(),
        )) {
            return Err(denial(
                ApplicationProgramValidationDenialKind::DuplicateInputBinding,
                connection.target_port(),
            ));
        }
    }
    for feature in &features {
        for input in feature.required_inputs() {
            if !bound_inputs.contains(&(feature.composition_instance(), feature.identity(), input))
            {
                return Err(denial(
                    ApplicationProgramValidationDenialKind::MissingRequiredInput,
                    format!("{}.{input}", feature.identity()),
                ));
            }
        }
    }
    Ok(ValidatedApplicationProgram {
        identity: Program::IDENTITY.clone(),
        features: features.into_boxed_slice(),
        actions: actions.into_boxed_slice(),
        connections: connections.into_boxed_slice(),
        rules: rules.into_boxed_slice(),
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

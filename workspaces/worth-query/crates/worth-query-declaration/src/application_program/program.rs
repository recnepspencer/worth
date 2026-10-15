use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::application_schema::ApplicationSchema;

use super::{
    ApplicationActionDeclaration, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationFeatureSpec, ApplicationProgramIdentity, ApplicationProgramManifest,
    ApplicationProgramOutputsShape, ApplicationProgramRevision,
    ApplicationProgramRevisionBudgetDenial, ApplicationProgramRuleDeclaration,
    ApplicationProgramRulesShape, ApplicationSemanticDescription,
};

mod definition_validation;
mod delivery_validation;
mod validation_denial;
use validation_denial::denial;
pub use validation_denial::{
    ApplicationProgramValidationDenial, ApplicationProgramValidationDenialKind,
};
mod dependency_graph;
mod validation;
use definition_validation::validate_features_and_actions;
use dependency_graph::require_acyclic_connections;
use validation::validate;

/// Complete authored static program definition.
pub trait ApplicationProgramDefinition<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    /// Exact root contribution tuple whose generated slots implement this
    /// program. Installation must consume this tuple before exposing a root.
    type Contributions;
    /// Complete transitive output topology used by the installed executor.
    type Outputs: super::ApplicationProgramOutputsShape<Schema>;
    /// Complete scoped invariant inventory owned by this composition.
    type Rules: ApplicationProgramRulesShape<Schema>;
    const IDENTITY: ApplicationProgramIdentity;
    const DERIVED_ARTIFACT_GOVERNANCE: super::ApplicationDerivedArtifactGovernance =
        super::ApplicationDerivedArtifactGovernance::Compatible;

    /// Complete feature inventory with every action attached to its owning
    /// feature occurrence.
    fn feature_specs() -> Vec<ApplicationFeatureSpec>;

    /// No declaration refuses ordinary comparison and adoption.
    fn adoption_authorization<
        Visitor: super::ApplicationProgramAdoptionAuthorizationVisitor<Schema>,
    >(
        visitor: Visitor,
    ) -> Visitor::Output {
        visitor.unavailable()
    }
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

/// Validated immutable program meaning consumed by installation.
pub struct ValidatedApplicationProgram<Schema, Program> {
    identity: ApplicationProgramIdentity,
    revision: ApplicationProgramRevision,
    features: Box<[ApplicationFeatureDeclaration]>,
    actions: Box<[ApplicationActionDeclaration]>,
    connections: Box<[ApplicationConnectionDeclaration]>,
    rules: Box<[ApplicationProgramRuleDeclaration]>,
    semantic_description: ApplicationSemanticDescription,
    adoption_authorization: Option<super::ApplicationProgramAdoptionAuthorizationDeclaration>,
    marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> ValidatedApplicationProgram<Schema, Program> {
    pub fn adoption_authorization(
        &self,
    ) -> Option<&super::ApplicationProgramAdoptionAuthorizationDeclaration> {
        self.adoption_authorization.as_ref()
    }

    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }
    /// Canonical content identity minted when this meaning was validated.
    pub fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
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
    /// Immutable authored meaning used for bounded program evolution.
    pub fn semantic_description(&self) -> &ApplicationSemanticDescription {
        &self.semantic_description
    }
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

/// Names an exceeded canonical revision budget as program validation meaning,
/// so a program too large to identify is denied rather than silently hashed.
pub(in crate::application_program) fn deny_canonical_revision_budget(
    identity: &ApplicationProgramIdentity,
    budget: ApplicationProgramRevisionBudgetDenial,
) -> ApplicationProgramValidationDenial {
    denial(
        ApplicationProgramValidationDenialKind::CanonicalRevisionBudgetExceeded,
        format!("{}: {budget}", identity.as_str()),
    )
}

use std::collections::BTreeSet;
use std::marker::PhantomData;

mod correspondence;
mod derived_artifact;
mod evaluated_requirement;
mod external_input;
mod scoped_action;
mod support;
pub use correspondence::{
    WorthQueryCorrespondedAction, WorthQueryInstalledRepeatedOptionalMember,
    WorthQueryRepeatedOptionalMemberState,
};
pub use derived_artifact::{WorthQueryInstalledDerivedArtifact, WorthQueryProgramArtifactPosture};
pub use evaluated_requirement::WorthQueryInstalledEvaluatedRequirement;
pub use external_input::{
    WorthQueryAdmittedExternalInput, WorthQueryCapturedExternalInput,
    WorthQueryInstalledExternalInputProvider,
};
pub use scoped_action::WorthQueryInstalledScopedAction;
pub use support::{
    WorthQueryProgramRuleKey, WorthQueryProgramSupportAdmission, WorthQueryProgramSupportDenial,
    WorthQueryProgramSupportEntry, WorthQueryProgramSupportRoster,
};

use worth_query_declaration::facade::application_program::{
    ApplicationActionDeclaration, ApplicationConnectionDeclaration, ApplicationFeatureDeclaration,
    ApplicationProgramDefinition, ApplicationProgramIdentity, ApplicationProgramRevision,
    ApplicationProgramRuleDeclaration, ValidatedApplicationProgram,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
};

/// Failure to bind validated program meaning to its exact installed schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationProgramInstallationDenial {
    subject: String,
    support_denial: Option<WorthQueryProgramSupportDenial>,
}

impl WorthQueryApplicationProgramInstallationDenial {
    pub fn subject(&self) -> &str {
        &self.subject
    }

    /// Hands back the structured support refusal this denial was raised from.
    ///
    /// Installation denials that never reached support admission — an
    /// undeclared program binding, for one — carry no support reason, so a
    /// caller reading `None` learns that the refusal happened elsewhere rather
    /// than that the reason was lost.
    pub const fn support_denial(&self) -> Option<&WorthQueryProgramSupportDenial> {
        self.support_denial.as_ref()
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

impl WorthQueryApplicationProgramInstallationDenial {
    fn new(subject: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            support_denial: None,
        }
    }
}

impl From<WorthQueryProgramSupportDenial> for WorthQueryApplicationProgramInstallationDenial {
    /// Renders one support refusal as the installation subject this boundary
    /// has always reported, while retaining the structured reason so a caller
    /// can act on the refusal instead of parsing prose.
    fn from(denial: WorthQueryProgramSupportDenial) -> Self {
        Self {
            subject: support_denial_subject(&denial),
            support_denial: Some(denial),
        }
    }
}

/// Names the installation subject each support refusal has always reported.
///
/// Every variant is answered by name: a new support refusal must decide its
/// installation subject here rather than inherit a catch-all rendering.
fn support_denial_subject(denial: &WorthQueryProgramSupportDenial) -> String {
    match denial {
        WorthQueryProgramSupportDenial::EmptyProgram { program } => program.as_str().to_owned(),
        WorthQueryProgramSupportDenial::UnsupportedRuleContract { rule, .. } => {
            rule.identity().to_owned()
        }
        WorthQueryProgramSupportDenial::UnsupportedAction { binding, .. } => binding.clone(),
        WorthQueryProgramSupportDenial::UndeclaredInstalledRule { rule } => {
            format!("undeclared installed rule: {}", rule.identity())
        }
        WorthQueryProgramSupportDenial::DuplicateProgram { .. } => denial.to_string(),
        WorthQueryProgramSupportDenial::UnrosteredProgram { .. } => denial.to_string(),
        WorthQueryProgramSupportDenial::ForeignSchemaBinding { .. } => denial.to_string(),
    }
}

/// Installed program meaning affine to one schema installation.
pub struct WorthQueryInstalledApplicationProgram<Schema, Program> {
    identity: ApplicationProgramIdentity,
    revision: ApplicationProgramRevision,
    schema_binding: ApplicationSchemaBindingIdentity,
    features: Box<[ApplicationFeatureDeclaration]>,
    actions: Box<[ApplicationActionDeclaration]>,
    connections: Box<[ApplicationConnectionDeclaration]>,
    rules: Box<[ApplicationProgramRuleDeclaration]>,
    marker: PhantomData<fn() -> (Schema, Program)>,
}

impl<Schema, Program> WorthQueryInstalledApplicationProgram<Schema, Program> {
    pub fn identity(&self) -> &ApplicationProgramIdentity {
        &self.identity
    }
    /// Canonical content identity of the validated meaning installed here.
    pub fn revision(&self) -> &ApplicationProgramRevision {
        &self.revision
    }
    pub fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        &self.schema_binding
    }
    pub fn features(&self) -> &[ApplicationFeatureDeclaration] {
        &self.features
    }
    pub fn actions(&self) -> &[ApplicationActionDeclaration] {
        &self.actions
    }

    pub fn contains_action_type<Binding: 'static>(&self) -> bool {
        let binding = std::any::TypeId::of::<Binding>();
        self.actions
            .iter()
            .any(|action| action.action_type() == binding)
    }

    pub fn repeated_optional_member<Binding, Correspondence>(
        &self,
    ) -> Option<WorthQueryInstalledRepeatedOptionalMember<Schema, Binding, Correspondence>>
    where
        Schema: ApplicationSchema,
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
        >,
        Correspondence: worth_query_declaration::facade::application_program::ApplicationRepeatedOptionalMemberCorrespondence<
            Schema,
            Binding,
        >,
    {
        let binding = std::any::TypeId::of::<Binding>();
        let correspondence = std::any::TypeId::of::<Correspondence>();
        self.actions
            .iter()
            .any(|action| {
                action.mutation_binding_type() == Some(binding)
                    && action
                        .correspondence()
                        .is_some_and(|declared| declared.correspondence_type() == correspondence)
            })
            .then(|| WorthQueryInstalledRepeatedOptionalMember::new(self.schema_binding.clone()))
    }

    pub fn evaluated_requirement<Operation, Rule>(
        &self,
    ) -> Option<WorthQueryInstalledEvaluatedRequirement<Schema, Operation, Rule>>
    where
        Schema: ApplicationSchema,
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema> + 'static,
        Rule: worth_query_declaration::facade::application_program::ApplicationEvaluatedRequirementRule<Schema, Operation>,
    {
        let rule_type = std::any::TypeId::of::<Rule>();
        let operation_type = std::any::TypeId::of::<Operation>();
        self.actions
            .iter()
            .any(|action| {
                action.operation_type() == operation_type
                    && action
                        .evaluated_requirement()
                        .is_some_and(|rule| rule.rule_type() == rule_type)
            })
            .then(|| WorthQueryInstalledEvaluatedRequirement::new(self.schema_binding.clone()))
    }

    pub fn external_input_provider<Operation, Provider>(
        &self,
    ) -> Option<WorthQueryInstalledExternalInputProvider<Schema, Operation, Provider>>
    where
        Schema: ApplicationSchema,
        Operation: worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<Schema> + 'static,
        Provider: worth_query_declaration::facade::application_program::ApplicationExternalInputProvider<Schema, Operation>,
    {
        let provider_type = std::any::TypeId::of::<Provider>();
        let operation_type = std::any::TypeId::of::<Operation>();
        self.actions
            .iter()
            .any(|action| {
                action.operation_type() == operation_type
                    && action
                        .external_input()
                        .is_some_and(|provider| provider.provider_type() == provider_type)
            })
            .then(|| WorthQueryInstalledExternalInputProvider::new(self.schema_binding.clone()))
    }

    pub fn contains_operation_type<Operation: 'static>(&self) -> bool {
        let operation = std::any::TypeId::of::<Operation>();
        self.actions
            .iter()
            .any(|action| action.operation_type() == operation && !action.conditional_only())
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

/// Installs the single program a host runs, which must own the installed rule
/// catalog by itself. This is exactly the one-entry roster case.
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
    let roster = WorthQueryProgramSupportAdmission::for_installed_schema(installed_schema)
        .support(&program)?
        .close()?;
    Ok(install_rostered_application_program(
        program,
        installed_schema,
        &roster,
    )?)
}

/// Installs one program a host already admitted into its support roster.
///
/// The roster carries the proof that every installed rule has a declaring
/// owner, so a rostered program may legally declare fewer rules than the
/// catalog holds while a peer program declares the rest.
pub fn install_rostered_application_program<Schema, Program>(
    program: ValidatedApplicationProgram<Schema, Program>,
    installed_schema: &crate::facade::WorthQueryInstalledApplicationSchema<Schema>,
    roster: &WorthQueryProgramSupportRoster<Schema>,
) -> Result<WorthQueryInstalledApplicationProgram<Schema, Program>, WorthQueryProgramSupportDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    let schema_binding = installed_schema.binding_identity();
    if roster.schema_binding() != &schema_binding {
        return Err(WorthQueryProgramSupportDenial::ForeignSchemaBinding {
            rostered: roster.schema_binding().clone(),
            presented: schema_binding,
        });
    }
    if roster.entry(program.revision()).is_none() {
        return Err(WorthQueryProgramSupportDenial::UnrosteredProgram {
            program: program.identity().clone(),
            revision: program.revision().clone(),
        });
    }
    Ok(WorthQueryInstalledApplicationProgram {
        identity: program.identity().clone(),
        revision: program.revision().clone(),
        schema_binding,
        features: program.features().to_vec().into_boxed_slice(),
        actions: program.actions().to_vec().into_boxed_slice(),
        connections: program.connections().to_vec().into_boxed_slice(),
        rules: program.rules().to_vec().into_boxed_slice(),
        marker: PhantomData,
    })
}

#[doc(hidden)]
pub fn require_complete_program_binding_membership<Schema, Program>(
    program: &WorthQueryInstalledApplicationProgram<Schema, Program>,
    installed_schema: &crate::facade::WorthQueryInstalledApplicationSchema<Schema>,
    output_source_bindings: &BTreeSet<std::any::TypeId>,
) -> Result<(), WorthQueryApplicationProgramInstallationDenial>
where
    Schema: ApplicationSchema,
{
    let action_bindings = program
        .actions()
        .iter()
        .filter_map(ApplicationActionDeclaration::mutation_binding_type)
        .collect::<BTreeSet<_>>();
    if let Some(binding) = installed_schema
        .installed_mutation_binding_inventory()
        .find(|binding| {
            binding.requires_application_program()
                && !action_bindings.contains(&binding.binding_type())
                && !output_source_bindings.contains(&binding.binding_type())
        })
    {
        return Err(WorthQueryApplicationProgramInstallationDenial::new(
            format!("undeclared program binding: {}", binding.identity()),
        ));
    }
    Ok(())
}

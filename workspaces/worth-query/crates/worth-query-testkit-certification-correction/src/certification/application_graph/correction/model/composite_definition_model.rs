//! An authored predicate parameter and its ordinary combined source operation.
use super::{query, schema::*};
use worth_query_host::facade::{
    declaration::{
        application_aftermath::*, application_operation::*, application_program::*,
        application_query::*, application_schema::*,
    },
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Change {
    pub key: String,
    pub value: u64,
    pub revision: u64,
}
#[derive(Clone, Debug)]
pub struct Changed;
#[derive(Clone, Debug)]
pub struct Missing;
worth_query_structured_value_binding!(pub InputBinding for Change { identity: "numeric-definition.input.v2" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "numeric-definition.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Missing { identity: "numeric-definition.denial.v1" });
worth_query_operation!(pub ChangeDefinition for NumericSchema, input InputBinding);

worth_query_operation!(pub Tick for NumericSchema, input InputBinding);

impl ApplicationDeclaredOperation<NumericSchema> for ChangeDefinition {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_read_field(Self::reference(), Primary::at("Primary"))
            .operation_read_field(
                Self::reference(),
                DefinitionRevision::at("DefinitionRevision"),
            )
            .operation_write(
                Self::reference(),
                DefinitionRevision::at("DefinitionRevision"),
            )
            .operation_write(Self::reference(), Primary::at("Primary"))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            Tick::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(
            Tick::reference(),
            DefinitionRevision::at("DefinitionRevision"),
        )
        .operation_write(
            Tick::reference(),
            DefinitionRevision::at("DefinitionRevision"),
        )
        .operation_read_field(Tick::reference(), Key::at("Key"))
        .operation_read_field(Tick::reference(), Primary::at("Primary"))
        .operation_read_field(Tick::reference(), Secondary::at("Secondary"))
        .operation_read_field(Tick::reference(), Ceiling::at("Ceiling"))
        .operation_write(Tick::reference(), Secondary::at("Secondary"))
        .operation_write(Tick::reference(), Primary::at("Primary"))
        .operation_write(Tick::reference(), Ceiling::at("Ceiling"))
        .application_query(super::composite_definition_query::definition())
        .application_query_binding::<super::composite_definition_query::Binding>()
}

pub struct Binding;
pub struct Decision {
    pub target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
    pub definition: WorthQueryAdmittedApplicationConditionalDefinition,
}
pub type Scope = ApplicationMutationFieldScope<Key>;
impl ApplicationMutationAdmissionBinding<NumericSchema> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<NumericSchema> for Binding {
    type Input = Change;
    type InputBinding = InputBinding;
    type Result = Changed;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = String;
    type Operation = ChangeDefinition;
    type Decision = Decision;
    type Denial = Missing;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = Scope;
    type PrincipalBinding = ActorBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;
    const IDENTITY: &'static str = "numeric-definition.binding.v1";
    const HANDLER_IDENTITY: &'static str = "numeric-definition.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "numeric-definition.command.v1";
    fn scope_field() -> Key {
        Key::at("Key")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        NumericSchema,
        ActorBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ActorBinding::reference()
    }
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
}
impl ApplicationMutationIntent<NumericSchema> for Change {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> Scope {
        Scope::new(Key::at("Key"), self.key.clone())
    }
}
pub struct RegisterDefinitionEdit;
impl worth_query_decl::facade::application_schema::fragment::Registration<NumericSchema> for RegisterDefinitionEdit {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                ChangeDefinition::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<Binding>())
    }
}
pub struct Feature;
pub struct Completion;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "numeric-definition.feature.v1";
}
impl ApplicationActionDelivery<NumericSchema, Binding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-definition.completion.v1";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<Binding, Completion>()
        .finish_action()
        .conditional_operation::<Tick>()
        .finish()
}

// The temporal owner's revision is a native scalar, separate from the business
// atom. Only actual numeric intents carry this optional role.

pub use crate::facade::certification::application_graph::correction::model::schema::storage_model::{DefinitionRevisionFacts, DefinitionRevision};

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
}
#[derive(Clone, Debug)]
pub struct Changed;
#[derive(Clone, Debug)]
pub struct Missing;
worth_query_structured_value_binding!(pub InputBinding for Change { identity: "numeric-definition.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Changed { identity: "numeric-definition.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Missing { identity: "numeric-definition.denial.v1" });
worth_query_operation!(pub ChangeDefinition for NumericSchema, input InputBinding);
worth_query_operation_reads!(ChangeDefinition => [Key, Primary]);
worth_query_operation_writes!(ChangeDefinition => [Primary]);
worth_query_operation!(pub Tick for NumericSchema, input InputBinding);
worth_query_operation_reads!(Tick => [Key, Primary, Secondary, Ceiling]);
worth_query_operation_writes!(Tick => [Primary, Secondary, Ceiling]);

impl ApplicationDeclaredOperation<NumericSchema> for ChangeDefinition {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Primary::reference())
            .operation_write(Self::reference(), Primary::reference())
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
        .operation_read_field(Tick::reference(), Key::reference())
        .operation_read_field(Tick::reference(), Primary::reference())
        .operation_read_field(Tick::reference(), Secondary::reference())
        .operation_read_field(Tick::reference(), Ceiling::reference())
        .operation_write(Tick::reference(), Secondary::reference())
        .operation_write(Tick::reference(), Primary::reference())
        .operation_write(Tick::reference(), Ceiling::reference())
        .application_query(super::composite_definition_query::definition())
        .application_query_binding::<super::composite_definition_query::Binding>()
}

pub struct Binding;
pub struct Decision {
    pub target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
    pub definition: WorthQueryAdmittedApplicationConditionalDefinition,
}
type Scope = ApplicationMutationFieldScope<
    NumericSchema,
    Number,
    NumberFacts,
    Key,
    String,
    ReadOnly,
    NoApplicationUnit,
>;
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
    fn scope_field() -> ApplicationFieldRef<
        NumericSchema,
        Number,
        NumberFacts,
        Key,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        Key::reference()
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
        Scope::new(Key::reference(), self.key.clone())
    }
}
pub struct CorrectDefinition;
impl ApplicationCorrectionBinding<NumericSchema> for CorrectDefinition {
    const IDENTITY: &'static str = "numeric-definition.correction.v1";
    type Source = Binding;
    type InverseIntent = Change;
    type Disclosure = query::ReadBinding;
    fn disclosure_parameters(
        _: &Change,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<NumericSchema, ChangeDefinition, Change, true, false>
    {
        ChangeDefinition::reference()
            .definition()
            .no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(Primary::reference())],
                U64ApplicationValueBinding::encode(&u64::MAX)
                    .unwrap()
                    .semantic_byte_width(),
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &Change,
        material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
    ) -> Result<Change, ApplicationCorrectionDerivationDenial> {
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
        };
        let value = preimage
            .field(&DeclaredPreImageLocus::from_field(Primary::reference()))
            .ok_or(ApplicationCorrectionDerivationDenial::MissingPreImage)?;
        Ok(Change {
            key: original.key.clone(),
            value: U64ApplicationValueBinding::decode(value)
                .map_err(|_| ApplicationCorrectionDerivationDenial::InvalidPreImage)?,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("definition-correction:{key}"))
    }
    fn visit<Visitor: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, Visitor>,
    ) -> Visitor::Output {
        dispatch.conventional()
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

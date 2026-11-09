use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationMutationBinding, ApplicationMutationFieldScope, ApplicationMutationIntent,
            NoApplicationMutationOutputs, NoApplicationMutationSource,
        },
        application_query::{
            ApplicationQueryBinding, ApplicationQueryBindingLimits, ApplicationQueryFieldScope,
            ApplicationQueryIntent, ApplicationQueryParameterSet,
        },
        application_schema::{
            ApplicationPrincipalBindingRef, ApplicationSchemaDeclarationBuilder,
            U64ApplicationValueBinding,
        },
    },
    primary_graph::{
        CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
        WorthQueryInvariantMutationTarget,
    },
    worth_query_structured_value_binding,
};

use super::schema::{
    AmendTemporal, AmendTemporalInput, AmendTemporalInputBinding, ExternalMapping, IntentDueField,
    IntentGateField, IntentIdentityField, IntentInputField, IntentLifecycleField,
    IntentQueryParametersBinding, IntentQueryResultBinding, IntentRevisionField, Principal,
    TemporalHostSchema, TemporalIntent, TemporalIntentQuery, TemporalPrincipalBinding,
};

#[derive(Clone, Debug)]
pub struct TemporalIntentRead {
    pub identity: String,
}

worth_query_structured_value_binding!(pub TemporalIntentReadBinding for TemporalIntentRead {
    identity: "worth.query.example.temporal-intent-read.v1"
});

pub struct TemporalIntentQueryBinding;

pub type TemporalIntentQueryScope = ApplicationQueryFieldScope<IntentIdentityField>;

impl ApplicationQueryBinding<TemporalHostSchema> for TemporalIntentQueryBinding {
    type Input = TemporalIntentRead;
    type InputBinding = TemporalIntentReadBinding;
    type Query = TemporalIntentQuery;
    type ParameterBinding = IntentQueryParametersBinding;
    type ResultBinding = IntentQueryResultBinding;
    type ScopeBinding = TemporalIntentQueryScope;
    type PrincipalBinding = TemporalPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str = "worth.query.example.temporal-intent-query.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);

    fn scope_field() -> IntentIdentityField {
        IntentIdentityField::at("IntentIdentityField")
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        TemporalHostSchema,
        TemporalPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        TemporalPrincipalBinding::reference()
    }
}

impl ApplicationQueryIntent<TemporalHostSchema> for TemporalIntentRead {
    type Binding = TemporalIntentQueryBinding;

    fn parameters(&self) -> ApplicationQueryParameterSet<TemporalIntentQuery> {
        ApplicationQueryParameterSet::new()
    }

    fn into_scope(self) -> TemporalIntentQueryScope {
        TemporalIntentQueryScope::new(
            IntentIdentityField::at("IntentIdentityField"),
            self.identity,
        )
    }
}

#[derive(Clone, Debug)]
pub struct AmendTemporalIntent {
    pub identity: String,
    pub amendment: AmendTemporalInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmendTemporalResult {
    pub revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AmendTemporalDenial {
    RevisionMustAdvance,
}

worth_query_structured_value_binding!(pub AmendTemporalResultBinding for AmendTemporalResult {
    identity: "worth.query.example.temporal-amend-result.v1"
});
worth_query_structured_value_binding!(pub AmendTemporalDenialBinding for AmendTemporalDenial {
    identity: "worth.query.example.temporal-amend-denial.v1"
});

pub struct AmendTemporalBinding;
impl worth_query_host::facade::declaration::application_operation::ApplicationMutationAdmissionBinding<TemporalHostSchema> for AmendTemporalBinding {
    type Admission = worth_query_host::facade::declaration::application_operation::ApplicationConventionalAdmission;
}

pub type AmendTemporalScope = ApplicationMutationFieldScope<IntentIdentityField>;

impl ApplicationMutationBinding<TemporalHostSchema> for AmendTemporalBinding {
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = AmendTemporalInput;
    type InputBinding = AmendTemporalInputBinding;
    type Result = AmendTemporalResult;
    type ResultBinding = AmendTemporalResultBinding;
    type IdempotencyKey = u64;
    type Operation = AmendTemporal;
    type Decision = WorthQueryInvariantMutationTarget<TemporalHostSchema, TemporalIntent>;
    type Denial = AmendTemporalDenial;
    type DenialBinding = AmendTemporalDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = AmendTemporalScope;
    type PrincipalBinding = TemporalPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.example.temporal-amend.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.example.temporal-amend-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "worth.query.example.temporal-amend-command.v1";

    fn scope_field() -> IntentIdentityField {
        IntentIdentityField::at("IntentIdentityField")
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        TemporalHostSchema,
        TemporalPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        TemporalPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<TemporalHostSchema> for AmendTemporalIntent {
    type Binding = AmendTemporalBinding;

    fn input(&self) -> &AmendTemporalInput {
        &self.amendment
    }

    fn scope_binding(&self) -> AmendTemporalScope {
        AmendTemporalScope::new(
            IntentIdentityField::at("IntentIdentityField"),
            self.identity.clone(),
        )
    }
}

pub struct AmendTemporalHandler;

impl OperationHandler<TemporalHostSchema, AmendTemporalBinding> for AmendTemporalHandler {
    fn decide(
        &self,
        input: &AmendTemporalInput,
        reader: &mut DecisionReader<'_, '_, '_, TemporalHostSchema, AmendTemporalBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<TemporalHostSchema, TemporalIntent>,
        AmendTemporalDenial,
    > {
        let entity = match reader.resolve_entity(
            IntentIdentityField::at("IntentIdentityField"),
            "intent-1".to_owned(),
        ) {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&entity, IntentRevisionField::at("IntentRevisionField")) {
            Ok(Some(revision)) if input.revision > revision => {}
            Ok(_) => return HandlerResult::DomainDenied(AmendTemporalDenial::RevisionMustAdvance),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        for read in [
            reader
                .field(&entity, IntentDueField::at("IntentDueField"))
                .map(|_| ()),
            reader
                .field(&entity, IntentLifecycleField::at("IntentLifecycleField"))
                .map(|_| ()),
            reader
                .field(&entity, IntentInputField::at("IntentInputField"))
                .map(|_| ()),
            reader
                .field(&entity, IntentGateField::at("IntentGateField"))
                .map(|_| ()),
        ] {
            if let Err(error) = read {
                return HandlerResult::ExecutionDenied(error);
            }
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn build_candidate(
        &self,
        input: &AmendTemporalInput,
        target: WorthQueryInvariantMutationTarget<TemporalHostSchema, TemporalIntent>,
        writer: &mut CandidateWriter<'_, TemporalHostSchema, AmendTemporalBinding>,
    ) -> HandlerResult<AmendTemporalResult, AmendTemporalDenial> {
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        };
        for result in [
            writer.write_field(
                &entity,
                IntentRevisionField::at("IntentRevisionField"),
                input.revision,
            ),
            writer.write_field(&entity, IntentDueField::at("IntentDueField"), input.due),
            writer.write_field(
                &entity,
                IntentLifecycleField::at("IntentLifecycleField"),
                input.lifecycle.clone(),
            ),
            writer.write_field(
                &entity,
                IntentInputField::at("IntentInputField"),
                input.input.clone(),
            ),
            writer.write_field(
                &entity,
                IntentGateField::at("IntentGateField"),
                input.gate.clone(),
            ),
        ] {
            if let Err(error) = result {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
            }
        }
        HandlerResult::Completed(AmendTemporalResult {
            revision: input.revision,
        })
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
) -> ApplicationSchemaDeclarationBuilder<TemporalHostSchema> {
    schema
        .application_query_binding::<TemporalIntentQueryBinding>()
        .application_mutation_binding::<AmendTemporalBinding>()
}

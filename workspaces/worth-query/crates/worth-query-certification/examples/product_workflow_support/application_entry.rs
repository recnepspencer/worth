use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationCandidateRequirements, ApplicationMutationBinding,
            ApplicationMutationFieldScope, ApplicationMutationIntent, NoApplicationMutationOutputs,
            NoApplicationMutationSource,
        },
        application_query::{
            ApplicationQueryBinding, ApplicationQueryBindingLimits, ApplicationQueryFieldScope,
            ApplicationQueryIntent, ApplicationQueryParameterSet,
        },
        application_schema::{
            ApplicationFieldRef, ApplicationPrincipalBindingRef,
            ApplicationSchemaDeclarationBuilder, EqualityPredicate, NoApplicationUnit, ReadOnly,
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
    IntentFacts, IntentGateField, IntentIdentityField, IntentInputField, IntentLifecycleField,
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

type TemporalIntentQueryScope = ApplicationQueryFieldScope<
    TemporalHostSchema,
    TemporalIntent,
    IntentFacts,
    IntentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

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
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 64);

    fn scope_field() -> ApplicationFieldRef<
        TemporalHostSchema,
        TemporalIntent,
        IntentFacts,
        IntentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        IntentIdentityField::reference()
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
        TemporalIntentQueryScope::new(IntentIdentityField::reference(), self.identity)
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

type AmendTemporalScope = ApplicationMutationFieldScope<
    TemporalHostSchema,
    TemporalIntent,
    IntentFacts,
    IntentIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<TemporalHostSchema> for AmendTemporalBinding {
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
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            worth_query_host::facade::declaration::application_operation::ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 5, 0),
            worth_query_host::facade::declaration::application_operation::ApplicationCandidateResourceCeiling::bounded(1024, 128),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        digest(&key.to_le_bytes())
    }

    fn input_identity(input: &AmendTemporalInput) -> [u8; 32] {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&input.revision.to_le_bytes());
        bytes.extend_from_slice(&input.due.to_le_bytes());
        bytes.extend_from_slice(input.lifecycle.as_bytes());
        bytes.extend_from_slice(input.input.as_bytes());
        bytes.extend_from_slice(input.gate.as_bytes());
        digest(&bytes)
    }

    fn scope_field() -> ApplicationFieldRef<
        TemporalHostSchema,
        TemporalIntent,
        IntentFacts,
        IntentIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        IntentIdentityField::reference()
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
        AmendTemporalScope::new(IntentIdentityField::reference(), self.identity.clone())
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
        let entity =
            match reader.resolve_entity(IntentIdentityField::reference(), "intent-1".to_owned()) {
                Ok(entity) => entity,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        match reader.field(&entity, IntentRevisionField::reference()) {
            Ok(Some(revision)) if input.revision > revision => {}
            Ok(_) => return HandlerResult::DomainDenied(AmendTemporalDenial::RevisionMustAdvance),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        for read in [
            reader
                .field(&entity, IntentDueField::reference())
                .map(|_| ()),
            reader
                .field(&entity, IntentLifecycleField::reference())
                .map(|_| ()),
            reader
                .field(&entity, IntentInputField::reference())
                .map(|_| ()),
            reader
                .field(&entity, IntentGateField::reference())
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

    fn candidate_requirements(
        &self,
        _: &AmendTemporalInput,
        _: &WorthQueryInvariantMutationTarget<TemporalHostSchema, TemporalIntent>,
    ) -> ApplicationCandidateRequirements {
        AmendTemporalBinding::CANDIDATES
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
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        for result in [
            writer.write_field(&entity, IntentRevisionField::reference(), input.revision),
            writer.write_field(&entity, IntentDueField::reference(), input.due),
            writer.write_field(
                &entity,
                IntentLifecycleField::reference(),
                input.lifecycle.clone(),
            ),
            writer.write_field(&entity, IntentInputField::reference(), input.input.clone()),
            writer.write_field(&entity, IntentGateField::reference(), input.gate.clone()),
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

fn digest(bytes: &[u8]) -> [u8; 32] {
    let mut digest = [0_u8; 32];
    for (index, byte) in bytes.iter().enumerate() {
        let slot = index % digest.len();
        digest[slot] = digest[slot].rotate_left(1) ^ byte.wrapping_add(index as u8);
    }
    digest
}

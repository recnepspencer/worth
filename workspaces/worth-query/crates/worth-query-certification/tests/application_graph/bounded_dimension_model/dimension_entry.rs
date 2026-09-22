//! The public request surface a caller uses to set and read a part dimension.
//!
//! One mutation binding writes the dimension and one query binding reports it.
//! Both are declared on the schema, so every court step goes through the same
//! ordinary application entry an external consumer would use.

use worth_query_host::facade::{
    declaration::{
        application_operation::{
            ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
            ApplicationCandidateResourceCeiling, ApplicationMutationBinding,
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
    BoundedDimensionSchema, ExternalMapping, Part, PartDimensionField, PartDimensionQuery,
    PartDimensionRowBinding, PartFacts, PartIdentityField, PartPrincipalBinding,
    PartQueryParametersBinding, Principal, SetPartDimension, SetPartDimensionInput,
    SetPartDimensionInputBinding,
};

fn migration_candidate_counts() -> &'static std::sync::Mutex<std::collections::BTreeMap<u64, usize>>
{
    static COUNTS: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeMap<u64, usize>>> =
        std::sync::OnceLock::new();
    COUNTS.get_or_init(Default::default)
}

pub fn reset_candidate_count(dimension: u64) {
    migration_candidate_counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&dimension);
}

pub fn candidate_count(dimension: u64) -> usize {
    migration_candidate_counts()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&dimension)
        .copied()
        .unwrap_or(0)
}

/// The one part every host in this court seeds and both programs act on.
pub const PART_IDENTITY: &str = "part-1";
/// Reserved fixture value whose candidate-authoring count proves migration recovery does not replay.
pub const MIGRATION_CANDIDATE_PROBE_DIMENSION: u64 = 17;

#[derive(Clone, Debug)]
pub struct PartDimensionRead {
    pub identity: String,
}

worth_query_structured_value_binding!(pub PartDimensionReadBinding for PartDimensionRead {
    identity: "worth.query.certification.bounded-dimension.read.v1"
});

pub struct PartDimensionQueryBinding;

type PartDimensionQueryScope = ApplicationQueryFieldScope<
    BoundedDimensionSchema,
    Part,
    PartFacts,
    PartIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationQueryBinding<BoundedDimensionSchema> for PartDimensionQueryBinding {
    type Input = PartDimensionRead;
    type InputBinding = PartDimensionReadBinding;
    type Query = PartDimensionQuery;
    type ParameterBinding = PartQueryParametersBinding;
    type ResultBinding = PartDimensionRowBinding;
    type ScopeBinding = PartDimensionQueryScope;
    type PrincipalBinding = PartPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str = "worth.query.certification.bounded-dimension.read-binding.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 64);

    fn scope_field() -> ApplicationFieldRef<
        BoundedDimensionSchema,
        Part,
        PartFacts,
        PartIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        PartIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BoundedDimensionSchema,
        PartPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PartPrincipalBinding::reference()
    }
}

impl ApplicationQueryIntent<BoundedDimensionSchema> for PartDimensionRead {
    type Binding = PartDimensionQueryBinding;

    fn parameters(&self) -> ApplicationQueryParameterSet<PartDimensionQuery> {
        ApplicationQueryParameterSet::new()
    }

    fn into_scope(self) -> PartDimensionQueryScope {
        PartDimensionQueryScope::new(PartIdentityField::reference(), self.identity)
    }
}

/// The request to give one part an exact dimension.
#[derive(Clone, Debug)]
pub struct SetPartDimensionIntent {
    pub input: SetPartDimensionInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartDimensionWritten {
    pub dimension: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetPartDimensionDenial {
    DimensionUnchanged,
}

worth_query_structured_value_binding!(pub PartDimensionWrittenBinding for PartDimensionWritten {
    identity: "worth.query.certification.bounded-dimension.written.v1"
});
worth_query_structured_value_binding!(pub SetPartDimensionDenialBinding for SetPartDimensionDenial {
    identity: "worth.query.certification.bounded-dimension.write-denial.v1"
});

pub struct SetPartDimensionBinding;

type SetPartDimensionScope = ApplicationMutationFieldScope<
    BoundedDimensionSchema,
    Part,
    PartFacts,
    PartIdentityField,
    String,
    ReadOnly,
    NoApplicationUnit,
>;

impl ApplicationMutationBinding<BoundedDimensionSchema> for SetPartDimensionBinding {
    type Input = SetPartDimensionInput;
    type InputBinding = SetPartDimensionInputBinding;
    type Result = PartDimensionWritten;
    type ResultBinding = PartDimensionWrittenBinding;
    type IdempotencyKey = u64;
    type Operation = SetPartDimension;
    type Decision = WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>;
    type Denial = SetPartDimensionDenial;
    type DenialBinding = SetPartDimensionDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = SetPartDimensionScope;
    type PrincipalBinding = PartPrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = NoApplicationMutationSource;

    const IDENTITY: &'static str = "worth.query.certification.bounded-dimension.set-binding.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.bounded-dimension.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.bounded-dimension.command.v1";
    const CANDIDATES: ApplicationCandidateRequirements =
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 2, 0),
            ApplicationCandidateResourceCeiling::bounded(1024, 1024),
        );

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        content_identity(&key.to_le_bytes())
    }

    fn input_identity(input: &SetPartDimensionInput) -> [u8; 32] {
        let mut bytes = input.identity.as_bytes().to_vec();
        bytes.extend_from_slice(&input.dimension.to_le_bytes());
        content_identity(&bytes)
    }

    fn scope_field() -> ApplicationFieldRef<
        BoundedDimensionSchema,
        Part,
        PartFacts,
        PartIdentityField,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        PartIdentityField::reference()
    }

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BoundedDimensionSchema,
        PartPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PartPrincipalBinding::reference()
    }
}

impl ApplicationMutationIntent<BoundedDimensionSchema> for SetPartDimensionIntent {
    type Binding = SetPartDimensionBinding;

    fn input(&self) -> &SetPartDimensionInput {
        &self.input
    }

    fn scope_binding(&self) -> SetPartDimensionScope {
        SetPartDimensionScope::new(PartIdentityField::reference(), self.input.identity.clone())
    }
}

pub struct SetPartDimensionHandler;

impl OperationHandler<BoundedDimensionSchema, SetPartDimensionBinding> for SetPartDimensionHandler {
    fn decide(
        &self,
        input: &SetPartDimensionInput,
        reader: &mut DecisionReader<'_, '_, '_, BoundedDimensionSchema, SetPartDimensionBinding>,
    ) -> HandlerResult<
        WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        SetPartDimensionDenial,
    > {
        let part =
            match reader.resolve_entity(PartIdentityField::reference(), input.identity.clone()) {
                Ok(part) => part,
                Err(error) => return HandlerResult::ExecutionDenied(error),
            };
        match reader.field(&part, PartDimensionField::reference()) {
            Ok(Some(current)) if current == input.dimension => {
                return HandlerResult::DomainDenied(SetPartDimensionDenial::DimensionUnchanged)
            }
            Ok(_) => {}
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&part) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &SetPartDimensionInput,
        _: &WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
    ) -> ApplicationCandidateRequirements {
        SetPartDimensionBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        input: &SetPartDimensionInput,
        target: WorthQueryInvariantMutationTarget<BoundedDimensionSchema, Part>,
        writer: &mut CandidateWriter<'_, BoundedDimensionSchema, SetPartDimensionBinding>,
    ) -> HandlerResult<PartDimensionWritten, SetPartDimensionDenial> {
        *migration_candidate_counts()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(input.dimension)
            .or_default() += 1;
        let part = match writer.projected_entity(&target) {
            Ok(part) => part,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        if let Err(error) =
            writer.write_field(&part, PartDimensionField::reference(), input.dimension)
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error));
        }
        HandlerResult::Completed(PartDimensionWritten {
            dimension: input.dimension,
        })
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema>,
) -> ApplicationSchemaDeclarationBuilder<BoundedDimensionSchema> {
    schema
        .application_query_binding::<PartDimensionQueryBinding>()
        .application_mutation_binding::<SetPartDimensionBinding>()
}

/// A stable content identity for one authored value. Nothing in this court
/// depends on the digest being cryptographic; it only has to separate two
/// different requests so idempotency cannot silently merge them.
fn content_identity(bytes: &[u8]) -> [u8; 32] {
    let mut identity = [0_u8; 32];
    let mut accumulator = 0xcbf2_9ce4_8422_2325_u64;
    for (index, byte) in bytes.iter().enumerate() {
        accumulator = (accumulator ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        identity[index % identity.len()] ^= (accumulator >> ((index % 8) * 8)) as u8;
    }
    identity
}

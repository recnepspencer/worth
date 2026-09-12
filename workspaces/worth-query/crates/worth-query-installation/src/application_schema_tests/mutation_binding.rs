use super::*;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
        ApplicationCandidateResourceCeiling, ApplicationMutationBinding,
        ApplicationMutationFieldScope, NoApplicationMutationOutputs,
    },
    application_schema::{
        ApplicationFieldMarkerIdentity, ApplicationFieldPresence,
        ApplicationStructuredValueBinding, DeclaredApplicationFieldValue, NoApplicationUnit,
        U64ApplicationValueBinding,
    },
    domain_computation::{WorthQueryResourceDimension, WorthQuerySemanticScaleAxis},
};

struct MutationSchema;
struct MutationDecision;
struct MutationDenial;
worth_query_declaration::worth_query_structured_value_binding!(MutationDenialBinding for MutationDenial {
    identity: "worth.query.installation-test.mutation-denial.v1"
});
struct ChangedResult;
struct ChangedOperation;
struct ChangedScopeField;

impl ApplicationSchema for MutationSchema {
    const OWNER: &'static str = TestSchema::OWNER;
    const NAME: &'static str = TestSchema::NAME;
    const MAJOR: u32 = TestSchema::MAJOR;
    const MINOR: u32 = TestSchema::MINOR;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        test_schema_members::<Self>(None)
            .application_mutation_binding::<MutationBinding>()
            .build()
    }
}

impl ApplicationOperationMarkerIdentity<MutationSchema> for ChangedOperation {
    type InputBinding = TestInputBinding;
    const IDENTIFIER: &'static str = "ChangedOperation";
}

impl
    ApplicationFieldMarkerIdentity<
        MutationSchema,
        FixtureEntity<MutationSchema>,
        FixtureIdentityAspect<MutationSchema>,
    > for ChangedScopeField
{
    const IDENTIFIER: &'static str = "ChangedScopeField";
}

impl DeclaredApplicationFieldValue for ChangedScopeField {
    type Value = u64;
    type Binding = U64ApplicationValueBinding;
    const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
}

worth_query_declaration::worth_query_structured_value_binding!(
    ChangedResultBinding for ChangedResult {
        identity: "worth.query.installation-test.changed-result.v1"
    }
);

type MutationScope<Field> = ApplicationMutationFieldScope<
    MutationSchema,
    FixtureEntity<MutationSchema>,
    FixtureIdentityAspect<MutationSchema>,
    Field,
    u64,
    ReadOnly,
    NoApplicationUnit,
>;

macro_rules! mutation_binding {
    ($binding:ident, $identity:literal, $operation:ty, $result:ty, $result_binding:ty, $decision:ty, $denial_binding:ty, $handler_identity:literal, $field:ty, $principal:expr, $bytes:expr) => {
        struct $binding;

        impl ApplicationMutationBinding<MutationSchema> for $binding {
            type Input = TestInput;
            type InputBinding = TestInputBinding;
            type Operation = $operation;
            type Result = $result;
            type ResultBinding = $result_binding;
            type IdempotencyKey = u64;
            type Decision = $decision;
            type Denial = <$denial_binding as ApplicationStructuredValueBinding>::Value;
            type DenialBinding = $denial_binding;
            type Output = NoApplicationMutationOutputs;
            type ScopeBinding = MutationScope<$field>;
            type PrincipalBinding = PrincipalBinding;
            type Mapping = FixtureEntity<MutationSchema>;
            type Principal = FixtureEntity<MutationSchema>;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;

            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str = $handler_identity;
            const IDEMPOTENCY_IDENTITY: &'static str =
                "worth.query.installation-test.idempotency.v1";
            const CANDIDATES: ApplicationCandidateRequirements =
                ApplicationCandidateRequirements::fixed_shape(
                    ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 1, 0),
                    ApplicationCandidateResourceCeiling::bounded($bytes, 4),
                );

            fn idempotency_key_identity(key: &Self::IdempotencyKey) -> [u8; 32] {
                let mut identity = [0; 32];
                identity[..8].copy_from_slice(&key.to_be_bytes());
                identity
            }

            fn input_identity(_input: &Self::Input) -> [u8; 32] {
                [1; 32]
            }

            fn scope_field() -> ApplicationFieldRef<
                MutationSchema,
                FixtureEntity<MutationSchema>,
                FixtureIdentityAspect<MutationSchema>,
                $field,
                u64,
                ReadOnly,
                EqualityPredicate,
                NoApplicationUnit,
            > {
                ApplicationFieldRef::from_schema_types()
            }

            fn principal_binding() -> ApplicationPrincipalBindingRef<
                MutationSchema,
                PrincipalBinding,
                FixtureEntity<MutationSchema>,
                FixtureEntity<MutationSchema>,
                u64,
                U64ApplicationValueBinding,
            > {
                $principal
            }
        }
    };
}

mutation_binding!(
    MutationBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    128
);
mutation_binding!(
    ChangedOperationBinding,
    "worth.query.installation-test.mutation-binding.v1",
    ChangedOperation,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    128
);
mutation_binding!(
    ChangedHandlerBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.changed-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    128
);
mutation_binding!(
    ChangedScopeBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    ChangedScopeField,
    test_principal_binding::<MutationSchema>(),
    128
);
mutation_binding!(
    ChangedCandidateBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    129
);
mutation_binding!(
    ChangedPrincipalMutationBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    changed_principal_binding(),
    128
);
mutation_binding!(
    ChangedResultMutationBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    ChangedResult,
    ChangedResultBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    128
);
mutation_binding!(
    MissingMutationBinding,
    "worth.query.installation-test.missing-mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    128
);

worth_query_declaration::worth_query_structured_value_binding!(ChangedDenialBinding for MutationDenial {
    identity: "worth.query.installation-test.changed-denial.v1"
});
mutation_binding!(
    ChangedDenialMutationBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    ChangedDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    128
);

fn changed_principal_binding() -> ApplicationPrincipalBindingRef<
    MutationSchema,
    PrincipalBinding,
    FixtureEntity<MutationSchema>,
    FixtureEntity<MutationSchema>,
    u64,
    U64ApplicationValueBinding,
> {
    let installed = test_principal_binding::<MutationSchema>();
    let mapping_identity = ApplicationFieldRef::<
        MutationSchema,
        FixtureEntity<MutationSchema>,
        FixtureIdentityAspect<MutationSchema>,
        FixtureExternalIdentityField<MutationSchema>,
        WorthQueryExternalPrincipalIdentity,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    let mapping_status = ApplicationFieldRef::<
        MutationSchema,
        FixtureEntity<MutationSchema>,
        FixtureIdentityAspect<MutationSchema>,
        FixtureMappingStatusField<MutationSchema>,
        WorthQueryPrincipalMappingStatus,
        ReadWrite,
        NoEqualityPredicate,
    >::from_schema_types();
    let principal_identity = ApplicationFieldRef::<
        MutationSchema,
        FixtureEntity<MutationSchema>,
        FixtureIdentityAspect<MutationSchema>,
        FixturePrincipalIdentityField<MutationSchema>,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_types();
    let target = ApplicationRelationRef::<
        MutationSchema,
        MappingTarget,
        FixtureEntity<MutationSchema>,
        FixtureEntity<MutationSchema>,
    >::from_schema_identifiers(
        installed.target_relation(),
        installed.mapping_entity(),
        installed.principal_entity(),
        worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
    );
    ApplicationPrincipalBindingRef::from_requirements(
        "ChangedPrincipalBinding",
        ApplicationPrincipalBindingRequirements {
            mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                mapping_identity,
            ),
            mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                mapping_status,
            ),
            target: ApplicationPrincipalTargetRequirement::from_relation(target),
            principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                principal_identity,
            ),
        },
    )
}

mod contract_checks;

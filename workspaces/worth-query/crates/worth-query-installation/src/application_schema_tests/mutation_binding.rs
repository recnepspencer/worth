use super::*;
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationFieldScope,
        ApplicationMutationOutputContract, ApplicationMutationOutputPostureSet,
        ApplicationMutationOutputRoleDescriptor, ApplicationMutationOutputRoleFamilyDescriptor,
        NoApplicationMutationSource, WorthQueryApplicationDeclaredOutputRoleFamily,
        WorthQueryApplicationOutputRoleFamily,
    },
    application_schema::{
        ApplicationFieldPresence, ApplicationStructuredValueBinding, U64ApplicationValueBinding,
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
struct MutationOutputFamilies;
struct ChangedMutationOutputFamilies;

struct OutputFamily;
struct ChangedOutputFamily;

impl WorthQueryApplicationOutputRoleFamily for OutputFamily {
    type Schema = MutationSchema;
    type Contract = MutationOutputFamilies;
    type Entity = FixtureEntity;
    const PREFIX: &'static str = "output.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 1;
}

impl WorthQueryApplicationOutputRoleFamily for ChangedOutputFamily {
    type Schema = MutationSchema;
    type Contract = ChangedMutationOutputFamilies;
    type Entity = FixtureEntity;
    const PREFIX: &'static str = "changed-output.";
    const POSTURES: ApplicationMutationOutputPostureSet = ApplicationMutationOutputPostureSet::ALL;
    const MINIMUM: usize = 1;
}

impl ApplicationMutationOutputContract<MutationSchema> for MutationOutputFamilies {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[<OutputFamily as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR];
}

impl ApplicationMutationOutputContract<MutationSchema> for ChangedMutationOutputFamilies {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[<ChangedOutputFamily as WorthQueryApplicationDeclaredOutputRoleFamily>::DESCRIPTOR];
}

impl ApplicationSchema for MutationSchema {
    const OWNER: &'static str = TestSchema::OWNER;
    const NAME: &'static str = <TestSchema as ApplicationSchema>::NAME;
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

worth_query_declaration::worth_query_structured_value_binding!(
    ChangedResultBinding for ChangedResult {
        identity: "worth.query.installation-test.changed-result.v1"
    }
);

type MutationScope<Field> = ApplicationMutationFieldScope<Field>;

macro_rules! mutation_binding {
    ($binding:ident, $identity:literal, $operation:ty, $result:ty, $result_binding:ty, $decision:ty, $denial_binding:ty, $handler_identity:literal, $field:expr, $principal:expr, $output:ty) => {
        struct $binding;

        impl ApplicationMutationBinding<MutationSchema> for $binding {
    fn workflow_execution_requirements() -> worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements {
        worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

            type Input = TestInput;
            type InputBinding = TestInputBinding;
            type Operation = $operation;
            type Result = $result;
            type ResultBinding = $result_binding;
            type IdempotencyKey = u64;
            type Decision = $decision;
            type Denial = <$denial_binding as ApplicationStructuredValueBinding>::Value;
            type DenialBinding = $denial_binding;
            type Output = $output;
            type ScopeBinding = MutationScope<FixturePrincipalIdentityField>;
            type PrincipalBinding = PrincipalBinding;
            type Mapping = FixtureEntity;
            type Principal = FixtureEntity;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = NoApplicationMutationSource;

            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str = $handler_identity;
            const IDEMPOTENCY_IDENTITY: &'static str =
                "worth.query.installation-test.idempotency.v1";

            fn scope_field() -> FixturePrincipalIdentityField {
                $field
            }

            fn principal_binding() -> ApplicationPrincipalBindingRef<
                MutationSchema,
                PrincipalBinding,
                FixtureEntity,
                FixtureEntity,
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
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    test_principal_binding::<MutationSchema>(),
    MutationOutputFamilies
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
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    test_principal_binding::<MutationSchema>(),
    MutationOutputFamilies
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
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    test_principal_binding::<MutationSchema>(),
    MutationOutputFamilies
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
    FixturePrincipalIdentityField::at("ChangedScopeField"),
    test_principal_binding::<MutationSchema>(),
    MutationOutputFamilies
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
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    changed_principal_binding(),
    MutationOutputFamilies
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
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    test_principal_binding::<MutationSchema>(),
    MutationOutputFamilies
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
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    test_principal_binding::<MutationSchema>(),
    MutationOutputFamilies
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
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    test_principal_binding::<MutationSchema>(),
    MutationOutputFamilies
);

mutation_binding!(
    ChangedOutputMutationBinding,
    "worth.query.installation-test.mutation-binding.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "worth.query.installation-test.mutation-handler.v1",
    FixturePrincipalIdentityField::at("PrincipalIdentityField"),
    test_principal_binding::<MutationSchema>(),
    ChangedMutationOutputFamilies
);

fn changed_principal_binding() -> ApplicationPrincipalBindingRef<
    MutationSchema,
    PrincipalBinding,
    FixtureEntity,
    FixtureEntity,
    u64,
    U64ApplicationValueBinding,
> {
    let installed = test_principal_binding::<MutationSchema>();
    let mapping_identity =
        worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
            InstallationFields,
            FixtureEntity,
            FixtureIdentityAspect,
            WorthQueryExternalPrincipalIdentity,
        >::at("ExternalIdentityField");
    let mapping_status =
        worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
            InstallationFields,
            FixtureEntity,
            FixtureIdentityAspect,
            WorthQueryPrincipalMappingStatus,
        >::at("MappingStatusField");
    let principal_identity =
        worth_query_declaration::facade::application_schema::fragment::FieldSelection::<
            InstallationFields,
            FixtureEntity,
            FixtureIdentityAspect,
            u64,
        >::at("PrincipalIdentityField");
    let target = ApplicationRelationRef::<
        MutationSchema,
        MappingTarget,
        FixtureEntity,
        FixtureEntity,
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
mod correction_binding;

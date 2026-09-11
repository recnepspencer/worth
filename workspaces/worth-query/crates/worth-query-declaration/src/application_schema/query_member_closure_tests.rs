use worth_foundational::facade::ScalarAspectType;

use super::validate_application_query_members;
use crate::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryDisclosurePosture,
        ApplicationQueryLaneEligibility, ApplicationQueryResultFieldRef,
        ApplicationQueryResultShapeBuilder, ErasedApplicationQueryDefinition,
        WorthQueryPortableApplicationQueryDisclosureParts,
    },
    application_schema::{
        ApplicationAbilityRef, ApplicationEntityRef, ApplicationFieldRef,
        ApplicationSchemaDeclarationDenial, ApplicationSchemaMember, EqualityPredicate, ReadOnly,
    },
};

struct TestSchema;
struct Account;
struct AccountFacts;
struct AccountId;
struct AccountParameters;
struct AccountResult;
struct AccountIdSlot;
struct ViewAccount;

crate::worth_query_structured_value_binding!(AccountQueryParametersBinding for AccountParameters { identity: "AccountParameters" });
crate::worth_query_structured_value_binding!(AccountQueryResultBinding for AccountResult { identity: "worth.query.test.member-closure.account-result.v1" });
crate::worth_query_application_query!(
    AccountQuery for TestSchema,
    identity "AccountQuery",
    parameters AccountQueryParametersBinding,
    result AccountQueryResultBinding,
    scope Account => "Account",
    name "account"
);
worth_query_portable_type!(AccountResult => "worth.query.test.member-closure.account-result.v1");
worth_query_portable_type!(AccountIdSlot => "worth.query.test.member-closure.account-id-slot.v1");

impl crate::application_schema::DeclaredApplicationFieldValue for AccountId {
    type Value = u64;
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const PRESENCE: crate::application_schema::ApplicationFieldPresence =
        crate::application_schema::ApplicationFieldPresence::Required;
}

impl crate::application_schema::RequiredApplicationFieldValue for AccountId {}

#[test]
fn valid_query_dependencies_and_projection_types_close() {
    assert_eq!(
        validate_application_query_members(&members(query("AccountId", "id"))),
        Ok(())
    );
}

#[test]
fn forged_projection_field_or_value_type_cannot_enter_package_meaning() {
    assert_eq!(
        validate_application_query_members(&members(query("MissingField", "id"))),
        Err(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    );
}

#[test]
fn one_schema_cannot_install_two_meanings_under_one_query_name() {
    let mut duplicated = dependencies();
    duplicated.push(ApplicationSchemaMember::ApplicationQuery {
        definition: query("AccountId", "id"),
    });
    duplicated.push(ApplicationSchemaMember::ApplicationQuery {
        definition: query("AccountId", "other_id"),
    });
    assert_eq!(
        validate_application_query_members(&duplicated),
        Err(ApplicationSchemaDeclarationDenial::DuplicateApplicationQuery)
    );
}

#[test]
fn governed_query_requires_its_exact_installed_ability_and_policy() {
    let governed = governed_query();
    assert_eq!(
        validate_application_query_members(&members(governed.clone())),
        Err(ApplicationSchemaDeclarationDenial::MissingAbilityDependency)
    );
    let mut ability_only = dependencies();
    ability_only.push(ApplicationSchemaMember::Ability {
        ability: "ViewAccount".to_string(),
        scope_entity: "Account".to_string(),
    });
    ability_only.push(ApplicationSchemaMember::ApplicationQuery {
        definition: governed.clone(),
    });
    assert_eq!(
        validate_application_query_members(&ability_only),
        Err(ApplicationSchemaDeclarationDenial::MissingAbilityPolicyDependency)
    );
    ability_only.insert(
        4,
        ApplicationSchemaMember::AbilityPolicy {
            ability: "ViewAccount".to_string(),
            scope_entity: "Account".to_string(),
            policy: "AccountVisibility".to_string(),
            paths: Vec::new(),
        },
    );
    assert_eq!(validate_application_query_members(&ability_only), Ok(()));
}

#[test]
fn governed_disclosure_requires_its_exact_installed_capability() {
    let mut parts = query("AccountId", "id").into_parts();
    parts.disclosure = ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Governed,
            classification: "restricted".to_owned(),
            capability_name: Some("ReadRestricted".to_owned()),
            capability_type: Some(
                crate::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.query.test.read-restricted-capability.v1".to_owned(),
                ),
            ),
            rules: Vec::new(),
        },
    );
    let reconstructed = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);

    assert_eq!(
        validate_application_query_members(&members(reconstructed)),
        Err(ApplicationSchemaDeclarationDenial::MissingApplicationQueryDependency)
    );
}

fn members(definition: ErasedApplicationQueryDefinition) -> Vec<ApplicationSchemaMember> {
    let mut members = dependencies();
    members.push(ApplicationSchemaMember::ApplicationQuery { definition });
    members
}

fn dependencies() -> Vec<ApplicationSchemaMember> {
    vec![
        ApplicationSchemaMember::Entity {
            entity: "Account".to_string(),
        },
        ApplicationSchemaMember::Aspect {
            entity: "Account".to_string(),
            aspect: "AccountFacts".to_string(),
            identity: worth_foundational::facade::AspectIdentity(0x91613005),
            revision: worth_foundational::facade::AspectContractRevision(1),
        },
        ApplicationSchemaMember::Field {
            entity: "Account".to_string(),
            aspect: "AccountFacts".to_string(),
            field: "AccountId".to_string(),
            presence: crate::application_schema::ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::UInt64,
            value_type:
                <u64 as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY
                    .as_str()
                    .to_string(),
            unit: None,
            frame: None,
            writable: false,
            equality_queryable: true,
        },
    ]
}

fn query(field_name: &'static str, output_name: &'static str) -> ErasedApplicationQueryDefinition {
    let field = ApplicationFieldRef::<
        TestSchema,
        Account,
        AccountFacts,
        AccountId,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_identifiers("Account", "AccountFacts", field_name);
    build_query(output_name, field)
}

fn governed_query() -> ErasedApplicationQueryDefinition {
    let field = ApplicationFieldRef::<
        TestSchema,
        Account,
        AccountFacts,
        AccountId,
        u64,
        ReadOnly,
        EqualityPredicate,
    >::from_schema_identifiers("Account", "AccountFacts", "AccountId");
    let account = ApplicationEntityRef::<TestSchema, Account>::from_schema_identifier("Account");
    let shape = ApplicationQueryResultShapeBuilder::<
        TestSchema,
        AccountQuery,
        Account,
        AccountResult,
        AccountQueryResultBinding,
    >::new(account)
    .field(ApplicationQueryResultFieldRef::<
        AccountQuery,
        AccountIdSlot,
        TestSchema,
        Account,
        AccountFacts,
        AccountId,
        u64,
        ReadOnly,
        EqualityPredicate,
        crate::application_schema::NoApplicationUnit,
    >::new("id", field))
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountQuery::reference())
        .root(account)
        .scope(account)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(
            ApplicationAbilityRef::<TestSchema, ViewAccount, Account>::from_schema_identifiers(
                "ViewAccount",
                "Account",
            ),
        )
        .build()
        .unwrap()
        .into_erased()
}

fn build_query(
    output_name: &'static str,
    field: ApplicationFieldRef<
        TestSchema,
        Account,
        AccountFacts,
        AccountId,
        u64,
        ReadOnly,
        EqualityPredicate,
    >,
) -> ErasedApplicationQueryDefinition {
    let account = ApplicationEntityRef::<TestSchema, Account>::from_schema_identifier("Account");
    let result_field = ApplicationQueryResultFieldRef::<
        AccountQuery,
        AccountIdSlot,
        TestSchema,
        Account,
        AccountFacts,
        AccountId,
        u64,
        ReadOnly,
        EqualityPredicate,
        crate::application_schema::NoApplicationUnit,
    >::new(output_name, field);
    let shape = ApplicationQueryResultShapeBuilder::<
        TestSchema,
        AccountQuery,
        Account,
        AccountResult,
        AccountQueryResultBinding,
    >::new(account)
    .field(result_field)
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountQuery::reference())
        .root(account)
        .scope(account)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
        .into_erased()
}

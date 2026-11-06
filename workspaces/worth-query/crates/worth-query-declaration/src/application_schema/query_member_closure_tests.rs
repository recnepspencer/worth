use worth_foundational::facade::ScalarAspectType;

use super::{shape_is_closed, validate_application_query_members};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use crate::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryDisclosurePosture, ApplicationQueryLaneEligibility,
        ApplicationQueryPredicate, ApplicationQueryResultFieldRef, ApplicationQueryResultRelation,
        ApplicationQueryResultShape, ApplicationQueryResultShapeBuilder,
        ApplicationQueryResultTraversalDirection, ErasedApplicationQueryDefinition,
        WorthQueryPortableApplicationQueryDisclosureParts,
        WorthQueryPortableApplicationQueryResultRelationParts,
        WorthQueryPortableApplicationQueryResultShapeParts,
    },
    application_schema::{
        ApplicationAbilityRef, ApplicationEntityRef, ApplicationSchemaDeclarationDenial,
        ApplicationSchemaMember,
    },
};

struct TestSchema;
struct Account;
struct AccountFacts;
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
fn relation_predicate_field_must_be_declared_and_equality_queryable() {
    let mut members = relation_predicate_dependencies(false);
    let shape = relation_predicate_shape();
    assert!(!shape_is_closed(&members, "PredicateQuery", &shape));

    let ApplicationSchemaMember::Field {
        equality_queryable, ..
    } = &mut members[5]
    else {
        panic!("fixture field must occupy the declared slot");
    };
    *equality_queryable = true;
    assert!(shape_is_closed(&members, "PredicateQuery", &shape));
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
            absence: worth_foundational::facade::AbsenceLaw::Required,
            entity: "Account".to_string(),
            aspect: AccountId::at("AccountId").aspect().to_string(),
            identity: worth_foundational::facade::AspectIdentity(0x91613005),
            revision: worth_foundational::facade::AspectContractRevision(1),
        },
        ApplicationSchemaMember::Field {
            content_format: None,
            entity: "Account".to_string(),
            aspect: AccountId::at("AccountId").aspect().to_string(),
            field: "AccountId".to_string(),
            presence: crate::application_schema::ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::UInt64,
            canonical: None,
            value_type:
                <u64 as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY
                    .as_str()
                    .to_string(),
            unit: None,
            frame: None,
            writable: false,
            equality_queryable: true,
            unique: false,
        },
    ]
}

fn relation_predicate_dependencies(equality_queryable: bool) -> Vec<ApplicationSchemaMember> {
    let mut members = dependencies();
    members.extend([
        ApplicationSchemaMember::Entity {
            entity: "Activity".to_owned(),
        },
        ApplicationSchemaMember::Aspect {
            absence: worth_foundational::facade::AbsenceLaw::Required,
            entity: "Activity".to_owned(),
            aspect: "ActivityFacts".to_owned(),
            identity: worth_foundational::facade::AspectIdentity(0x91613006),
            revision: worth_foundational::facade::AspectContractRevision(1),
        },
        ApplicationSchemaMember::Field {
            content_format: None,
            entity: "Activity".to_owned(),
            aspect: "ActivityFacts".to_owned(),
            field: "ActivityId".to_owned(),
            presence: crate::application_schema::ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::UInt64,
            canonical: None,
            value_type:
                <u64 as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY
                    .as_str()
                    .to_owned(),
            unit: None,
            frame: None,
            writable: false,
            equality_queryable,
            unique: false,
        },
        ApplicationSchemaMember::Relation {
            relation: "AccountAllActivity".to_owned(),
            from: "Account".to_owned(),
            to: "Activity".to_owned(),
            integrity: crate::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into(),
        },
    ]);
    members
}

fn relation_predicate_shape() -> ApplicationQueryResultShape {
    let identity = |value: &str| WorthQueryPortableTypeIdentity::from_untrusted(value.to_owned());
    let nested = ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: identity("PredicateQuery"),
            root_entity: "Activity".to_owned(),
            result_type: identity("ActivityResult"),
            fields: Vec::new(),
            relations: Vec::new(),
        },
    );
    let relation = ApplicationQueryResultRelation::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultRelationParts {
            query_type: identity("PredicateQuery"),
            slot_type: identity("ActivitySlot"),
            relation: "AccountAllActivity".to_owned(),
            from: "Account".to_owned(),
            to: "Activity".to_owned(),
            direction: ApplicationQueryResultTraversalDirection::Forward,
            output_name: "activity".to_owned(),
            cardinality: ApplicationQueryCardinality::ExactlyOne,
            predicate: Some(ApplicationQueryPredicate::from_untrusted_fields(
                "Activity".to_owned(),
                "ActivityFacts".to_owned(),
                "ActivityId".to_owned(),
                "activity".to_owned(),
                ScalarAspectType::UInt64,
            )),
            nested_shape: nested,
        },
    );
    ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: identity("PredicateQuery"),
            root_entity: "Account".to_owned(),
            result_type: identity("AccountResult"),
            fields: Vec::new(),
            relations: vec![relation],
        },
    )
}

fn query(field_name: &'static str, output_name: &'static str) -> ErasedApplicationQueryDefinition {
    let _field = AccountId::at("AccountId");
    build_query(output_name, AccountId::at(field_name))
}

fn governed_query() -> ErasedApplicationQueryDefinition {
    let field = AccountId::at("AccountId");
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
        AccountId,
    >::new("id", field))
    .build();
    ApplicationQueryDefinitionBuilder::declare(AccountQuery::reference())
        .root(account)
        .scope(account)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
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

fn build_query(output_name: &'static str, field: AccountId) -> ErasedApplicationQueryDefinition {
    let account = ApplicationEntityRef::<TestSchema, Account>::from_schema_identifier("Account");
    let result_field =
        ApplicationQueryResultFieldRef::<AccountQuery, AccountIdSlot, TestSchema, AccountId>::new(
            output_name,
            field,
        );
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
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
        .into_erased()
}

#[path = "query_member_closure_tests/managed_source_authorization.rs"]
mod managed_source_authorization;

use crate::application_schema::fragment;
struct QueryModel;
impl fragment::Fragment for QueryModel {
    const NAME: &'static str = "query_model";
    type Bindings = QueryBindings;
    type Members = (Account,);
}
struct QueryBindings;
impl fragment::BindingSet for QueryBindings {
    type Work<'a> = crate::application_schema::ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut crate::application_schema::ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<u64> for QueryBindings {
    type Binding = crate::application_schema::U64ApplicationValueBinding;
    const WRITABLE: bool = false;
    const EQUALITY_QUERYABLE: bool = true;
}
impl fragment::Entity for Account {
    const NAME: &'static str = "Account";
    type Aspects = (AccountFacts,);
    type Relations = ();
}
impl fragment::Aspect for AccountFacts {
    const NAME: &'static str = "AccountFacts";
    type Row = QueryRow0;
}
struct QueryRow0(u64);
impl fragment::Record<QueryRow0> for QueryBindings {
    fn record() -> impl fragment::Shape<QueryRow0, Self> {
        fragment::field("AccountId", |row: &QueryRow0| row.0).build(
            |value, _: &mut crate::application_schema::ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(QueryRow0(value))
            },
        )
    }
}
type AccountId = fragment::FieldSelection<QueryModel, Account, AccountFacts, u64>;

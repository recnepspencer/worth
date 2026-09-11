use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
        ApplicationQueryOrderingDirection, ApplicationQueryParameterSet, ApplicationQueryReference,
        ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
        ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
    },
    authoring::{
        AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName, OrderingSelector,
        RelationName, TraversalSelector,
    },
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_relation,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledApplicationQuery,
    WorthQueryInstalledPackageIndex, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

use crate::runtime::{
    QuerySchemaView, ScalarAspectType, SchemaFieldView, SchemaRelationView, WorthQueryReadFamily,
};

use super::super::super::support::public_bridge_runtime::workspace;

worth_query_application_schema! {
    pub schema EquivalenceSchema {
        owner: application_query_equivalence,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Account::reference())
                .entity(Activity::reference())
                .aspect(Account::reference(), AccountFacts::reference())
                .aspect(Account::reference(), Identity::reference())
                .aspect(Activity::reference(), ActivityFacts::reference())
                .field(Account::reference(), AccountId::reference())
                .field(Account::reference(), Id::reference())
                .field(Activity::reference(), ActivitySequence::reference())
                .relation(
                    AccountActivity::reference(),
                    Account::reference(),
                    Activity::reference(),
                )
                .application_query(query_definition())
        }
    }
}

worth_query_entity!(pub Account for EquivalenceSchema);
worth_query_entity!(pub Activity for EquivalenceSchema);
worth_query_aspect!(pub AccountFacts for EquivalenceSchema, Account; identity = AspectIdentity(0x91611051), revision = AspectContractRevision(1),);
worth_query_aspect!(pub Identity for EquivalenceSchema, Account; identity = AspectIdentity(0x91611052), revision = AspectContractRevision(1),);
worth_query_aspect!(pub ActivityFacts for EquivalenceSchema, Activity; identity = AspectIdentity(0x91611053), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountId for EquivalenceSchema, Account, AccountFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub Id for EquivalenceSchema, Account, Identity:
    String => worth_query_declaration::facade::application_schema::StringApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub ActivitySequence for EquivalenceSchema, Activity, ActivityFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding, read_only, equality
);
worth_query_relation!(
    pub AccountActivity in EquivalenceSchema, Account => Activity; integrity = same_context_unbounded_retain_dangling);

pub(super) struct ActivityParameters;
pub(super) struct ActivityResult;
struct AccountIdSlot;
struct IdentityIdSlot;
struct ActivitySlot;
struct ActivitySequenceSlot;

worth_query_declaration::worth_query_structured_value_binding!(
    pub(super) ActivityParametersBinding for ActivityParameters {
        identity: "worth.query.test.runtime.application-query-equivalence.activity-parameters.v1"
    }
);
worth_query_declaration::worth_query_structured_value_binding!(
    pub(super) ActivityResultBinding for ActivityResult {
        identity: "worth.query.test.runtime.application-query-equivalence.activity-result.v1"
    }
);
worth_query_declaration::worth_query_structured_value_binding!(
    ActivityNestedResultBinding for () {
        identity: "worth.query.test.runtime.application-query-equivalence.activity-nested-result.v1"
    }
);

worth_query_declaration::worth_query_portable_type!(ActivityResult => "worth.query.test.runtime.application-query-equivalence.activity-result.v1");
worth_query_declaration::worth_query_portable_type!(AccountIdSlot => "worth.query.test.runtime.application-query-equivalence.account-id-slot.v1");
worth_query_declaration::worth_query_portable_type!(IdentityIdSlot => "worth.query.test.runtime.application-query-equivalence.identity-id-slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitySlot => "worth.query.test.runtime.application-query-equivalence.activity-slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitySequenceSlot => "worth.query.test.runtime.application-query-equivalence.activity-sequence-slot.v1");

worth_query_application_query!(
    pub(super) ActivityQuery for EquivalenceSchema,
    identity "worth.query.test.runtime.application-query-equivalence.activity-query.v1",
    parameters ActivityParametersBinding,
    result ActivityResultBinding,
    scope Account => "Account",
    name "account_activity_equivalence"
);

pub(super) fn application_parameters() -> ApplicationQueryParameterSet<ActivityQuery> {
    ApplicationQueryParameterSet::new()
}

pub(super) fn installed_application_query() -> WorthQueryInstalledApplicationQuery<
    EquivalenceSchema,
    ActivityQuery,
    ActivityParameters,
    ActivityResult,
    Account,
> {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "application_query_equivalence",
        1,
        0,
    ))
    .application_schema(EquivalenceSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(EquivalenceSchema::declaration().unwrap())
    .unwrap()
    .application_query(query_reference())
    .unwrap()
}

pub(super) fn mature_family() -> WorthQueryReadFamily {
    workspace("application-query-equivalence")
        .define_read_family("application-query-equivalence", |read| {
            read.local_collection(
                "Account",
                mature_schema(),
                |query| {
                    query
                        .traverse(
                            TraversalSelector::bounded("AccountActivity", 1)
                                .expect("one direct relation should admit"),
                        )
                        .project(field("AccountFacts", "AccountId"))
                        .project(field("Identity", "Id"))
                        .project(field("ActivityFacts", "ActivitySequence"))
                        .order_by(
                            OrderingSelector::ascending("Identity", "Id")
                                .expect("typed identity ordering should admit"),
                        )
                },
                |shape| {
                    shape
                        .field(result_field("AccountFacts", "AccountId", "account_id"))
                        .field(result_field("Identity", "Id", "id"))
                        .field(result_field(
                            "ActivityFacts",
                            "ActivitySequence",
                            "activity_sequence",
                        ))
                },
            )
        })
        .unwrap()
}

pub(super) fn mature_flat_family() -> WorthQueryReadFamily {
    workspace("application-query-equivalence-flat")
        .define_read_family("application-query-equivalence-flat", |read| {
            read.local_collection(
                "Account",
                mature_schema(),
                |query| {
                    query
                        .project(field("AccountFacts", "AccountId"))
                        .project(field("Identity", "Id"))
                        .order_by(
                            OrderingSelector::ascending("Identity", "Id")
                                .expect("typed identity ordering should admit"),
                        )
                },
                |shape| {
                    shape
                        .field(result_field("AccountFacts", "AccountId", "account_id"))
                        .field(result_field("Identity", "Id", "id"))
                },
            )
        })
        .unwrap()
}

fn query_reference() -> ApplicationQueryReference<
    EquivalenceSchema,
    ActivityQuery,
    ActivityParameters,
    ActivityResult,
    Account,
> {
    ActivityQuery::reference()
}

fn query_definition(
) -> worth_query_declaration::facade::application_query::ApplicationQueryDefinition<
    EquivalenceSchema,
    ActivityQuery,
    ActivityParameters,
    ActivityResult,
    Account,
> {
    let account_id = ApplicationQueryResultFieldRef::<
        ActivityQuery,
        AccountIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("account_id", AccountId::reference());
    let activity_sequence = ApplicationQueryResultFieldRef::<
        ActivityQuery,
        ActivitySequenceSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("activity_sequence", ActivitySequence::reference());
    let identity_projection = ApplicationQueryResultFieldRef::<
        ActivityQuery,
        IdentityIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("id", Id::reference());
    let identity_ordering = ApplicationQueryResultFieldRef::<
        ActivityQuery,
        IdentityIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("id", Id::reference());
    let nested = ApplicationQueryResultShapeBuilder::<
        EquivalenceSchema,
        ActivityQuery,
        Activity,
        (),
        ActivityNestedResultBinding,
    >::new(Activity::reference())
    .field(activity_sequence);
    let shape = ApplicationQueryResultShapeBuilder::<
        EquivalenceSchema,
        ActivityQuery,
        Account,
        ActivityResult,
        ActivityResultBinding,
    >::new(Account::reference())
    .field(account_id)
    .field(identity_projection)
    .relation(
        ApplicationQueryResultRelationRef::<
            ActivityQuery,
            ActivitySlot,
            _,
            _,
            _,
            _,
            ForwardResultTraversal,
            ManyResults,
        >::forward_many("activity", AccountActivity::reference()),
        nested,
    )
    .build();
    ApplicationQueryDefinitionBuilder::declare(query_reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 3))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(
            identity_ordering,
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .unwrap()
}

fn mature_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "application-query-equivalence",
        [
            SchemaFieldView::new(
                AspectName::new("AccountFacts").unwrap(),
                FieldName::new("AccountId").unwrap(),
                ScalarAspectType::UInt64,
            ),
            SchemaFieldView::new(
                AspectName::new("Identity").unwrap(),
                FieldName::new("Id").unwrap(),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                AspectName::new("ActivityFacts").unwrap(),
                FieldName::new("ActivitySequence").unwrap(),
                ScalarAspectType::UInt64,
            ),
        ],
        [SchemaRelationView::new(
            RelationName::new("AccountActivity").unwrap(),
            1,
        )],
    )
}

fn field(aspect: &str, field: &str) -> AspectFieldSelector {
    AspectFieldSelector::new(aspect, field).unwrap()
}

fn result_field(aspect: &str, field: &str, delivered: &str) -> AuthoredResultShapeField {
    AuthoredResultShapeField::new(aspect, field, delivered).unwrap()
}

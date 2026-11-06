#[path = "support/records.rs"]
mod storage_model;
pub use storage_model::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryOrderingDirection,
        ApplicationQueryParameterSet, ApplicationQueryReference, ApplicationQueryResultFieldRef,
        ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder,
        ForwardResultTraversal, ManyResults,
    },
    authoring::{
        AspectFieldSelector, AspectName, AuthoredResultShapeField, FieldName, OrderingSelector,
        RelationName, TraversalSelector,
    },
};
use worth_query_declaration::worth_query_application_query;
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

use worth_query_declaration::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct EquivalenceSchema;
impl Fragment for EquivalenceSchema {
    const NAME: &'static str = "EquivalenceSchema";
    type Bindings = NoBindings;
    type Members = (QueryRecords, Contracts);
}
impl ApplicationSchema for EquivalenceSchema {
    const OWNER: &'static str = "application_query_equivalence";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
impl EquivalenceSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
pub struct Contracts;
impl Registration<EquivalenceSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<EquivalenceSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<EquivalenceSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.application_query(query_definition()))
    }
}

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
    .certification_query(query_reference())
    .unwrap()
}

pub(super) fn mature_family(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryReadFamily {
    workspace(request_execution, "application-query-equivalence")
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
                        .project(field(AccountId::at("AccountId").aspect(), "AccountId"))
                        .project(field(Id::at("Id").aspect(), "Id"))
                        .project(field(
                            ActivitySequence::at("ActivitySequence").aspect(),
                            "ActivitySequence",
                        ))
                        .order_by(
                            OrderingSelector::ascending(Id::at("Id").aspect(), "Id")
                                .expect("typed identity ordering should admit"),
                        )
                },
                |shape| {
                    shape
                        .field(result_field(
                            AccountId::at("AccountId").aspect(),
                            "AccountId",
                            "account_id",
                        ))
                        .field(result_field(Id::at("Id").aspect(), "Id", "id"))
                        .field(result_field(
                            ActivitySequence::at("ActivitySequence").aspect(),
                            "ActivitySequence",
                            "activity_sequence",
                        ))
                },
            )
        })
        .unwrap()
}

pub(super) fn mature_flat_family(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryReadFamily {
    workspace(request_execution, "application-query-equivalence-flat")
        .define_read_family("application-query-equivalence-flat", |read| {
            read.local_collection(
                "Account",
                mature_schema(),
                |query| {
                    query
                        .project(field(AccountId::at("AccountId").aspect(), "AccountId"))
                        .project(field(Id::at("Id").aspect(), "Id"))
                        .order_by(
                            OrderingSelector::ascending(Id::at("Id").aspect(), "Id")
                                .expect("typed identity ordering should admit"),
                        )
                },
                |shape| {
                    shape
                        .field(result_field(
                            AccountId::at("AccountId").aspect(),
                            "AccountId",
                            "account_id",
                        ))
                        .field(result_field(Id::at("Id").aspect(), "Id", "id"))
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
    let account_id = ApplicationQueryResultFieldRef::<ActivityQuery, AccountIdSlot, _, _>::new(
        "account_id",
        AccountId::at("AccountId"),
    );
    let activity_sequence =
        ApplicationQueryResultFieldRef::<ActivityQuery, ActivitySequenceSlot, _, _>::new(
            "activity_sequence",
            ActivitySequence::at("ActivitySequence"),
        );
    let identity_projection =
        ApplicationQueryResultFieldRef::<ActivityQuery, IdentityIdSlot, _, _>::new(
            "id",
            Id::at("Id"),
        );
    let identity_ordering =
        ApplicationQueryResultFieldRef::<ActivityQuery, IdentityIdSlot, _, _>::new(
            "id",
            Id::at("Id"),
        );
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
                AspectName::new(AccountId::at("AccountId").aspect()).unwrap(),
                FieldName::new("AccountId").unwrap(),
                ScalarAspectType::UInt64,
            ),
            SchemaFieldView::new(
                AspectName::new(Id::at("Id").aspect()).unwrap(),
                FieldName::new("Id").unwrap(),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                AspectName::new(ActivitySequence::at("ActivitySequence").aspect()).unwrap(),
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

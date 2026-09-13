use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOrderingDirection, ApplicationQueryParameterRef, ApplicationQueryReference,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityRef, U64ApplicationValueBinding,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_portable_type, worth_query_relation,
    worth_query_structured_value_binding,
};

use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDefinition, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

use super::WorthQueryApplicationQueryInstallationDenialKind;

mod authority_validation_tests;
mod canonical_basis_residue;
mod installed_identity;
mod selector_identity;
mod shape_identity;

worth_query_application_schema! {
    pub schema QueryTestSchema {
        owner: application_query_installation_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Account::reference())
                .entity(Activity::reference())
                .aspect(Account::reference(), AccountFacts::reference())
                .aspect(Activity::reference(), ActivityFacts::reference())
                .field(Account::reference(), AccountId::reference())
                .field(Activity::reference(), ActivitySequence::reference())
                .field(Activity::reference(), ActivityKind::reference())
                .field(Activity::reference(), ActivityStatus::reference())
                .relation(
                    AccountActivity::reference(),
                    Account::reference(),
                    Activity::reference(),
                )
                .application_query(definition(
                    ApplicationQueryOrderingDirection::Descending,
                    "sequence",
                ))
                .application_query(shape_identity::grouped_one_definition())
                .application_query(shape_identity::grouped_two_definition())
        }
    }
}

worth_query_entity!(pub Account for QueryTestSchema);
worth_query_entity!(pub Activity for QueryTestSchema);
worth_query_aspect!(pub AccountFacts for QueryTestSchema, Account; identity = AspectIdentity(0x9161104a), revision = AspectContractRevision(1),);
worth_query_aspect!(pub ActivityFacts for QueryTestSchema, Activity; identity = AspectIdentity(0x9161104b), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountId for QueryTestSchema, Account, AccountFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_only, equality
);
worth_query_field!(
    pub ActivitySequence for QueryTestSchema, Activity, ActivityFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_only, equality
);
worth_query_field!(
    pub ActivityKind for QueryTestSchema, Activity, ActivityFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_only, equality
);
worth_query_field!(
    pub ActivityStatus for QueryTestSchema, Activity, ActivityFacts:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_only, equality
);
worth_query_relation!(
    pub AccountActivity in QueryTestSchema, Account => Activity; integrity = same_context_unbounded_retain_dangling);

pub(super) struct ActivityQueryParameters;
pub(super) struct ActivityQueryResult;
struct AccountParameter;
struct AccountIdSlot;
struct ActivitySequenceSlot;
struct ActivityRelationSlot;

worth_query_portable_type!(ActivityQueryResult => "worth.query.test.installation.activity-result.v1");
worth_query_portable_type!(AccountIdSlot => "worth.query.test.installation.account-id-slot.v1");
worth_query_portable_type!(ActivitySequenceSlot => "worth.query.test.installation.sequence-slot.v1");
worth_query_portable_type!(ActivityRelationSlot => "worth.query.test.installation.relation-slot.v1");
worth_query_structured_value_binding!(
    pub(super) ActivityQueryParametersBinding for ActivityQueryParameters {
        identity: "ActivityQueryParameters"
    }
);
worth_query_structured_value_binding!(
    pub(super) ActivityQueryResultBinding for ActivityQueryResult {
        identity: "worth.query.test.installation.activity-result.v1"
    }
);
worth_query_structured_value_binding!(
    ActivityItemResultBinding for () { identity: "worth.rust.unit" }
);

worth_query_application_query!(
    pub(super) ActivityQuery for QueryTestSchema,
    identity "ActivityQuery",
    parameters ActivityQueryParametersBinding,
    result ActivityQueryResultBinding,
    scope Account => "Account",
    name "account_activity"
);
worth_query_application_query!(
    ActivityScopedQuery for QueryTestSchema,
    identity "ActivityScopedQuery",
    parameters ActivityQueryParametersBinding,
    result ActivityQueryResultBinding,
    scope Activity => "Activity",
    name "activity_scoped_account_activity"
);
fn query_reference() -> ApplicationQueryReference<
    QueryTestSchema,
    ActivityQuery,
    ActivityQueryParameters,
    ActivityQueryResult,
    Account,
> {
    ActivityQuery::reference()
}

fn account_parameter<Query>(
) -> ApplicationQueryParameterRef<Query, AccountParameter, U64ApplicationValueBinding> {
    ApplicationQueryParameterRef::from_query_identifier("account")
}

fn definition(
    direction: ApplicationQueryOrderingDirection,
    output_name: &'static str,
) -> ApplicationQueryDefinition<
    QueryTestSchema,
    ActivityQuery,
    ActivityQueryParameters,
    ActivityQueryResult,
    Account,
> {
    definition_for::<ActivityQuery, Account, ActivitySequenceSlot>(
        ActivityQuery::reference(),
        Account::reference(),
        direction,
        output_name,
    )
}

pub(super) fn definition_with_sequence_slot<
    SequenceSlot: worth_query_declaration::facade::portable_identity::WorthQueryPortableType,
>(
    direction: ApplicationQueryOrderingDirection,
    output_name: &'static str,
) -> ApplicationQueryDefinition<
    QueryTestSchema,
    ActivityQuery,
    ActivityQueryParameters,
    ActivityQueryResult,
    Account,
> {
    definition_for::<ActivityQuery, Account, SequenceSlot>(
        ActivityQuery::reference(),
        Account::reference(),
        direction,
        output_name,
    )
}

fn definition_for<
    Query,
    Scope,
    SequenceSlot: worth_query_declaration::facade::portable_identity::WorthQueryPortableType,
>(
    reference: ApplicationQueryReference<
        QueryTestSchema,
        Query,
        ActivityQueryParameters,
        ActivityQueryResult,
        Scope,
    >,
    scope: ApplicationEntityRef<QueryTestSchema, Scope>,
    direction: ApplicationQueryOrderingDirection,
    output_name: &'static str,
) -> ApplicationQueryDefinition<
    QueryTestSchema,
    Query,
    ActivityQueryParameters,
    ActivityQueryResult,
    Scope,
>
where
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        QueryTestSchema,
        ResultBinding = ActivityQueryResultBinding,
    >,
{
    let sequence =
        ApplicationQueryResultFieldRef::<Query, SequenceSlot, _, _, _, _, _, _, _, _>::new(
            output_name,
            ActivitySequence::reference(),
        );
    let nested = ApplicationQueryResultShapeBuilder::<
        QueryTestSchema,
        Query,
        Activity,
        (),
        ActivityItemResultBinding,
    >::new(Activity::reference())
    .field(sequence);
    let shape = ApplicationQueryResultShapeBuilder::<
        QueryTestSchema,
        Query,
        Account,
        ActivityQueryResult,
        ActivityQueryResultBinding,
    >::new(Account::reference())
    .field(ApplicationQueryResultFieldRef::<
        Query,
        AccountIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("account_id", AccountId::reference()))
    .relation(
        ApplicationQueryResultRelationRef::<
            Query,
            ActivityRelationSlot,
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
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Account::reference())
        .scope(scope)
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 2))
        .disclosure(ApplicationQueryDisclosureContract::installed_policy(
            "account-holder",
        ))
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
        .public()
        .parameter(account_parameter::<Query>())
        .where_equal(AccountId::reference(), account_parameter::<Query>())
        .order_by(sequence, direction)
        .build()
        .unwrap()
}

fn installed_schema() -> crate::facade::WorthQueryInstalledApplicationSchema<QueryTestSchema> {
    installed_index()
        .bind_application_schema(QueryTestSchema::declaration().unwrap())
        .unwrap()
}

fn installed_index() -> WorthQueryInstalledPackageIndex {
    installed_index_with(WorthQueryInstallationRuntimeIdentity::fresh(), false)
}

fn installed_index_with(
    runtime: WorthQueryInstallationRuntimeIdentity,
    package_drift: bool,
) -> WorthQueryInstalledPackageIndex {
    let mut package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "application_query_installation_test",
        1,
        0,
    ))
    .application_schema(QueryTestSchema::declaration().unwrap());
    if package_drift {
        package = package.definition(WorthQueryPortableDefinition::declaration_family(
            "extra",
            "query-package-drift",
        ));
    }
    let package = package.validate().unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        runtime,
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
}

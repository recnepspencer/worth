use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOrderingDirection, ApplicationQueryParameterRef, ApplicationQueryReference,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEntityRef, U64ApplicationValueBinding,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDefinition, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

use super::WorthQueryApplicationQueryInstallationDenialKind;

mod records;
use records::*;

mod authority_validation_tests;
mod canonical_basis_residue;
mod installed_identity;
mod selector_identity;
mod shape_identity;

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
) -> ApplicationQueryDefinitionBuilder<
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
) -> ApplicationQueryDefinitionBuilder<
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
) -> ApplicationQueryDefinitionBuilder<
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
    let sequence = ApplicationQueryResultFieldRef::<Query, SequenceSlot, _, _>::new(
        output_name,
        ActivitySequence::at("ActivitySequence"),
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
    .field(
        ApplicationQueryResultFieldRef::<Query, AccountIdSlot, _, _>::new(
            "account_id",
            AccountId::at("AccountId"),
        ),
    )
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
        .disclosure(ApplicationQueryDisclosureContract::installed_policy(
            "account-holder",
        ))
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_historical())
        .public()
        .parameter(account_parameter::<Query>())
        .where_equal(AccountId::at("AccountId"), account_parameter::<Query>())
        .order_by(sequence, direction)
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

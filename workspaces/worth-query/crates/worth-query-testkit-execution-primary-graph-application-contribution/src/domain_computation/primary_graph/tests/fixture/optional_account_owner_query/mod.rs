//! A public, exactly scoped account row with an optional incoming owner.

use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountOwner;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;
use worth_query_declaration::facade::application_query::*;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;
use worth_relational::facade::identity::EntityId;

pub struct OwnerSlot;
worth_query_declaration::worth_query_portable_type!(OwnerSlot => "worth.query.test.optional_account_owner.slot.v1");
pub struct OptionalAccountOwnerResult {
    pub account: EntityId,
    pub owner: Option<EntityId>,
}
worth_query_declaration::worth_query_portable_type!(OptionalAccountOwnerResult => "worth.query.test.optional_account_owner.result.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub ParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ResultBinding for OptionalAccountOwnerResult { identity: "worth.query.test.optional_account_owner.result.v1" });
worth_query_declaration::worth_query_structured_value_binding!(pub OwnerBinding for () { identity: "worth.rust.unit" });
worth_query_declaration::worth_query_application_query!(
    pub OptionalAccountOwnerQuery for IdentityExecutionSchema,
    identity "OptionalAccountOwnerQuery",
    parameters ParametersBinding,
    result ResultBinding,
    scope Account => "Account",
    name "optional_account_owner"
);

pub fn owner() -> ApplicationQueryResultRelationRef<
    OptionalAccountOwnerQuery,
    OwnerSlot,
    IdentityExecutionSchema,
    AccountOwner,
    Principal,
    Account,
    ReverseResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::reverse_optional("owner", AccountOwner::reference())
}

impl WorthQueryApplicationProjection<IdentityExecutionSchema, OptionalAccountOwnerQuery>
    for OptionalAccountOwnerResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            OptionalAccountOwnerQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            account: row.entity_id(),
            owner: row.optional(owner())?.map(|owner| owner.entity_id()),
        })
    }
}

pub fn definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    OptionalAccountOwnerQuery,
    AccountSummaryParameters,
    OptionalAccountOwnerResult,
    Account,
> {
    let owner_shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        OptionalAccountOwnerQuery,
        Principal,
        (),
        OwnerBinding,
    >::new(Principal::reference());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        OptionalAccountOwnerQuery,
        Account,
        OptionalAccountOwnerResult,
        ResultBinding,
    >::new(Account::reference())
    // The closed pair-plan door requires the first query's declared status result.
    .field(crate::domain_computation::primary_graph::tests::fixture::application_queries::status_result_field::<
        OptionalAccountOwnerQuery,
    >())
    .relation(owner(), owner_shape)
    .build();
    ApplicationQueryDefinitionBuilder::declare(OptionalAccountOwnerQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}

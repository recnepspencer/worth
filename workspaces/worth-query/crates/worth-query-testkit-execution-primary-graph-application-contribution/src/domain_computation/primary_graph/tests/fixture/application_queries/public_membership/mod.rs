use crate::domain_computation::primary_graph::tests::fixture::application_queries::*;
use worth_relational::facade::identity::EntityId;

pub struct PublicAccountMembershipResult {
    pub members: Vec<EntityId>,
    pub tag: String,
}
worth_query_declaration::worth_query_portable_type!(
    PublicAccountMembershipResult => "worth.query.test.execution.account_membership.result.v1"
);
worth_query_declaration::worth_query_structured_value_binding!(pub PublicAccountMembershipParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub PublicAccountMembershipResultBinding for PublicAccountMembershipResult { identity: "worth.query.test.execution.account_membership.result.v1" });
worth_query_declaration::worth_query_application_query!(
    pub PublicAccountMembershipQuery for IdentityExecutionSchema,
    identity "PublicAccountMembershipQuery",
    parameters PublicAccountMembershipParametersBinding,
    result PublicAccountMembershipResultBinding,
    scope Account => "Account",
    name "public_account_membership"
);

impl
    worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        PublicAccountMembershipQuery,
    > for PublicAccountMembershipResult
{
    fn project(
        row: &worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            PublicAccountMembershipQuery,
        >,
    ) -> Result<
        Self,
        worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial,
    > {
        let tag = row.field(tag_result_field())?;
        Ok(Self {
            members: if tag == "none" {
                Vec::new()
            } else {
                vec![row.entity_id()]
            },
            tag,
        })
    }
}

pub fn public_membership_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    PublicAccountMembershipQuery,
    AccountSummaryParameters,
    PublicAccountMembershipResult,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        PublicAccountMembershipQuery,
        Account,
        PublicAccountMembershipResult,
        PublicAccountMembershipResultBinding,
    >::new(Account::reference())
    .field(tag_result_field())
    .build();
    ApplicationQueryDefinitionBuilder::declare(PublicAccountMembershipQuery::reference())
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

pub fn tag_result_field() -> ApplicationQueryResultFieldRef<
    PublicAccountMembershipQuery,
    StatusResultSlot,
    IdentityExecutionSchema,
    AccountMembershipTag,
> {
    ApplicationQueryResultFieldRef::new("tag", AccountMembershipTag::at("AccountMembershipTag"))
}

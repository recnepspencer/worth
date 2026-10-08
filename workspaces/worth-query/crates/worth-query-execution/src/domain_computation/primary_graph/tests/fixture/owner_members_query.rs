//! Genuine owner-root census for managed image growth qualification.
use super::{
    Account, AccountLabel, AccountOwner, AccountPolicy, AccountSummaryParameters,
    IdentityExecutionSchema, Principal,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::facade::application_schema::{
    EqualityPredicate, NoApplicationUnit, ReadWrite,
};
use worth_relational::facade::identity::EntityId;

pub struct MembersSlot;
pub struct LabelSlot;
worth_query_declaration::worth_query_portable_type!(MembersSlot => "worth.query.test.owner_members.members.v1");
worth_query_declaration::worth_query_portable_type!(LabelSlot => "worth.query.test.owner_members.label.v1");
pub struct OwnerMembersResult {
    pub members: Vec<(EntityId, String)>,
}
worth_query_declaration::worth_query_portable_type!(OwnerMembersResult => "worth.query.test.owner_members.result.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub ParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ResultBinding for OwnerMembersResult { identity: "worth.query.test.owner_members.result.v1" });
worth_query_declaration::worth_query_structured_value_binding!(pub MemberBinding for () { identity: "worth.rust.unit" });
worth_query_declaration::worth_query_application_query!(
    pub OwnerMembersQuery for IdentityExecutionSchema,
    identity "OwnerMembersQuery", parameters ParametersBinding, result ResultBinding,
    scope Principal => "Principal", name "owner_members"
);
fn members() -> ApplicationQueryResultRelationRef<
    OwnerMembersQuery,
    MembersSlot,
    IdentityExecutionSchema,
    AccountOwner,
    Principal,
    Account,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("members", AccountOwner::reference())
}
fn label() -> ApplicationQueryResultFieldRef<
    OwnerMembersQuery,
    LabelSlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountLabel,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("label", AccountLabel::reference())
}
impl WorthQueryApplicationProjection<IdentityExecutionSchema, OwnerMembersQuery>
    for OwnerMembersResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, IdentityExecutionSchema, OwnerMembersQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            members: row
                .many(members())?
                .iter()
                .map(|member| Ok((member.entity_id(), member.field(label())?)))
                .collect::<Result<_, _>>()?,
        })
    }
}
pub(super) fn definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    OwnerMembersQuery,
    AccountSummaryParameters,
    OwnerMembersResult,
    Principal,
> {
    let child = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        OwnerMembersQuery,
        Account,
        (),
        MemberBinding,
    >::new(Account::reference())
    .field(label());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        OwnerMembersQuery,
        Principal,
        OwnerMembersResult,
        ResultBinding,
    >::new(Principal::reference())
    .relation(members(), child)
    .build();
    ApplicationQueryDefinitionBuilder::declare(OwnerMembersQuery::reference())
        .root(Principal::reference())
        .scope(Principal::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}

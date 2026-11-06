//! A membership row's members, each paired with the leading state of the row's tag.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::application_queries::public_membership::PublicAccountMembershipResult;
use worth_relational::facade::identity::EntityId;

pub fn members(row: &PublicAccountMembershipResult) -> Vec<(EntityId, String)> {
    row.members
        .iter()
        .map(|root| (*root, row.tag.split('-').next().unwrap().to_string()))
        .collect::<Vec<_>>()
}

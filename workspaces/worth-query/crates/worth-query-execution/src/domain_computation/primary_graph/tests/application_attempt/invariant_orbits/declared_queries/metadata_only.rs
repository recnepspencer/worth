//! The declared orbit slot is itself a result; member DTO fields are optional.
use super::*;
use worth_query_declaration::{facade::application_query::*, *};
pub struct Member;
worth_query_portable_type!(Member => "worth.query.declared-orbit.member.v1");
worth_query_structured_value_binding!(pub MemberBinding for Member {identity:"worth.query.declared-orbit.member.v1"});
worth_query_application_query!(pub Names for Schema: OrbitModel, identity "DeclaredOrbitNames", parameters ParametersBinding, result MemberBinding, scope Node => "Node", name "declared_orbit_names");
impl<S: OrbitModel> WorthQueryApplicationProjection<S, Names> for Member {
    fn project(
        _: &WorthQueryApplicationProjectionRow<'_, S, Names>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self)
    }
}
pub(super) fn definition<S: OrbitModel>(
) -> ApplicationQueryDefinition<S, Names, Parameters, Member, Node> {
    let shape = ApplicationQueryResultShapeBuilder::<S, Names, Node, Member, MemberBinding>::new(
        Node::reference(),
    )
    .build();
    ApplicationQueryDefinitionBuilder::declare(Names::reference())
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .orbit(whole::<S, Names>())
        .build()
        .unwrap()
}
#[test]
fn complete_orbit_metadata_does_not_require_an_arbitrary_member_field() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = world();
    let result = read::run(&world, Names::reference(), 0);
    let orbit = result.orbit(whole::<OrbitSchema<true>, Names>()).unwrap();
    assert_eq!(orbit.len(), 3);
    assert_eq!(orbit.members().len(), 3);
}

//! The declared orbit slot is itself a result; member DTO fields are optional.

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitModel;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::Parameters;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::whole;

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::*;
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::*;
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
pub fn definition<S: OrbitModel>(
) -> ApplicationQueryDefinitionBuilder<S, Names, Parameters, Member, Node> {
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
}

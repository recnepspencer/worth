//! The declared orbit slot is itself a result; member DTO fields are optional.

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::metadata_only::Names;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::whole;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::world;

use worth_query_declaration::facade::application_query::*;
#[test]
fn complete_orbit_metadata_does_not_require_an_arbitrary_member_field() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = world();
    let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Names::reference(), 0);
    let orbit = result.orbit(whole::<OrbitSchema<true>, Names>()).unwrap();
    assert_eq!(orbit.len(), 3);
    assert_eq!(orbit.members().len(), 3);
}

//! Installed complete orbit answers, with the existing bootstrap and admission path.

pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::world;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::KeySlot;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::Next;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::NodeRow;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitModel;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::OtherSlot;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::Parameters;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::builder;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::definition;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::native_oracle;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::orbit;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::whole;

pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
pub use worth_execution::ExecutionRequest;
pub use worth_query_execution::facade::application_invariants::WorthQueryAggregateIdentity;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotResult;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAccessContext;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
pub use worth_query_execution::facade::domain_computation::WorthQueryObservedSource;
pub use worth_query_execution::facade::domain_computation::WorthQueryPrincipalResolutionMode;
pub use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::*;
#[test]
fn complete_declared_orbit_keeps_anchor_order_identity_and_original_result_custody() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = world();
    let result = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Orbit::reference(), 1);
    let selected = result.orbit(orbit::<OrbitSchema<true>>()).unwrap();
    let keys: Vec<_> = selected.members().iter().map(|row| row.key).collect();
    let (expected, native_identity) = native_oracle(&world);
    assert_eq!(keys, expected);
    assert_eq!(selected.identity(), &native_identity);
    assert_eq!(selected.len(), 3);
    let identity = selected.identity().clone();
    let other: worth_query_declaration::facade::application_query::ApplicationQueryOrbitRef<
        OrbitSchema<true>,
        Orbit,
        OtherSlot,
        Next,
        Node,
    > = worth_query_declaration::facade::application_query::ApplicationQueryOrbitRef::new(
        "orbit",
        Next::reference(),
    );
    assert!(result.orbit(other).is_none());
    let admitted = result.into_admitted_disclosed();
    assert_eq!(
        admitted
            .orbit(orbit::<OrbitSchema<true>>())
            .unwrap()
            .identity(),
        &identity
    );
    let demand = admitted.into_output_demand_source();
    assert_eq!(
        demand
            .orbit(orbit::<OrbitSchema<true>>())
            .unwrap()
            .members()
            .iter()
            .map(|row| row.key)
            .collect::<Vec<_>>(),
        keys
    );
    let sibling = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Orbit::reference(), 2);
    assert_eq!(
        sibling
            .orbit(orbit::<OrbitSchema<true>>())
            .unwrap()
            .identity(),
        &identity
    );
}

#[path = "components/mod.rs"]
pub mod components;
#[path = "currentness/mod.rs"]
pub mod currentness;
#[path = "disclosure/mod.rs"]
pub mod disclosure;
#[path = "metadata_only/mod.rs"]
pub mod metadata_only;
#[path = "planning/mod.rs"]
pub mod planning;
#[path = "portable/mod.rs"]
pub mod portable;

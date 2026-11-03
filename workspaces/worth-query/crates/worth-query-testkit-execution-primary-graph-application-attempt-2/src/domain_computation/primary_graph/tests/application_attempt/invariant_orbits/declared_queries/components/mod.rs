//! Installed component contracts exercised through genuine Query and Native owners.

use worth_execution::ExecutionRequest;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::Graph;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::NodeRow;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitModel;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::OrbitSchema;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::Parameters;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
use worth_query_execution::facade::application_invariants::WorthQueryAggregateIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryObservedSource;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::component_selection;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::components;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::denial;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::key;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::metadata_only;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::declarations::orbit;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::root;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::world;

use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Shells;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::components::binding::Visible;
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::*;
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::facade::application_schema::*;
use worth_query_declaration::*;
pub struct ComponentSlot;
worth_query_portable_type!(ComponentSlot=>"worth.query.components.query-slot.v1");
worth_query_application_query!(pub Components for Schema:OrbitModel,identity "DeclaredComponents",parameters ParametersBinding,result crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::metadata_only::MemberBinding,scope Node=>"Node",name "declared_components");
macro_rules! query {
    ($q:ident,$name:literal) => {
        worth_query_application_query!(pub $q for Schema:OrbitModel,identity stringify!($q),parameters ParametersBinding,result RowsBinding,scope Node=>"Node",name $name);
    };
}
query!(Census, "component_census");
query!(Of, "component_of");
query!(Partial, "component_partial");
query!(Empty, "component_empty");
query!(Allowed, "component_allowed");
query!(Omitted, "component_omitted");
query!(Missing, "component_missing");
query!(GuardDenied, "component_guarddenied");
query!(GuardAllowed, "component_guardallowed");
query!(AggregateDenied, "component_aggregatedenied");
query!(OrderingDenied, "component_orderingdenied");
impl<S: OrbitModel> WorthQueryApplicationProjection<S, Components> for crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::metadata_only::Member {
    fn project(
        _: &WorthQueryApplicationProjectionRow<'_, S, Components>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self)
    }
}
pub fn selection<S: OrbitModel, Q: ApplicationQueryMarkerIdentity<S, Scope = Node>>(
) -> ApplicationQueryComponentRef<S, Q, ComponentSlot, Shells, Node> {
    ApplicationQueryComponentRef::census("shells", Shells::reference())
}
pub fn of<S: OrbitModel>() -> ApplicationQueryComponentRef<S, Of, ComponentSlot, Shells, Node> {
    ApplicationQueryComponentRef::of_scope_member("shells", Shells::reference())
}
pub fn declare<const O: bool>(
    schema: ApplicationSchemaDeclarationBuilder<OrbitSchema<O>>,
) -> ApplicationSchemaDeclarationBuilder<OrbitSchema<O>> {
    crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::components::declarations::declare(schema)
}
pub fn interleaved_world() -> World {
    world::install_orbit_with_component_visibility(
        Graph {
            nodes: 5,
            edges: vec![(0, 2), (2, 4), (4, 0), (1, 3), (3, 1)],
            active: false,
            blocked: vec![],
        },
        crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::probe::Probe::new(|_, _, _| {}),
        vec![(0, 1)],
    )
}

#[path = "declarations/mod.rs"]
pub mod declarations;

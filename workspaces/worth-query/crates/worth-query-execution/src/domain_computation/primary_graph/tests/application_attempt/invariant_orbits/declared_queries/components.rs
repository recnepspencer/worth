//! Installed descriptive component queries refuse before ordinary member-row execution.
use super::super::decision::components::binding::Shells;
use super::*;
use worth_query_declaration::{facade::application_query::*, *};
use worth_query_installation::facade::WorthQueryReadGraphPlanningContract;
pub struct ComponentSlot;
worth_query_portable_type!(ComponentSlot=>"worth.query.components.query-slot.v1");
worth_query_application_query!(pub Components for Schema:OrbitModel,identity "DeclaredComponents",parameters ParametersBinding,result metadata_only::MemberBinding,scope Node=>"Node",name "declared_components");
impl<S: OrbitModel> WorthQueryApplicationProjection<S, Components> for metadata_only::Member {
    fn project(
        _: &WorthQueryApplicationProjectionRow<'_, S, Components>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self)
    }
}
fn selection<S: OrbitModel>(
) -> ApplicationQueryComponentRef<S, Components, ComponentSlot, Shells, Node> {
    ApplicationQueryComponentRef::census("shells", Shells::reference())
}
pub(super) fn definition<S: OrbitModel>(
) -> ApplicationQueryDefinition<S, Components, Parameters, metadata_only::Member, Node> {
    ApplicationQueryDefinitionBuilder::declare(Components::reference())
        .root(Node::reference())
        .scope(Node::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                S,
                Components,
                Node,
                metadata_only::Member,
                metadata_only::MemberBinding,
            >::new(Node::reference())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Node::reference()).forward_closure(Next::reference()),
        )
        .component_selection(selection::<S>())
        .build()
        .unwrap()
}
#[test]
fn installed_component_query_retains_real_planning_meaning_and_refuses_ordinary_rows() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
    let world = world();
    let installed = read::installed(&world, Components::reference());
    let graph = installed.read_graph();
    assert_eq!(
        graph.component_selection(),
        Some(selection::<OrbitSchema<true>>().meaning())
    );
    assert_eq!(
        WorthQueryReadGraphPlanningContract::component_selection(graph),
        graph.component_selection()
    );
    assert!(graph.orbit().is_none());
    assert!(graph.projections().is_empty());
    assert!(graph.relations().is_empty());
    assert!(graph.canonical_planning_basis().basis().payload().entries().iter().any(|entry| {
        matches!(entry.locus(), worth_foundational::facade::CanonicalBasisLocus::Named(worth_foundational::facade::InternedString::Raw(name)) if name == "components.set")
    }));
    let host = crate::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let request = crate::domain_computation::primary_graph::tests::fixture::live_scope();
    let denied = read::execute(&world, &execution, &request, Components::reference(), 0);
    match denied {
        Err(read::Stop::Admission(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationQueryAdmissionDenialKind::ExecutionShapeUnsupported
        ),
        Err(other) => panic!("component query reached the wrong refusal boundary: {other:?}"),
        Ok(_) => panic!("unsupported component query executed as ordinary member rows"),
    }
    let control = read::execute(&world, &execution, &request, Orbit::reference(), 0).unwrap();
    assert_eq!(
        control
            .orbit(whole::<OrbitSchema<true>, Orbit>())
            .unwrap()
            .len(),
        3
    );
}

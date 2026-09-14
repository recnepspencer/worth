#[path = "application_query_equivalence/one_axis_parity.rs"]
mod one_axis_parity;
#[path = "application_query_equivalence/support.rs"]
mod support;

use worth_foundational::facade::{
    compare_canonical_basis, prepare_canonical_comparison, CanonicalBasisLocus,
    CanonicalComparisonOutcome, CanonicalEquivalenceBasis, CanonicalMismatchKind,
};
use worth_query_admission::facade::{
    application_query::{admit_application_query_parameters, WorthQueryApplicationQueryLane},
    graph_read_access::{
        WorthQueryGraphReadAccessRequirementRow, WorthQueryGraphReadOrderingPosture,
        WorthQueryGraphReadResultPressure, WorthQueryGraphReadTraversalOperator,
    },
};
use worth_query_admission::integration::derive_graph_read_access_requirements_for_contract;
use worth_query_installation::facade::{
    WorthQueryReadGraphOrderingMechanism, WorthQueryReadGraphPlanningContract,
};

use crate::runtime::graph_read_access::explain_graph_read_access_requirements_for_family;

#[test]
fn equivalent_graphs_share_semantics_and_retain_their_traversal_mechanisms() {
    let mature = support::mature_family();
    let application = support::installed_application_query();
    let parameters =
        admit_application_query_parameters(&application, support::application_parameters())
            .unwrap();

    assert_ne!(
        mature.read_graph().digest(),
        application.read_graph().digest().render_hex(),
        "the courtroom must compare independently constructed graph sources"
    );
    assert_ne!(
        mature.read_graph().schema_basis_digest(),
        application.read_graph().schema_basis_digest(),
        "schema authority remains source-owned and may not be erased to force equality"
    );
    assert_eq!(
        planning_semantics(mature.read_graph()),
        planning_semantics(application.read_graph())
    );
    let comparison = prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        mature
            .read_graph()
            .canonical_planning_basis()
            .basis()
            .clone(),
        application
            .read_graph()
            .canonical_planning_basis()
            .basis()
            .clone(),
    )
    .into_result()
    .expect("both graph sources use the same supported planning basis");
    let comparison_outcome = compare_canonical_basis(&comparison);
    match comparison_outcome {
        CanonicalComparisonOutcome::Mismatched(mismatch) => {
            assert_eq!(mismatch.kind(), CanonicalMismatchKind::ValueMismatch);
            assert_eq!(
                mismatch.left_locus(),
                Some(&CanonicalBasisLocus::Named("schema-basis".into()))
            );
            assert_eq!(mismatch.left_locus(), mismatch.right_locus());
        }
        outcome => panic!(
            "source-owned schema authority must remain the only exact canonical mismatch: {outcome:#?}"
        ),
    }
    assert_eq!(
        mature.read_graph().ordering(0).unwrap().mechanism,
        WorthQueryReadGraphOrderingMechanism::ProviderOrdered
    );
    assert_eq!(
        WorthQueryReadGraphPlanningContract::ordering(application.read_graph(), 0)
            .unwrap()
            .mechanism,
        WorthQueryReadGraphOrderingMechanism::BoundedProjectedCollection
    );

    let mature_requirements = explain_graph_read_access_requirements_for_family(&mature).unwrap();
    let application_requirements = derive_graph_read_access_requirements_for_contract(
        application.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
        application.canonical_work_policy().admission_planning(),
    )
    .expect("the application courtroom requirements fit the installed canonical budget");
    let mature_ordering = requirement_row(mature_requirements.rows(), "ordering_support");
    let application_ordering = requirement_row(application_requirements.rows(), "ordering_support");
    assert_eq!(
        mature_ordering.ordering_posture(),
        Some(&WorthQueryGraphReadOrderingPosture::ProviderOrdered)
    );
    assert_eq!(
        application_ordering.ordering_posture(),
        Some(&WorthQueryGraphReadOrderingPosture::BoundedProjectedCollection)
    );
    assert_eq!(
        shared_requirement_semantics(mature_requirements.rows()),
        shared_requirement_semantics(application_requirements.rows())
    );
    assert_eq!(
        requirement_row(mature_requirements.rows(), "directional_adjacency").traversal_operator(),
        Some(&WorthQueryGraphReadTraversalOperator::DeclarationTraversal)
    );
    assert_eq!(
        requirement_row(application_requirements.rows(), "directional_adjacency")
            .traversal_operator(),
        Some(&WorthQueryGraphReadTraversalOperator::DirectEdge)
    );
    for kind in ["traversal_workset", "visited_set"] {
        assert!(mature_requirements
            .rows()
            .iter()
            .any(|row| row.kind().as_str() == kind));
        assert!(application_requirements
            .rows()
            .iter()
            .all(|row| row.kind().as_str() != kind));
    }
    assert_eq!(
        requirement_row(application_requirements.rows(), "result_buffer").result_pressure(),
        Some(&WorthQueryGraphReadResultPressure::CollectionWide)
    );
    assert!(application_requirements
        .rows()
        .iter()
        .any(|row| row.kind().as_str() == "proof_support"));
    assert!(mature_requirements
        .rows()
        .iter()
        .all(|row| row.maximum_cardinality().is_none()));
    assert!(application_requirements
        .rows()
        .iter()
        .all(|row| row.maximum_cardinality() == Some(32)));
}

#[test]
fn mature_fanout_alone_adds_proof_support_and_wide_result_pressure() {
    let flat =
        explain_graph_read_access_requirements_for_family(&support::mature_flat_family()).unwrap();
    let fanout = explain_graph_read_access_requirements_for_family(&support::mature_family())
        .expect("fanout family should explain");

    assert!(!flat
        .rows()
        .iter()
        .any(|row| row.kind().as_str() == "proof_support"));
    assert_eq!(
        requirement_row(flat.rows(), "result_buffer").result_pressure(),
        Some(&WorthQueryGraphReadResultPressure::CollectionNarrow)
    );
    assert!(fanout
        .rows()
        .iter()
        .any(|row| row.kind().as_str() == "proof_support"));
    assert_eq!(
        requirement_row(fanout.rows(), "result_buffer").result_pressure(),
        Some(&WorthQueryGraphReadResultPressure::CollectionWide)
    );
}

#[derive(Debug, Eq, PartialEq)]
struct PlanningSemantics {
    root: String,
    cardinality: String,
    projections: Vec<String>,
    relations: Vec<String>,
    predicates: Vec<String>,
    orderings: Vec<String>,
    maximum_traversal_depth: usize,
}

fn planning_semantics(graph: &impl WorthQueryReadGraphPlanningContract) -> PlanningSemantics {
    let mut semantics = PlanningSemantics {
        root: graph.root_entity().to_string(),
        cardinality: format!("{:?}", graph.cardinality()),
        projections: (0..graph.projection_count())
            .map(|index| {
                let projection = graph.projection(index).unwrap();
                format!(
                    "{}:{}:{}",
                    projection.aspect.as_str(),
                    projection.field.as_str(),
                    projection.output_name
                )
            })
            .collect(),
        relations: (0..graph.relation_count())
            .map(|index| {
                let relation = graph.relation(index).unwrap();
                format!(
                    "{}:{:?}:{:?}:{}",
                    relation.relation, relation.direction, relation.cardinality, relation.depth
                )
            })
            .collect(),
        predicates: (0..graph.predicate_count())
            .map(|index| {
                let predicate = graph.predicate(index).unwrap();
                format!(
                    "{}:{}:{}:{}",
                    predicate.aspect.as_str(),
                    predicate.field.as_str(),
                    predicate.parameter,
                    predicate.scalar_family.canonical_name()
                )
            })
            .collect(),
        orderings: (0..graph.ordering_count())
            .map(|index| {
                let ordering = graph.ordering(index).unwrap();
                format!(
                    "{}:{}:{}:{:?}:{}",
                    ordering.collection_path,
                    ordering.aspect.as_str(),
                    ordering.field.as_str(),
                    ordering.direction,
                    ordering.scalar_family.canonical_name(),
                )
            })
            .collect(),
        maximum_traversal_depth: graph.maximum_traversal_depth(),
    };
    semantics.projections.sort();
    semantics.relations.sort();
    semantics.predicates.sort();
    semantics
}

fn requirement_semantics(rows: &[WorthQueryGraphReadAccessRequirementRow]) -> Vec<String> {
    let mut rows = rows
        .iter()
        .map(|row| {
            let predicates = row
                .predicate_field_authorities()
                .iter()
                .map(|field| {
                    format!(
                        "{}:{}:{}",
                        field.native_aspect_key().as_str(),
                        field.native_field_key().as_str(),
                        field.field_kind()
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let orderings = row
                .ordering_field_authorities()
                .iter()
                .map(|field| {
                    format!(
                        "{}:{}:{}:{}:{}",
                        field.collection_path(),
                        field.native_aspect_key().as_str(),
                        field.native_field_key().as_str(),
                        field.direction(),
                        field.field_kind()
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{:?}:{}:{}:{:?}:{:?}:{:?}",
                row.kind().as_str(),
                row.rebuild_basis(),
                row.relation_name(),
                row.relation_direction(),
                row.relation_depth(),
                row.fanout_posture(),
                row.predicate_family(),
                row.traversal_operator(),
                row.lifecycle_class(),
                predicates,
                orderings,
                row.result_pressure(),
                row.invalidation_basis(),
                row.complexity_contract(),
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows
}

fn shared_requirement_semantics(rows: &[WorthQueryGraphReadAccessRequirementRow]) -> Vec<String> {
    requirement_semantics(
        &rows
            .iter()
            .filter(|row| {
                !matches!(
                    row.kind().as_str(),
                    "directional_adjacency" | "traversal_workset" | "visited_set"
                )
            })
            .cloned()
            .collect::<Vec<_>>(),
    )
}

fn requirement_row<'a>(
    rows: &'a [WorthQueryGraphReadAccessRequirementRow],
    kind: &str,
) -> &'a WorthQueryGraphReadAccessRequirementRow {
    rows.iter()
        .find(|row| row.kind().as_str() == kind)
        .expect("required planning row should exist")
}

use worth_foundational::facade::{
    CanonicalBasisEntry, CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget,
};

use super::{digest, prepare_artifact, text, unsigned, WorthQueryApplicationCanonicalArtifact};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryCardinality, ApplicationQueryOrderingDirection,
};

use crate::application_query::{
    WorthQueryReadGraphPlanningContract, WorthQueryReadGraphRelationDirection,
};

pub(in crate::application_query) fn prepare_planning_basis(
    graph: &impl WorthQueryReadGraphPlanningContract,
    budget: CanonicalDigestWorkBudget,
) -> Result<WorthQueryApplicationCanonicalArtifact, CanonicalDigestDerivationDenial> {
    let mut entries = vec![
        digest("schema-basis", graph.schema_basis_digest()),
        text("root-entity", graph.root_entity()),
        text("cardinality", cardinality_name(graph.cardinality())),
    ];
    append_projections(&mut entries, graph);
    append_relations(&mut entries, graph);
    entries.push(text(
        "root-union-dedup-required",
        if graph.root_union_dedup_required() {
            "true"
        } else {
            "false"
        },
    ));
    append_predicates(&mut entries, graph);
    append_guards(&mut entries, graph);
    append_ordering(&mut entries, graph);
    entries.push(unsigned(
        "maximum-traversal-depth",
        graph.maximum_traversal_depth(),
    ));
    prepare_artifact("normalized-read-planning", entries, budget)
}

fn append_projections(
    entries: &mut Vec<CanonicalBasisEntry>,
    graph: &impl WorthQueryReadGraphPlanningContract,
) {
    let mut projections = (0..graph.projection_count())
        .map(|index| {
            graph
                .projection(index)
                .expect("planning projection count must be exact")
        })
        .collect::<Vec<_>>();
    projections.sort_by_key(|projection| {
        (
            projection.aspect.as_str(),
            projection.field.as_str(),
            projection.output_name,
        )
    });
    for (index, projection) in projections.into_iter().enumerate() {
        let path = format!("projection[{index}]");
        entries.extend([
            text(format!("{path}.aspect"), projection.aspect.as_str()),
            text(format!("{path}.field"), projection.field.as_str()),
            text(format!("{path}.output"), projection.output_name),
        ]);
    }
}

fn append_relations(
    entries: &mut Vec<CanonicalBasisEntry>,
    graph: &impl WorthQueryReadGraphPlanningContract,
) {
    let mut relations = (0..graph.relation_count())
        .map(|index| {
            graph
                .relation(index)
                .expect("planning relation count must be exact")
        })
        .collect::<Vec<_>>();
    relations.sort_by_key(|relation| {
        (
            relation.relation,
            match relation.direction {
                WorthQueryReadGraphRelationDirection::Forward => "forward",
                WorthQueryReadGraphRelationDirection::Reverse => "reverse",
            },
            cardinality_name(relation.cardinality),
            relation.depth,
            relation.complete_closure,
            relation.aggregate_selection,
        )
    });
    for (index, relation) in relations.into_iter().enumerate() {
        let path = format!("relation[{index}]");
        if relation.aggregate_selection {
            entries.push(text(format!("{path}.aggregate-selection"), "whole-orbit"));
        }
        if relation.complete_closure {
            entries.push(text(format!("{path}.traversal"), "closure"));
        }
        entries.extend([
            text(format!("{path}.name"), relation.relation),
            text(
                format!("{path}.direction"),
                match relation.direction {
                    WorthQueryReadGraphRelationDirection::Forward => "forward",
                    WorthQueryReadGraphRelationDirection::Reverse => "reverse",
                },
            ),
            text(
                format!("{path}.cardinality"),
                cardinality_name(relation.cardinality),
            ),
            unsigned(format!("{path}.depth"), relation.depth),
        ]);
    }
}

fn append_predicates(
    entries: &mut Vec<CanonicalBasisEntry>,
    graph: &impl WorthQueryReadGraphPlanningContract,
) {
    let mut predicates = (0..graph.predicate_count())
        .map(|index| {
            graph
                .predicate(index)
                .expect("planning predicate count must be exact")
        })
        .collect::<Vec<_>>();
    predicates.sort_by_key(|predicate| {
        (
            predicate.aspect.as_str(),
            predicate.field.as_str(),
            predicate.parameter,
            predicate.scalar_family.canonical_name(),
        )
    });
    for (index, predicate) in predicates.into_iter().enumerate() {
        let path = format!("predicate[{index}]");
        entries.extend([
            text(format!("{path}.aspect"), predicate.aspect.as_str()),
            text(format!("{path}.field"), predicate.field.as_str()),
            text(format!("{path}.parameter"), predicate.parameter),
            text(
                format!("{path}.scalar"),
                predicate.scalar_family.canonical_name(),
            ),
        ]);
    }
}

fn append_guards(
    entries: &mut Vec<CanonicalBasisEntry>,
    graph: &impl WorthQueryReadGraphPlanningContract,
) {
    let mut guards = (0..graph.guard_count())
        .map(|index| {
            graph
                .guard(index)
                .expect("planning guard count must be exact")
        })
        .collect::<Vec<_>>();
    guards.sort_by_key(|guard| {
        (
            guard.after_step,
            guard.entity,
            guard.aspect.as_str(),
            guard.field.as_str(),
            guard.scalar_family.canonical_name(),
            guard.value_type,
        )
    });
    for (index, guard) in guards.into_iter().enumerate() {
        let path = format!("guard[{index}]");
        entries.extend([
            unsigned(format!("{path}.after-step"), guard.after_step),
            text(format!("{path}.entity"), guard.entity),
            text(format!("{path}.aspect"), guard.aspect.as_str()),
            text(format!("{path}.field"), guard.field.as_str()),
            text(
                format!("{path}.scalar"),
                guard.scalar_family.canonical_name(),
            ),
            text(format!("{path}.value-type"), guard.value_type),
            text(
                format!("{path}.expected"),
                worth_foundational::facade::prepare_aspect_value_identity_basis(guard.expected)
                    .as_str(),
            ),
        ]);
    }
}

fn append_ordering(
    entries: &mut Vec<CanonicalBasisEntry>,
    graph: &impl WorthQueryReadGraphPlanningContract,
) {
    for index in 0..graph.ordering_count() {
        let ordering = graph
            .ordering(index)
            .expect("planning ordering count must be exact");
        let path = format!("ordering[{index}]");
        entries.extend([
            text(format!("{path}.collection-path"), ordering.collection_path),
            text(format!("{path}.aspect"), ordering.aspect.as_str()),
            text(format!("{path}.field"), ordering.field.as_str()),
            text(
                format!("{path}.direction"),
                ordering_direction_name(ordering.direction),
            ),
            text(
                format!("{path}.scalar"),
                ordering.scalar_family.canonical_name(),
            ),
        ]);
    }
}

const fn cardinality_name(cardinality: ApplicationQueryCardinality) -> &'static str {
    match cardinality {
        ApplicationQueryCardinality::OptionalOne => "optional-one",
        ApplicationQueryCardinality::ExactlyOne => "exactly-one",
        ApplicationQueryCardinality::Many => "many",
    }
}

const fn ordering_direction_name(direction: ApplicationQueryOrderingDirection) -> &'static str {
    match direction {
        ApplicationQueryOrderingDirection::Ascending => "ascending",
        ApplicationQueryOrderingDirection::Descending => "descending",
    }
}

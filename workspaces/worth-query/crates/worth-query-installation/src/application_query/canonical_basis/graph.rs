use worth_foundational::facade::{
    CanonicalBasisEntry, CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget,
};
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;

use super::{digest, prepare_artifact, text, unsigned, WorthQueryApplicationCanonicalArtifact};
use crate::application_query::graph_access_contract::WorthQueryInstalledGraphReadMeaning;
use crate::application_query::{
    WorthQueryInstalledGraphOrdering, WorthQueryInstalledGraphPredicate,
    WorthQueryInstalledGraphProjection, WorthQueryInstalledGraphRelation,
};

pub(in crate::application_query) fn prepare_graph_basis(
    meaning: &WorthQueryInstalledGraphReadMeaning,
    budget: CanonicalDigestWorkBudget,
) -> Result<WorthQueryApplicationCanonicalArtifact, CanonicalDigestDerivationDenial> {
    let mut entries = vec![
        digest("schema-basis", &meaning.schema_basis_digest),
        text("root-entity", &meaning.root_entity),
        text("cardinality", cardinality_name(meaning.cardinality)),
    ];
    append_projections(&mut entries, &meaning.projections);
    append_root_paths(&mut entries, &meaning.root_paths);
    append_relations(&mut entries, &meaning.relations);
    append_predicates(&mut entries, &meaning.predicates);
    append_ordering(&mut entries, &meaning.ordering);
    entries.extend([
        unsigned("maximum-traversal-depth", meaning.maximum_traversal_depth),
        unsigned("maximum-result-count", meaning.maximum_result_count),
    ]);
    prepare_artifact("installed-read-graph", entries, budget)
}

fn append_root_paths(
    entries: &mut Vec<CanonicalBasisEntry>,
    paths: &[crate::application_query::WorthQueryInstalledRootPath],
) {
    for (path_index, root_path) in paths.iter().enumerate() {
        for (step_index, step) in root_path.steps().iter().enumerate() {
            let path = format!("root-path[{path_index}].step[{step_index}]");
            entries.extend([
                text(format!("{path}.relation"), step.relation()),
                text(format!("{path}.from"), step.from()),
                text(format!("{path}.to"), step.to()),
                text(
                    format!("{path}.direction"),
                    match step.direction() {
                        worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection::Forward => "forward",
                        worth_query_declaration::facade::application_query::ApplicationQueryRootPathDirection::Reverse => "reverse",
                    },
                ),
                unsigned(format!("{path}.depth"), step.depth()),
            ]);
        }
        for (guard_index, guard) in root_path.guards().iter().enumerate() {
            let guard_path = format!("root-path[{path_index}].guard[{guard_index}]");
            entries.extend([
                unsigned(format!("{guard_path}.after-step"), guard.after_step()),
                text(format!("{guard_path}.entity"), guard.entity()),
                text(format!("{guard_path}.aspect"), guard.aspect().as_str()),
                text(format!("{guard_path}.field"), guard.field().as_str()),
                text(
                    format!("{guard_path}.scalar"),
                    guard.scalar_family().canonical_name(),
                ),
                text(format!("{guard_path}.value-type"), guard.value_type()),
                text(
                    format!("{guard_path}.expected"),
                    worth_foundational::facade::prepare_aspect_value_identity_basis(
                        guard.expected(),
                    )
                    .as_str(),
                ),
            ]);
        }
    }
}

fn append_projections(
    entries: &mut Vec<CanonicalBasisEntry>,
    projections: &[WorthQueryInstalledGraphProjection],
) {
    for (index, projection) in projections.iter().enumerate() {
        let path = format!("projection[{index}]");
        entries.extend([
            text(format!("{path}.result-path"), projection.result_path()),
            text(format!("{path}.query-type"), projection.query_type()),
            text(format!("{path}.slot-type"), projection.slot_type()),
            text(format!("{path}.entity"), projection.entity()),
            text(format!("{path}.aspect"), projection.aspect()),
            text(format!("{path}.field"), projection.field()),
            text(format!("{path}.output"), projection.output_name()),
            text(
                format!("{path}.scalar"),
                projection.scalar_family().canonical_name(),
            ),
            text(format!("{path}.value-type"), projection.value_type()),
            text(
                format!("{path}.presence"),
                field_presence_name(projection.presence()),
            ),
        ]);
    }
}

const fn field_presence_name(
    value: worth_query_declaration::facade::application_schema::ApplicationFieldPresence,
) -> &'static str {
    match value {
        worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Required => {
            "required"
        }
        worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Optional => {
            "optional"
        }
    }
}

fn append_relations(
    entries: &mut Vec<CanonicalBasisEntry>,
    relations: &[WorthQueryInstalledGraphRelation],
) {
    for (index, relation) in relations.iter().enumerate() {
        let path = format!("relation[{index}]");
        entries.extend([
            text(format!("{path}.result-path"), relation.result_path()),
            text(format!("{path}.query-type"), relation.query_type()),
            text(format!("{path}.slot-type"), relation.slot_type()),
            text(format!("{path}.name"), relation.relation()),
            text(format!("{path}.from"), relation.from()),
            text(format!("{path}.to"), relation.to()),
            text(
                format!("{path}.direction"),
                direction_name(relation.direction()),
            ),
            text(format!("{path}.output"), relation.output_name()),
            text(
                format!("{path}.cardinality"),
                cardinality_name(relation.cardinality()),
            ),
            unsigned(format!("{path}.depth"), relation.depth()),
        ]);
        if let Some(predicate) = relation.predicate() {
            let (entity, aspect, field) = predicate.field();
            entries.extend([
                text(format!("{path}.predicate.entity"), entity),
                text(format!("{path}.predicate.aspect"), aspect),
                text(format!("{path}.predicate.field"), field),
                text(format!("{path}.predicate.parameter"), predicate.parameter()),
                text(
                    format!("{path}.predicate.scalar"),
                    predicate.scalar_family().canonical_name(),
                ),
            ]);
        }
    }
}

fn append_predicates(
    entries: &mut Vec<CanonicalBasisEntry>,
    predicates: &[WorthQueryInstalledGraphPredicate],
) {
    for (index, predicate) in predicates.iter().enumerate() {
        let path = format!("predicate[{index}]");
        let (entity, aspect, field) = predicate.field();
        entries.extend([
            text(format!("{path}.entity"), entity),
            text(format!("{path}.aspect"), aspect),
            text(format!("{path}.field"), field),
            text(format!("{path}.parameter"), predicate.parameter()),
            text(
                format!("{path}.scalar"),
                predicate.scalar_family().canonical_name(),
            ),
        ]);
    }
}

fn append_ordering(
    entries: &mut Vec<CanonicalBasisEntry>,
    ordering: &[WorthQueryInstalledGraphOrdering],
) {
    for (index, ordering) in ordering.iter().enumerate() {
        let path = format!("ordering[{index}]");
        let (entity, aspect, field) = ordering.field();
        entries.extend([
            text(format!("{path}.result-path"), ordering.result_path()),
            text(format!("{path}.collection-path"), ordering.collection_path()),
            text(format!("{path}.query-type"), ordering.query_type()),
            text(format!("{path}.slot-type"), ordering.slot_type()),
            text(format!("{path}.entity"), entity),
            text(format!("{path}.aspect"), aspect),
            text(format!("{path}.field"), field),
            text(format!("{path}.output"), ordering.output_name()),
            text(
                format!("{path}.direction"),
                match ordering.direction() {
                    worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection::Ascending => "ascending",
                    worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection::Descending => "descending",
                },
            ),
            text(
                format!("{path}.scalar"),
                ordering.scalar_family().canonical_name(),
            ),
            text(format!("{path}.value-type"), ordering.value_type()),
        ]);
    }
}

const fn cardinality_name(value: ApplicationQueryCardinality) -> &'static str {
    match value {
        ApplicationQueryCardinality::OptionalOne => "optional-one",
        ApplicationQueryCardinality::ExactlyOne => "exactly-one",
        ApplicationQueryCardinality::Many => "many",
    }
}

const fn direction_name(
    value: worth_query_declaration::facade::application_query::ApplicationQueryResultTraversalDirection,
) -> &'static str {
    match value {
        worth_query_declaration::facade::application_query::ApplicationQueryResultTraversalDirection::Forward => "forward",
        worth_query_declaration::facade::application_query::ApplicationQueryResultTraversalDirection::Reverse => "reverse",
    }
}

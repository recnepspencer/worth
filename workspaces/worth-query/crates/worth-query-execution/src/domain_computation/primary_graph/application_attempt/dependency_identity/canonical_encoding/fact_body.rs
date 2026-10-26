//! Canonical semantic content of an observed fact.
use super::*;
pub(super) fn append_fact(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    fact: &WorthQueryApplicationObservedFact,
) {
    match fact {
        WorthQueryApplicationObservedFact::Aggregate { observation } => {
            aggregate::append_values(entries, prefix, observation)
        }
        WorthQueryApplicationObservedFact::StructuralWalk { observation } => {
            structural_walk::append_values(entries, prefix, observation)
        }
        WorthQueryApplicationObservedFact::SourcePublication { observation } => {
            source_publication::append(entries, prefix, observation)
        }
        WorthQueryApplicationObservedFact::RetiredOutputEntity {
            entity_id,
            kind: entity_kind,
            created_at,
            deleted_at,
            ..
        } => {
            kind(entries, prefix, "retired-output-entity");
            entity(entries, prefix, "entity", *entity_id);
            number_u64(
                entries,
                prefix,
                "entity-kind",
                u64::from(entity_kind.as_u32()),
            );
            number_u64(entries, prefix, "created-at", created_at.as_u64());
            number_u64(entries, prefix, "deleted-at", deleted_at.as_u64());
        }
        WorthQueryApplicationObservedFact::AspectPresence {
            kind: entity_kind,
            present,
            ..
        } => aspect_presence::append(entries, prefix, *entity_kind, *present),
        WorthQueryApplicationObservedFact::SourceEntity { .. } => {
            kind(entries, prefix, "source-entity")
        }
        WorthQueryApplicationObservedFact::SourceAspectRevision {
            native_revision, ..
        } => {
            kind(entries, prefix, "source-aspect-revision");
            optional_u64(entries, prefix, "revision", *native_revision);
        }
        WorthQueryApplicationObservedFact::SourceFieldRevision {
            native_revision, ..
        } => {
            kind(entries, prefix, "source-field-revision");
            optional_u64(
                entries,
                prefix,
                "revision",
                native_revision.map(|revision| revision.version().0),
            );
            push(
                entries,
                format!("{prefix}.presence"),
                CanonicalBasisEntryKind::Value,
                native_revision.map_or(CanonicalBasisValue::Null, |revision| {
                    text(match revision.presence() {
                        worth_relational::facade::runtime::RelationalFieldPresence::Present => {
                            "present"
                        }
                        worth_relational::facade::runtime::RelationalFieldPresence::Absent => {
                            "absent"
                        }
                    })
                }),
            );
        }
        WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
            native_revision,
            comparison_work_limit,
            endpoints,
            ..
        } => {
            kind(entries, prefix, "source-adjacency-revision");
            optional_u64(
                entries,
                prefix,
                "revision",
                native_revision.map(|revision| revision.0),
            );
            number(
                entries,
                prefix,
                "comparison-work-limit",
                *comparison_work_limit,
            );
            entities(entries, prefix, "endpoint", endpoints);
        }
        WorthQueryApplicationObservedFact::Entity { .. } => kind(entries, prefix, "entity"),
        WorthQueryApplicationObservedFact::Field {
            kind: field_kind,
            value,
            ..
        } => {
            kind(entries, prefix, "field");
            number_u64(
                entries,
                prefix,
                "entity-kind",
                u64::from(field_kind.as_u32()),
            );
            push(
                entries,
                format!("{prefix}.value"),
                CanonicalBasisEntryKind::Value,
                canonical_basis_value_for_aspect_value(value),
            );
        }
        WorthQueryApplicationObservedFact::AbsentField {
            kind: field_kind, ..
        } => {
            kind(entries, prefix, "absent-field");
            number_u64(
                entries,
                prefix,
                "entity-kind",
                u64::from(field_kind.as_u32()),
            );
        }
        WorthQueryApplicationObservedFact::OptionalAspectAbsent {
            kind: field_kind, ..
        } => {
            kind(entries, prefix, "optional-aspect-absent");
            number_u64(
                entries,
                prefix,
                "entity-kind",
                u64::from(field_kind.as_u32()),
            );
        }
        WorthQueryApplicationObservedFact::Relation {
            matching_relations, ..
        } => {
            kind(entries, prefix, "relation");
            relations(entries, prefix, "match", matching_relations);
        }
        WorthQueryApplicationObservedFact::Adjacency {
            maximum_work_units,
            relations: observed,
            ..
        } => {
            kind(entries, prefix, "adjacency");
            number(entries, prefix, "maximum-work-units", *maximum_work_units);
            number(entries, prefix, "relation-count", observed.len());
            for (index, relation) in observed.iter().enumerate() {
                record(
                    entries,
                    prefix,
                    &format!("relation.{index}.id"),
                    relation.relation_id,
                );
                entity(
                    entries,
                    prefix,
                    &format!("relation.{index}.from"),
                    relation.from,
                );
                entity(
                    entries,
                    prefix,
                    &format!("relation.{index}.to"),
                    relation.to,
                );
            }
        }
        WorthQueryApplicationObservedFact::IndexedEntitySelection { .. } => {
            indexed_selection::append(entries, prefix, fact)
        }
        WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor { .. }
        | WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent { .. }
        | WorthQueryApplicationObservedFact::WorkflowInstanceCapacity { .. }
        | WorthQueryApplicationObservedFact::WorkflowHistoryBasis { .. } => {
            workflow_fact::append(entries, prefix, fact)
        }
    }
}

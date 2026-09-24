use worth_foundational::facade::CanonicalBasisEntry;

use super::{entity, kind, number, optional_entity, record};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

pub(super) fn append(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    fact: &WorthQueryApplicationObservedFact,
) {
    match fact {
        WorthQueryApplicationObservedFact::WorkflowDefinitionPredecessor {
            lineage,
            expected_definition,
            maximum_work_units,
            ..
        } => {
            kind(entries, prefix, "workflow-definition-predecessor");
            number(entries, prefix, "maximum-work-units", *maximum_work_units);
            optional_entity(entries, prefix, "lineage", *lineage);
            optional_entity(entries, prefix, "expected-definition", *expected_definition);
        }
        WorthQueryApplicationObservedFact::WorkflowDefinitionCurrent {
            lineage,
            expected_definition,
            maximum_work_units,
            ..
        } => {
            kind(entries, prefix, "workflow-definition-current");
            number(entries, prefix, "maximum-work-units", *maximum_work_units);
            entity(entries, prefix, "lineage", *lineage);
            entity(entries, prefix, "expected-definition", *expected_definition);
        }
        WorthQueryApplicationObservedFact::WorkflowInstanceCapacity {
            lineage,
            maximum_instances,
            instances,
            ..
        } => {
            kind(entries, prefix, "workflow-instance-capacity");
            entity(entries, prefix, "lineage", *lineage);
            number(entries, prefix, "maximum-instances", *maximum_instances);
            number(entries, prefix, "instance-count", instances.len());
            for (index, relation) in instances.iter().enumerate() {
                record(
                    entries,
                    prefix,
                    &format!("instance.{index}.relation"),
                    relation.relation_id,
                );
                entity(
                    entries,
                    prefix,
                    &format!("instance.{index}.entity"),
                    relation.from,
                );
            }
        }
        WorthQueryApplicationObservedFact::WorkflowTransitionCapacity {
            instance,
            maximum_transitions,
            transitions,
            ..
        } => {
            kind(entries, prefix, "workflow-transition-capacity");
            entity(entries, prefix, "instance", *instance);
            number(entries, prefix, "maximum-transitions", *maximum_transitions);
            number(entries, prefix, "transition-count", transitions.len());
            for (index, relation) in transitions.iter().enumerate() {
                record(
                    entries,
                    prefix,
                    &format!("transition.{index}.relation"),
                    relation.relation_id,
                );
                entity(
                    entries,
                    prefix,
                    &format!("transition.{index}.entity"),
                    relation.to,
                );
            }
        }
        _ => unreachable!("workflow fact dispatcher accepts only workflow facts"),
    }
}

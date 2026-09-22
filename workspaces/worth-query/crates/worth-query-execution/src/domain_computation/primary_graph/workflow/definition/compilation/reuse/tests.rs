use std::sync::Arc;

use worth_relational::facade::identity::{EntityId, PartitionId};

use super::*;
use crate::domain_computation::primary_graph::workflow::definition::compilation::{
    plan::{
        CompiledWorkflowNode, CompiledWorkflowNodeKind, CompiledWorkflowNodeMeaning,
        CompiledWorkflowSemanticPlan,
    },
    publication_binding::{
        separate_compiled_definition, ColdCompiledWorkflowDefinition,
        WorkflowDefinitionPublicationBinding, WorkflowDefinitionPublicationRevisions,
    },
};

#[test]
fn equal_semantics_share_meaning_without_sharing_publication_bindings() {
    let mut reuse = WorkflowDefinitionCompilationReuse::new(16 * 1024);
    let semantic_key = semantic_key("same-content");
    let first_key = publication_key(1, 10);
    let second_key = publication_key(1, 20);
    let first = semantic("terminal");
    let retained = reuse
        .retain(
            first_key,
            semantic_key.clone(),
            Arc::clone(&first),
            binding(10, 11),
        )
        .expect("first publication must fit");
    let shared = reuse
        .retain(
            second_key,
            semantic_key.clone(),
            semantic("terminal"),
            binding(20, 21),
        )
        .expect("equal semantics must share");

    assert!(Arc::ptr_eq(&retained, &shared));
    let first_reused = reuse
        .reuse(first_key, &semantic_key)
        .expect("first binding remains retained");
    let second_reused = reuse
        .reuse(second_key, &semantic_key)
        .expect("second binding remains retained");
    assert_ne!(
        first_reused.binding.definition,
        second_reused.binding.definition
    );
    assert_ne!(
        first_reused.binding.node_entities,
        second_reused.binding.node_entities
    );
    assert_eq!(reuse.counters().cold_retains(), 2);
    assert_eq!(reuse.counters().semantic_reuse_hits(), 1);
}

#[test]
fn same_reuse_key_denies_different_semantics() {
    let mut reuse = WorkflowDefinitionCompilationReuse::new(16 * 1024);
    let key = semantic_key("collision");
    let publication = publication_key(1, 10);
    reuse
        .retain(publication, key.clone(), semantic("first"), binding(10, 11))
        .expect("first meaning must fit");

    assert_eq!(
        reuse.retain(publication, key, semantic("different"), binding(20, 21),),
        Err(WorkflowDefinitionCompilationReuseDenial::SemanticCollision)
    );
    assert_eq!(
        reuse
            .reuse(publication, &semantic_key("collision"))
            .expect("collision denial must preserve the retained compilation")
            .semantic
            .nodes[0]
            .path
            .as_str(),
        "first"
    );
}

#[test]
fn equal_content_under_different_installed_support_does_not_share_semantics() {
    let mut reuse = WorkflowDefinitionCompilationReuse::new(16 * 1024);
    let first_key = semantic_key("same-content");
    let mut changed_support_key = first_key.clone();
    changed_support_key.support_identity = [9; 32];
    let first = reuse
        .retain(
            publication_key(1, 10),
            first_key,
            semantic("terminal"),
            binding(10, 11),
        )
        .expect("first support must fit");
    let changed = reuse
        .retain(
            publication_key(1, 20),
            changed_support_key,
            semantic("terminal"),
            binding(20, 21),
        )
        .expect("changed support must compile independently");

    assert!(!Arc::ptr_eq(&first, &changed));
    assert_eq!(reuse.counters().semantic_reuse_hits(), 0);
}

#[test]
fn byte_budget_evicts_the_least_recent_publication_and_releases_meaning() {
    let first_key = semantic_key("first");
    let mut sizing = WorkflowDefinitionCompilationReuse::new(16 * 1024);
    sizing
        .retain(
            publication_key(1, 10),
            first_key.clone(),
            semantic("first"),
            binding(10, 11),
        )
        .expect("sizing entry must fit");
    let one_entry_budget = sizing.retained_bytes();
    let mut reuse = WorkflowDefinitionCompilationReuse::new(one_entry_budget);
    reuse
        .retain(
            publication_key(1, 10),
            first_key.clone(),
            semantic("first"),
            binding(10, 11),
        )
        .expect("first entry must fit exact budget");
    let second_key = semantic_key("other");
    reuse
        .retain(
            publication_key(1, 20),
            second_key.clone(),
            semantic("other"),
            binding(20, 21),
        )
        .expect("second entry must evict the first");

    assert!(reuse.reuse(publication_key(1, 10), &first_key).is_none());
    assert!(reuse.reuse(publication_key(1, 20), &second_key).is_some());
    assert!(reuse.retained_bytes() <= one_entry_budget);
    assert_eq!(reuse.counters().evictions(), 1);
}

#[test]
fn oversized_replacement_preserves_the_retained_publication_and_budget() {
    let key = semantic_key("retained");
    let publication = publication_key(1, 10);
    let mut sizing = WorkflowDefinitionCompilationReuse::new(16 * 1024);
    sizing
        .retain(
            publication,
            key.clone(),
            semantic("retained"),
            binding(10, 11),
        )
        .expect("retained entry must fit");
    let retained_bytes = sizing.retained_bytes();
    let mut oversized = binding(20, 21);
    oversized.node_entities = Arc::from(vec![entity(21); 16 * 1024]);

    assert_eq!(
        sizing.retain(publication, key.clone(), semantic("retained"), oversized),
        Err(WorkflowDefinitionCompilationReuseDenial::ByteBudgetExceeded)
    );
    assert!(sizing.reuse(publication, &key).is_some());
    assert_eq!(sizing.retained_bytes(), retained_bytes);
    assert_eq!(sizing.counters().denials(), 1);
    assert_eq!(sizing.counters().evictions(), 0);
}

#[test]
fn production_counters_distinguish_cold_misses_from_warm_hits() {
    let key = semantic_key("counted");
    let publication = publication_key(1, 10);
    let mut reuse = WorkflowDefinitionCompilationReuse::new(16 * 1024);

    assert!(reuse.reuse(publication, &key).is_none());
    reuse
        .retain(
            publication,
            key.clone(),
            semantic("counted"),
            binding(10, 11),
        )
        .expect("counted entry must fit");
    assert!(reuse.reuse(publication, &key).is_some());

    let counters = reuse.counters();
    assert_eq!(counters.cold_misses(), 1);
    assert_eq!(counters.cold_retains(), 1);
    assert_eq!(counters.warm_hits(), 1);
    assert_eq!(counters.retained_bytes(), reuse.retained_bytes());
    assert_eq!(counters.maximum_retained_bytes(), 16 * 1024);
}

fn semantic_key(identity: &str) -> WorkflowDefinitionSemanticReuseKey {
    WorkflowDefinitionSemanticReuseKey {
        content_identity: identity.to_owned(),
        program_revision: "program".to_owned(),
        vocabulary_identity: "vocabulary".to_owned(),
        support_identity: [7; 32],
    }
}

fn publication_key(
    branch_occurrence: u64,
    definition_slot: u64,
) -> WorkflowDefinitionPublicationReuseKey {
    WorkflowDefinitionPublicationReuseKey {
        branch_occurrence,
        definition: entity(definition_slot),
    }
}

fn semantic(path: &str) -> Arc<CompiledWorkflowSemanticPlan> {
    compilation(path, 10, 11).0
}

fn binding(definition_slot: u64, node_slot: u64) -> WorkflowDefinitionPublicationBinding {
    compilation("terminal", definition_slot, node_slot).1
}

fn compilation(
    path: &str,
    definition_slot: u64,
    node_slot: u64,
) -> (
    Arc<CompiledWorkflowSemanticPlan>,
    WorkflowDefinitionPublicationBinding,
) {
    separate_compiled_definition(ColdCompiledWorkflowDefinition {
        lineage: entity(1),
        definition: entity(definition_slot),
        start_node: entity(node_slot),
        nodes: vec![CompiledWorkflowNode {
            entity: entity(node_slot),
            meaning: Arc::new(CompiledWorkflowNodeMeaning {
                path: path.to_owned(),
                kind: CompiledWorkflowNodeKind::Terminal,
            }),
        }]
        .into_boxed_slice(),
        connections: Box::default(),
        revisions: WorkflowDefinitionPublicationRevisions {
            start: None,
            nodes: None,
            connections: None,
        },
    })
    .expect("fixture definition must separate")
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(1), slot, 1)
}

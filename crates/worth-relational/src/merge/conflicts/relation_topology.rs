use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use crate::identity::data::EntityId;
use crate::merge::conflicts::ancestor_record_basis::AncestorRecordBasisContext;
use crate::merge::conflicts::target_record_resolution::visible_target_record;
use crate::merge::data::{
    MergeConflictClassification, RelationConflictPropagation, TopologyRegionConflictReason,
    VisibleMergeRecord,
};
use crate::storage::data::RelationalReadView;
use crate::transactions::data::RecordRef;

pub(super) fn refine_relation_topology_conflicts(
    runtime: &crate::runtime::RelationalRuntime,
    source_records_by_ref: &BTreeMap<RecordRef, &VisibleMergeRecord>,
    ancestor_basis: &AncestorRecordBasisContext<'_>,
    target_view: &RelationalReadView,
    mut classifications: Vec<MergeConflictClassification>,
) -> Vec<MergeConflictClassification> {
    let mut relation_nodes = Vec::<RelationTopologyNode>::new();
    for (index, classification) in classifications.iter().enumerate() {
        let Some(relation_evidence) = classification.relation_evidence.as_ref() else {
            continue;
        };
        let Some(source_record) = source_records_by_ref.get(&classification.record) else {
            continue;
        };
        let endpoint_ids = relation_topology_endpoints(
            source_record,
            classification.target_record.as_ref(),
            ancestor_basis,
            target_view,
        );
        if endpoint_ids.is_empty() {
            continue;
        }
        relation_nodes.push(RelationTopologyNode {
            classification_index: index,
            record: classification.record.clone(),
            endpoint_ids,
            rewired: relation_evidence.propagation
                == RelationConflictPropagation::RelationLocalRewireCandidate,
        });
    }

    if relation_nodes.is_empty() {
        return classifications;
    }

    let mut endpoint_index = BTreeMap::<EntityId, Vec<usize>>::new();
    for (node_index, node) in relation_nodes.iter().enumerate() {
        for endpoint in &node.endpoint_ids {
            endpoint_index
                .entry(*endpoint)
                .or_default()
                .push(node_index);
        }
    }

    let mut visited = vec![false; relation_nodes.len()];
    let mut scoped_relation_candidates = 0;
    let mut scoped_endpoint_incidences = 0;
    let mut region_conflicts_detected = 0;
    let mut region_records_escalated = 0;
    for node_index in 0..relation_nodes.len() {
        if visited[node_index] {
            continue;
        }
        let component =
            relation_topology_component(node_index, &relation_nodes, &endpoint_index, &mut visited);
        let component_has_rewire_seed = component
            .iter()
            .any(|component_index| relation_nodes[*component_index].rewired);
        if !component_has_rewire_seed {
            continue;
        }
        scoped_relation_candidates += component.len();
        scoped_endpoint_incidences += component
            .iter()
            .map(|component_index| relation_nodes[*component_index].endpoint_ids.len())
            .sum::<usize>();
        let component_records = Arc::<[RecordRef]>::from(
            component
                .iter()
                .map(|component_index| relation_nodes[*component_index].record.clone())
                .collect::<Vec<_>>(),
        );
        let rewired_records = Arc::<[RecordRef]>::from(
            component
                .iter()
                .filter(|component_index| relation_nodes[**component_index].rewired)
                .map(|component_index| relation_nodes[*component_index].record.clone())
                .collect::<Vec<_>>(),
        );
        let escalate_to_region = rewired_records.len() > 1;
        if escalate_to_region {
            region_conflicts_detected += 1;
            region_records_escalated += rewired_records.len();
        }
        for component_index in component {
            let classification =
                &mut classifications[relation_nodes[component_index].classification_index];
            if let Some(evidence) = classification.relation_evidence.as_mut() {
                evidence.topology_neighborhood_records = component_records.clone();
                evidence.topology_neighborhood_rewired_records = rewired_records.clone();
                if relation_nodes[component_index].rewired && escalate_to_region {
                    evidence.propagation =
                        RelationConflictPropagation::EscalatesToTopologyRegionConflict;
                    evidence.topology_region_conflict_reason =
                        Some(TopologyRegionConflictReason::ConnectedRewireNeighborhood);
                }
            }
        }
    }

    runtime
        .performance_access()
        .count_merge_topology_region_detection(
            scoped_relation_candidates,
            scoped_endpoint_incidences,
            region_conflicts_detected,
            region_records_escalated,
        );
    classifications
}

#[derive(Debug)]
struct RelationTopologyNode {
    classification_index: usize,
    record: RecordRef,
    endpoint_ids: BTreeSet<EntityId>,
    rewired: bool,
}

fn relation_topology_component(
    seed_index: usize,
    nodes: &[RelationTopologyNode],
    endpoint_index: &BTreeMap<EntityId, Vec<usize>>,
    visited: &mut [bool],
) -> Vec<usize> {
    let mut queue = VecDeque::from([seed_index]);
    let mut component = Vec::new();
    visited[seed_index] = true;
    while let Some(node_index) = queue.pop_front() {
        component.push(node_index);
        for endpoint in &nodes[node_index].endpoint_ids {
            if let Some(neighbors) = endpoint_index.get(endpoint) {
                for neighbor in neighbors {
                    if !visited[*neighbor] {
                        visited[*neighbor] = true;
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
    }
    component.sort_by(|left, right| nodes[*left].record.cmp(&nodes[*right].record));
    component
}

fn relation_topology_endpoints(
    source_record: &VisibleMergeRecord,
    candidate_target_record: Option<&RecordRef>,
    ancestor_basis: &AncestorRecordBasisContext<'_>,
    target_view: &RelationalReadView,
) -> BTreeSet<EntityId> {
    let mut endpoints = BTreeSet::new();
    if let Some(source) = source_record.source_relation.as_ref() {
        endpoints.insert(source.source);
        endpoints.insert(source.target);
    }
    if let Some(target_relation) = candidate_target_record
        .and_then(|target_record| visible_target_record(target_view, target_record))
        .and_then(|record| record.target_relation)
        .or_else(|| source_record.target_relation.clone())
    {
        endpoints.insert(target_relation.source);
        endpoints.insert(target_relation.target);
    }
    if let Some(base) = ancestor_basis.relation_basis(source_record, candidate_target_record) {
        endpoints.insert(base.source_endpoint());
        endpoints.insert(base.target_endpoint());
    }
    endpoints
}

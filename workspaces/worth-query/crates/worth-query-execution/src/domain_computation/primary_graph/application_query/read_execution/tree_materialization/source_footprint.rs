use worth_query_declaration::facade::application_query::ApplicationQueryResultTraversalDirection;
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;
use worth_relational::facade::runtime::{RelationalAdjacencyDirection, VisibilityProjectionView};

use super::{allocate_claimed_result_vector, projection_denial, ResultTreeWork};
use crate::domain_computation::primary_graph::application_query::{
    disclosure::WorthQueryApplicationQueryGovernance,
    observed_source::{
        WorthQueryObservedAdjacencyRevision, WorthQueryObservedAspectRevision,
        WorthQueryObservedSourceFootprint,
    },
    projection::WorthQueryApplicationProjectionNode,
    read_execution::WorthQueryApplicationReadExecutionDenial,
    resource_lifecycle::WorthQueryApplicationResultBufferReservation,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

pub(super) fn collect_source_footprints(
    projection: &VisibilityProjectionView<'_>,
    graph: &WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &WorthQueryApplicationQueryGovernance,
    roots: &[WorthQueryApplicationProjectionNode],
    work: &mut ResultTreeWork,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<Vec<WorthQueryObservedSourceFootprint>, WorthQueryApplicationReadExecutionDenial> {
    let mut footprints = allocate_claimed_result_vector(result_buffer, roots.len(), "source")?;
    for root in roots {
        let counts = footprint_counts(contract, governance, root);
        let mut footprint = WorthQueryObservedSourceFootprint {
            root: root.entity_id(),
            complete: true,
            entities: allocate_claimed_result_vector(
                result_buffer,
                counts.entities,
                root.result_path(),
            )?,
            aspects: allocate_claimed_result_vector(
                result_buffer,
                counts.fields,
                root.result_path(),
            )?,
            adjacencies: allocate_claimed_result_vector(
                result_buffer,
                counts.relations,
                root.result_path(),
            )?,
        };
        collect_node(
            projection,
            graph,
            contract,
            governance,
            root,
            work,
            result_buffer,
            &mut footprint,
        )?;
        normalize_source_footprint(&mut footprint);
        footprints.push(footprint);
    }
    Ok(footprints)
}

fn normalize_source_footprint(footprint: &mut WorthQueryObservedSourceFootprint) {
    footprint
        .aspects
        .sort_by(|left, right| (left.entity, &left.aspect).cmp(&(right.entity, &right.aspect)));
    footprint.aspects.dedup();
    footprint.entities.sort();
    footprint.entities.dedup();
    footprint.adjacencies.sort_by(|left, right| {
        (left.anchor, left.relation_kind, left.direction as u8).cmp(&(
            right.anchor,
            right.relation_kind,
            right.direction as u8,
        ))
    });
    footprint.adjacencies.dedup();
}

#[derive(Clone, Copy, Default)]
struct FootprintCounts {
    entities: usize,
    fields: usize,
    relations: usize,
}

fn footprint_counts(
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &WorthQueryApplicationQueryGovernance,
    node: &WorthQueryApplicationProjectionNode,
) -> FootprintCounts {
    node.relations().iter().fold(
        FootprintCounts {
            entities: 1,
            fields: unique_source_aspect_count(contract, governance, node.result_path()),
            relations: node.relations().len(),
        },
        |mut counts, relation| {
            for child in relation.rows() {
                let child = footprint_counts(contract, governance, child);
                counts.entities = counts.entities.saturating_add(child.entities);
                counts.fields = counts.fields.saturating_add(child.fields);
                counts.relations = counts.relations.saturating_add(child.relations);
            }
            counts
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn collect_node(
    projection: &VisibilityProjectionView<'_>,
    graph: &WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &WorthQueryApplicationQueryGovernance,
    node: &WorthQueryApplicationProjectionNode,
    work: &mut ResultTreeWork,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
    footprint: &mut WorthQueryObservedSourceFootprint,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    footprint.complete &= node.source_dependencies_complete();
    footprint.entities.push(node.entity_id());
    work.charge_source_observation(1, node.result_path())?;
    let node_aspect_start = footprint.aspects.len();
    for field in contract.projections().iter().filter(|field| {
        field.parent_path() == node.result_path()
            && governance.is_disclosed(field.slot_key_identity().as_ref())
    }) {
        if footprint.aspects[node_aspect_start..]
            .iter()
            .any(|stamped| stamped.aspect == *field.aspect_key())
        {
            continue;
        }
        let contract_revision = graph
            .aspect_contract(field.entity(), field.aspect_key())
            .ok_or_else(|| projection_denial(field.result_path()))?
            .revision();
        result_buffer
            .claim(field.entity().len().saturating_add(field.aspect().len()))
            .map_err(|()| super::result_buffer_denial(field.result_path()))?;
        footprint.aspects.push(WorthQueryObservedAspectRevision {
            entity: node.entity_id(),
            entity_name: field.entity().to_owned(),
            aspect: field.aspect_key().clone(),
            contract_revision,
            native_revision: projection
                .entity_aspect_version(node.entity_id(), field.aspect_key())
                .ok_or_else(|| projection_denial(field.result_path()))?,
        });
        work.charge_source_observation(1, field.result_path())?;
    }
    for projected in node.relations() {
        let relation = contract
            .relations()
            .iter()
            .find(|relation| relation.slot_type() == projected.slot_type())
            .ok_or_else(|| projection_denial(projected.result_path()))?;
        let layout = graph
            .relation(relation.relation())
            .ok_or_else(|| projection_denial(relation.result_path()))?;
        let direction = match relation.direction() {
            ApplicationQueryResultTraversalDirection::Forward => {
                RelationalAdjacencyDirection::Outgoing
            }
            ApplicationQueryResultTraversalDirection::Reverse => {
                RelationalAdjacencyDirection::Incoming
            }
        };
        let revision = projection
            .bounded_adjacency_structural_revision(
                node.entity_id(),
                layout.kind,
                direction,
                work.remaining_work(),
            )
            .map_err(|_| {
                crate::domain_computation::primary_graph::application_query::read_execution::read_execution_denial(
                    crate::domain_computation::primary_graph::application_query::read_execution::WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                    relation.result_path(),
                )
            })?;
        work.charge_source_observation(revision.work_units(), relation.result_path())?;
        let mut endpoints = allocate_claimed_result_vector(
            result_buffer,
            projected.rows().len(),
            relation.result_path(),
        )?;
        endpoints.extend(
            projected
                .rows()
                .iter()
                .map(WorthQueryApplicationProjectionNode::entity_id),
        );
        footprint
            .adjacencies
            .push(WorthQueryObservedAdjacencyRevision {
                anchor: node.entity_id(),
                relation_kind: layout.kind,
                direction,
                native_revision: revision.revision(),
                comparison_work_limit: revision.work_units(),
                endpoints,
            });
        for child in projected.rows() {
            collect_node(
                projection,
                graph,
                contract,
                governance,
                child,
                work,
                result_buffer,
                footprint,
            )?;
        }
    }
    Ok(())
}

fn unique_source_aspect_count(
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &WorthQueryApplicationQueryGovernance,
    result_path: &str,
) -> usize {
    contract
        .projections()
        .iter()
        .enumerate()
        .filter(|(index, field)| {
            field.parent_path() == result_path
                && governance.is_disclosed(field.slot_key_identity().as_ref())
                && !contract.projections()[..*index].iter().any(|earlier| {
                    earlier.parent_path() == result_path
                        && governance.is_disclosed(earlier.slot_key_identity().as_ref())
                        && earlier.aspect_key() == field.aspect_key()
                })
        })
        .count()
}

#[cfg(test)]
mod tests;

use worth_query_declaration::facade::application_query::ApplicationQueryResultTraversalDirection;
use worth_query_installation::facade::WorthQueryInstalledGraphReadContract;
use worth_relational::facade::runtime::{
    AdjacencyStructuralRevisionDenial, RelationalAdjacencyDirection, VisibilityProjectionView,
};

use super::{allocate_result_vector, projection_denial, ResultTreeWork};
use crate::domain_computation::primary_graph::application_query::{
    disclosure::WorthQueryApplicationQueryGovernance,
    observed_source::{
        WorthQueryObservedAdjacencyRevision, WorthQueryObservedFieldRevision,
        WorthQueryObservedSourceFootprint,
    },
    projection::WorthQueryApplicationProjectionNode,
    read_execution::WorthQueryApplicationReadExecutionDenial,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;

pub(super) fn collect_source_footprints(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &VisibilityProjectionView,
    graph: &WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &WorthQueryApplicationQueryGovernance,
    roots: &[WorthQueryApplicationProjectionNode],
    selected_predicate_source: Option<&WorthQueryObservedFieldRevision>,
    root_path_source: Option<&std::collections::BTreeMap<worth_relational::facade::identity::EntityId, std::sync::Arc<crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedRootSelection>>>,
    work: &mut ResultTreeWork,
) -> Result<Vec<WorthQueryObservedSourceFootprint>, WorthQueryApplicationReadExecutionDenial> {
    let mut footprints = allocate_result_vector(roots.len(), "source")?;
    for root in roots {
        let mut footprint = WorthQueryObservedSourceFootprint {
            root: root.entity_id(),
            complete: true,
            entities: Vec::new(),
            aspects: Vec::new(),
            adjacencies: Vec::new(),
            root_selection: root_path_source
                .map(|sources| {
                    sources
                        .get(&root.entity_id())
                        .cloned()
                        .ok_or_else(|| projection_denial(root.result_path()))
                })
                .transpose()?,
        };
        collect_node(
            execution,
            projection,
            graph,
            contract,
            governance,
            root,
            work,
            &mut footprint,
        )?;
        if let Some(source) = selected_predicate_source {
            if source.entity != root.entity_id() {
                return Err(projection_denial(root.result_path()));
            }
            match footprint.aspects.iter().find(|observed| {
                observed.entity == source.entity
                    && observed.aspect == source.aspect
                    && observed.field == source.field
            }) {
                Some(observed) if observed != source => {
                    return Err(projection_denial(root.result_path()));
                }
                Some(_) => {}
                None => {
                    footprint.aspects.push(source.clone());
                    work.charge_source_observation(1, root.result_path())?;
                }
            }
        }
        work.checkpoint(root.result_path())?;
        normalize_source_footprint(&mut footprint);
        work.checkpoint(root.result_path())?;

        footprints.push(footprint);
    }
    Ok(footprints)
}

fn normalize_source_footprint(footprint: &mut WorthQueryObservedSourceFootprint) {
    footprint.aspects.sort_by(|left, right| {
        (left.entity, &left.aspect, &left.field).cmp(&(right.entity, &right.aspect, &right.field))
    });
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

#[allow(clippy::too_many_arguments)]
fn collect_node(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    projection: &VisibilityProjectionView,
    graph: &WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &WorthQueryApplicationQueryGovernance,
    node: &WorthQueryApplicationProjectionNode,
    work: &mut ResultTreeWork,

    footprint: &mut WorthQueryObservedSourceFootprint,
) -> Result<(), WorthQueryApplicationReadExecutionDenial> {
    footprint.complete &= node.source_dependencies_complete();
    work.checkpoint(node.result_path())?;
    footprint.entities.push(node.entity_id());
    work.charge_source_observation(1, node.result_path())?;
    let node_aspect_start = footprint.aspects.len();
    for field in contract.projections().iter().filter(|field| {
        field.parent_path() == node.result_path()
            && governance.is_disclosed(field.slot_key_identity().as_ref())
    }) {
        if footprint.aspects[node_aspect_start..]
            .iter()
            .any(|stamped| {
                stamped.aspect == *field.aspect_key() && stamped.field == *field.field_key()
            })
        {
            continue;
        }
        let contract_revision = graph
            .aspect_contract(field.entity(), field.aspect_key())
            .ok_or_else(|| projection_denial(field.result_path()))?
            .revision();
        let entity_name = field.entity().to_owned();
        let aspect = field.aspect_key().clone();
        let field_key = field.field_key().clone();

        footprint.aspects.push(WorthQueryObservedFieldRevision {
            entity: node.entity_id(),
            entity_name,
            aspect,
            field: field_key,
            contract_revision,
            native_revision: projection.entity_field_revision(
                execution,
                node.entity_id(),
                &worth_foundational::facade::AspectFieldLocator::new(
                    worth_foundational::facade::LocatorAuthority::Authoritative,
                    field.aspect_key().clone(),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        field.field_key().clone(),
                    ),
                ),
            ).map_err(|cause| crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(cause, node.result_path()))?,
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
        work.checkpoint(relation.result_path())?;
        let revision = projection
            .bounded_adjacency_structural_revision(execution,
                node.entity_id(),
                layout.kind,
                direction,
                1,
            )
            .map_err(|unavailable| match unavailable {
                AdjacencyStructuralRevisionDenial::Execution(cause) => super::super::denial::relational_read_denial(cause, relation.result_path()),
                AdjacencyStructuralRevisionDenial::WorkBudgetExceeded => {
                    crate::domain_computation::primary_graph::application_query::read_execution::read_execution_denial(
                        crate::domain_computation::primary_graph::application_query::read_execution::WorthQueryApplicationReadExecutionDenialKind::WorkLimitExceeded,
                        relation.result_path(),
                    )
                }
                AdjacencyStructuralRevisionDenial::AnchorUnavailable
                | AdjacencyStructuralRevisionDenial::BasisUnavailable => {
                    projection_denial(relation.result_path())
                }
            })?;
        work.charge_source_observation(revision.work_units(), relation.result_path())?;
        if let Some(predicate) = relation.predicate() {
            let contract_revision = graph
                .aspect_contract(relation.child_entity(), predicate.aspect_key())
                .ok_or_else(|| projection_denial(relation.result_path()))?
                .revision();
            for entity_id in projected.predicate_sources() {
                let entity_name = relation.child_entity().to_owned();
                let aspect = predicate.aspect_key().clone();
                let field = predicate.field_key().clone();

                footprint.entities.push(*entity_id);
                footprint.aspects.push(WorthQueryObservedFieldRevision {
                    entity: *entity_id,
                    entity_name,
                    aspect,
                    field,
                    contract_revision,
                    native_revision: projection.entity_field_revision(
                        execution,
                        *entity_id,
                        &worth_foundational::facade::AspectFieldLocator::new(
                            worth_foundational::facade::LocatorAuthority::Authoritative,
                            predicate.aspect_key().clone(),
                            worth_foundational::facade::CanonicalFieldPath::single(
                                predicate.field_key().clone(),
                            ),
                        ),
                    ).map_err(|cause| crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(cause, node.result_path()))?,
                });
                work.charge_source_observation(1, relation.result_path())?;
            }
        }
        let filtered_relation = relation.predicate().is_some();
        let mut endpoints = allocate_result_vector(
            if filtered_relation {
                projected.predicate_sources().len()
            } else {
                projected.rows().len()
            },
            relation.result_path(),
        )?;
        if filtered_relation {
            endpoints.extend_from_slice(projected.predicate_sources());
        } else {
            endpoints.extend(
                projected
                    .rows()
                    .iter()
                    .map(WorthQueryApplicationProjectionNode::entity_id),
            );
        }
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
                execution, projection, graph, contract, governance, child, work, footprint,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

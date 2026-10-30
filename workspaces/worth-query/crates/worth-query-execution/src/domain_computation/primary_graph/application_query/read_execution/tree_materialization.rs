use crate::domain_computation::primary_graph::read_observation::WorthQueryInvariantProjectionWork;
use std::collections::BTreeMap;
use std::sync::Arc;

use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::{
    WorthQueryInstalledGraphProjection, WorthQueryInstalledGraphReadContract,
    WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::{
    identity::EntityId,
    indexes::{DerivedIndexGenerationId, DerivedIndexId, RelatedEntityOrderingBoundary},
    storage::RecordLifecycleState,
};

use super::{
    read_execution_denial, OneShotReadWorkObservation, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::application_query::projection::{
    WorthQueryApplicationDisclosedProjectionTree, WorthQueryApplicationProjectedField,
    WorthQueryApplicationProjectedRelation, WorthQueryApplicationProjectionNode,
    WorthQueryApplicationWorkingProjectionTree,
};

mod bounded_ordering;
mod collection_selection;
pub(super) use collection_selection::*;
pub(in crate::domain_computation::primary_graph::application_query) mod fallible_heap;
mod fields;
mod material_storage;
mod relation_attachment;
mod relation_distribution;
mod relation_occurrence;
mod relation_target_filter;
mod source_footprint;
mod work;
use fields::{direct_projections, project_fields, projection_scope};

use bounded_ordering::order_collection;
use relation_attachment::{advance_omitted_relation, attach_relation};
use work::ResultTreeWork;

pub(super) struct MaterializedApplicationResultTree {
    pub(super) rows: WorthQueryApplicationDisclosedProjectionTree,
    pub(super) source_footprints:
        Vec<crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint>,
    pub(super) projected_records: usize,
    pub(super) projected_fields: usize,
    pub(super) reads: WorthQueryInvariantProjectionWork,
    pub(super) ordering_comparisons: usize,
    pub(super) ordered_index_entries_examined: usize,
    pub(super) relation_predicate_work_units: usize,
    pub(super) work_units: usize,
    pub(super) continuation: Option<OrderedCollectionProgress>,
}

pub(super) fn materialize_result_tree(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    root_ids: &[EntityId],
    selected_predicate_source: Option<&crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedFieldRevision>,
    root_path_source: Option<&BTreeMap<EntityId, Arc<crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedRootSelection>>>,

    collection_selection: ResultTreeCollectionSelection,
    interruption: super::ReadInterruption<'_>,

    spent: Option<&OneShotReadWorkObservation<'_>>,
    material_source: Option<
        &Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
    >,
) -> Result<MaterializedApplicationResultTree, WorthQueryApplicationReadExecutionDenial> {
    let projection = runtime
        .project_snapshot(snapshot)
        .ok_or_else(|| projection_denial(contract.root_entity()))?;
    let mut work = ResultTreeWork::new(interruption, spent, *execution, material_source);
    work.checkpoint("root")?;
    let mut collection_selection = ActiveResultTreeCollectionSelection::new(collection_selection);
    let mut rows = project_nodes(
        execution,
        runtime,
        &projection,
        graph,
        contract,
        governance,
        parameters,
        None,
        "root",
        contract.root_entity(),
        root_ids,
        &mut work,
        &mut collection_selection,
    )?;
    if rows.len() != root_ids.len() {
        return Err(projection_denial(contract.root_entity()));
    }
    order_collection(contract, governance, "root", &mut rows, &mut work)?;
    let source_footprints = source_footprint::collect_source_footprints(
        execution,
        &projection,
        graph,
        contract,
        governance,
        &rows,
        selected_predicate_source,
        root_path_source,
        &mut work,
        material_source,
    )?;
    let rows = WorthQueryApplicationWorkingProjectionTree::new(rows).into_disclosed(governance);
    Ok(MaterializedApplicationResultTree {
        rows,
        source_footprints,
        projected_records: work.projected_records,
        projected_fields: work.projected_fields,
        reads: work.reads,
        ordering_comparisons: work.ordering_comparisons,
        ordered_index_entries_examined: work.ordered_index_entries_examined,
        relation_predicate_work_units: work.relation_predicate_work_units,
        work_units: work.work_units,
        continuation: collection_selection.into_progress(),
    })
}

#[allow(clippy::too_many_arguments)]
fn project_nodes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    source_path: Option<Arc<str>>,
    result_path: &str,
    entity_name: &str,
    entity_ids: &[EntityId],
    work: &mut ResultTreeWork,
    collection_selection: &mut ActiveResultTreeCollectionSelection,
) -> Result<Vec<WorthQueryApplicationProjectionNode>, WorthQueryApplicationReadExecutionDenial> {
    let fields = direct_projections(contract, governance, result_path, work)?;
    let relations = direct_relations(contract, result_path, work)?;
    let source_dependencies_complete = contract
        .projections()
        .iter()
        .filter(|projection| projection.parent_path() == result_path)
        .all(|projection| governance.is_disclosed(projection.slot_key_identity().as_ref()))
        && relations
            .iter()
            .all(|relation| governance.is_disclosed(relation.slot_key_identity().as_ref()));
    work.charge_projection(entity_ids.len(), fields.len(), result_path)?;
    work.material_slots::<WorthQueryApplicationProjectionNode>(entity_ids.len())?;
    let mut nodes = allocate_result_vector::<WorthQueryApplicationProjectionNode>(
        entity_ids.len(),
        result_path,
    )?;
    let scope = projection_scope(&fields);
    let kind = graph
        .entity_kind(entity_name)
        .ok_or_else(|| projection_denial(entity_name))?;
    for entity_id in entity_ids {
        work.checkpoint(result_path)?;
        let projected_fields = projection
            .entity_record_with_projection_scope(execution, *entity_id, scope.clone(), |record| {
                work.reads.record_endpoint();
                (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live).then(
                    || {
                        work.material_slots::<WorthQueryApplicationProjectedField>(fields.len())?;
                        let projected = allocate_result_vector::<
                            WorthQueryApplicationProjectedField,
                        >(fields.len(), result_path)?;
                        project_fields(record, &fields, projected, work)
                    },
                )
            }).map_err(|cause| crate::domain_computation::primary_graph::application_query::read_execution::denial::relational_read_denial(cause, result_path))?
            .ok_or_else(|| projection_denial(result_path))??;
        work.material_slots::<WorthQueryApplicationProjectedRelation>(relations.len())?;
        let projected_relations = allocate_result_vector::<WorthQueryApplicationProjectedRelation>(
            relations.len(),
            result_path,
        )?;
        nodes.push(WorthQueryApplicationProjectionNode::new(
            *entity_id,
            source_path.clone(),
            source_dependencies_complete,
            projected_fields,
            projected_relations,
        ));
    }
    for relation in relations {
        work.checkpoint(relation.result_path())?;
        if !governance.is_disclosed(relation.slot_key_identity().as_ref()) {
            advance_omitted_relation(
                execution,
                runtime,
                graph,
                relation,
                &nodes,
                work,
                collection_selection,
            )?;
            continue;
        }
        attach_relation(
            execution,
            runtime,
            projection,
            graph,
            contract,
            governance,
            parameters,
            relation,
            &mut nodes,
            work,
            collection_selection,
        )?;
    }
    Ok(nodes)
}

pub(super) fn allocate_result_vector<T>(
    capacity: usize,
    subject: &str,
) -> Result<Vec<T>, WorthQueryApplicationReadExecutionDenial> {
    let mut vector = Vec::new();
    vector
        .try_reserve_exact(capacity)
        .map_err(|_| result_buffer_denial(subject))?;
    Ok(vector)
}

fn direct_relations<'a>(
    contract: &'a WorthQueryInstalledGraphReadContract,
    parent: &str,
    work: &ResultTreeWork,
) -> Result<Vec<&'a WorthQueryInstalledGraphRelation>, WorthQueryApplicationReadExecutionDenial> {
    work.collect_material(
        contract.relations().len(),
        contract
            .relations()
            .iter()
            .filter(|relation| relation.parent_path() == parent),
        parent,
    )
}

fn projection_denial(subject: impl Into<String>) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::ProjectionUnavailable,
        subject,
    )
}

fn result_buffer_denial(subject: impl Into<String>) -> WorthQueryApplicationReadExecutionDenial {
    read_execution_denial(
        WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded,
        subject,
    )
}

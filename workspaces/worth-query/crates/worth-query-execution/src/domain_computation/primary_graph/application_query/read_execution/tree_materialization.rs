use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::{
    ApplicationFieldPresence, WorthQueryInstalledGraphProjection,
    WorthQueryInstalledGraphReadContract, WorthQueryInstalledGraphRelation,
};
use worth_relational::facade::{
    identity::EntityId,
    indexes::{DerivedIndexGenerationId, DerivedIndexId, RelatedEntityOrderingBoundary},
    runtime::{ProjectionAspectRequirement, ProjectionAspectScope},
    storage::RecordLifecycleState,
};

use super::{
    read_execution_denial, WorthQueryApplicationReadExecutionDenial,
    WorthQueryApplicationReadExecutionDenialKind,
};
use crate::domain_computation::primary_graph::application_query::projection::{
    WorthQueryApplicationDisclosedProjectionTree, WorthQueryApplicationProjectedField,
    WorthQueryApplicationProjectedRelation, WorthQueryApplicationProjectionNode,
    WorthQueryApplicationWorkingProjectionTree,
};
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationResultBufferReservation;

mod bounded_ordering;
mod relation_attachment;
mod relation_distribution;
mod relation_target_filter;
mod source_footprint;
mod work;

use bounded_ordering::order_collection;
use relation_attachment::{advance_omitted_relation, attach_relation};
use work::ResultTreeWork;

pub(super) struct MaterializedApplicationResultTree {
    pub(super) rows: WorthQueryApplicationDisclosedProjectionTree,
    pub(super) source_footprints:
        Vec<crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedSourceFootprint>,
    pub(super) projected_records: usize,
    pub(super) projected_fields: usize,
    pub(super) adjacency_lists_read: usize,
    pub(super) relation_records_examined: usize,
    pub(super) ordering_comparisons: usize,
    pub(super) ordered_index_entries_examined: usize,
    pub(super) relation_predicate_work_units: usize,
    pub(super) work_units: usize,
    pub(super) continuation: Option<OrderedCollectionProgress>,
}

pub(super) struct OrderedCollectionWindow {
    pub(super) snapshot: worth_relational::facade::snapshots::SnapshotHandle,
    pub(super) collection_path: String,
    pub(super) index_id: DerivedIndexId,
    pub(super) expected_generation: Option<DerivedIndexGenerationId>,
    pub(super) after: Option<RelatedEntityOrderingBoundary>,
    pub(super) page_width: usize,
}

pub(super) struct TargetedCollectionChild {
    pub(super) collection_path: String,
    pub(super) child_entity_id: EntityId,
}

pub(super) enum ResultTreeCollectionSelection {
    Complete,
    Ordered(OrderedCollectionWindow),
    Targeted(TargetedCollectionChild),
}

pub(super) struct OrderedCollectionProgress {
    pub(super) generation_id: DerivedIndexGenerationId,
    pub(super) next_boundary: Option<RelatedEntityOrderingBoundary>,
    pub(super) has_more: bool,
}

pub(super) fn materialize_result_tree(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    root_ids: &[EntityId],
    maximum_work: usize,
    collection_selection: ResultTreeCollectionSelection,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<MaterializedApplicationResultTree, WorthQueryApplicationReadExecutionDenial> {
    let projection = runtime
        .read_truth()
        .project_snapshot(snapshot)
        .ok_or_else(|| projection_denial(contract.root_entity()))?;
    let mut work = ResultTreeWork::new(maximum_work);
    let mut collection_selection = ActiveResultTreeCollectionSelection::new(collection_selection);
    let mut rows = project_nodes(
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
        result_buffer,
    )?;
    order_collection(contract, governance, "root", &mut rows, &mut work)?;
    let source_footprints = source_footprint::collect_source_footprints(
        &projection,
        graph,
        contract,
        governance,
        &rows,
        &mut work,
        result_buffer,
    )?;
    let rows = WorthQueryApplicationWorkingProjectionTree::new(rows)
        .into_disclosed(governance, result_buffer);
    Ok(MaterializedApplicationResultTree {
        rows,
        source_footprints,
        projected_records: work.projected_records,
        projected_fields: work.projected_fields,
        adjacency_lists_read: work.adjacency_lists_read,
        relation_records_examined: work.relation_records_examined,
        ordering_comparisons: work.ordering_comparisons,
        ordered_index_entries_examined: work.ordered_index_entries_examined,
        relation_predicate_work_units: work.relation_predicate_work_units,
        work_units: work.work_units,
        continuation: collection_selection.into_progress(),
    })
}

pub(super) enum ActiveResultTreeCollectionSelection {
    Complete,
    Ordered(ActiveOrderedCollectionWindow),
    Targeted(TargetedCollectionChild),
}

impl ActiveResultTreeCollectionSelection {
    fn new(selection: ResultTreeCollectionSelection) -> Self {
        match selection {
            ResultTreeCollectionSelection::Complete => Self::Complete,
            ResultTreeCollectionSelection::Ordered(window) => {
                Self::Ordered(ActiveOrderedCollectionWindow::new(window))
            }
            ResultTreeCollectionSelection::Targeted(target) => Self::Targeted(target),
        }
    }

    fn into_progress(self) -> Option<OrderedCollectionProgress> {
        match self {
            Self::Ordered(window) => window.into_progress(),
            Self::Complete | Self::Targeted(_) => None,
        }
    }
}

pub(super) struct ActiveOrderedCollectionWindow {
    request: Option<OrderedCollectionWindow>,
    progress: Option<OrderedCollectionProgress>,
}

impl ActiveOrderedCollectionWindow {
    fn new(request: OrderedCollectionWindow) -> Self {
        Self {
            request: Some(request),
            progress: None,
        }
    }

    fn into_progress(self) -> Option<OrderedCollectionProgress> {
        self.progress
    }
}

#[allow(clippy::too_many_arguments)]
fn project_nodes(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    projection: &worth_relational::facade::runtime::VisibilityProjectionView<'_>,
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
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<Vec<WorthQueryApplicationProjectionNode>, WorthQueryApplicationReadExecutionDenial> {
    let fields = direct_projections(contract, governance, result_path);
    let relations = direct_relations(contract, result_path);
    let source_dependencies_complete = contract
        .projections()
        .iter()
        .filter(|projection| projection.parent_path() == result_path)
        .all(|projection| governance.is_disclosed(projection.slot_key_identity().as_ref()))
        && relations
            .iter()
            .all(|relation| governance.is_disclosed(relation.slot_key_identity().as_ref()));
    work.charge_projection(entity_ids.len(), fields.len(), result_path)?;
    let mut nodes = allocate_claimed_result_vector::<WorthQueryApplicationProjectionNode>(
        result_buffer,
        entity_ids.len(),
        result_path,
    )?;
    let scope = projection_scope(&fields);
    let kind = graph
        .entity_kind(entity_name)
        .ok_or_else(|| projection_denial(entity_name))?;
    for entity_id in entity_ids {
        let projected_fields = projection
            .entity_record_with_projection_scope(*entity_id, scope.clone(), |record| {
                (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live).then(
                    || {
                        let projected = allocate_claimed_result_vector::<
                            WorthQueryApplicationProjectedField,
                        >(
                            result_buffer, fields.len(), result_path
                        )?;
                        project_fields(record, &fields, projected, result_buffer)
                    },
                )
            })
            .ok_or_else(|| projection_denial(result_path))??;
        let projected_relations = allocate_claimed_result_vector::<
            WorthQueryApplicationProjectedRelation,
        >(result_buffer, relations.len(), result_path)?;
        nodes.push(WorthQueryApplicationProjectionNode::new(
            *entity_id,
            source_path.clone(),
            source_dependencies_complete,
            projected_fields,
            projected_relations,
        ));
    }
    for relation in relations {
        if !governance.is_disclosed(relation.slot_key_identity().as_ref()) {
            advance_omitted_relation(runtime, graph, relation, &nodes, work, collection_selection)?;
            continue;
        }
        attach_relation(
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
            result_buffer,
        )?;
    }
    Ok(nodes)
}

fn project_fields(
    record: worth_relational::facade::runtime::EntityProjectionRecord<'_>,
    fields: &[&WorthQueryInstalledGraphProjection],
    mut projected: Vec<WorthQueryApplicationProjectedField>,
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
) -> Result<Vec<WorthQueryApplicationProjectedField>, WorthQueryApplicationReadExecutionDenial> {
    for projection in fields {
        let Some(value) =
            record.aspect_field_value(projection.aspect_key(), projection.field_key())
        else {
            if projection.presence() == ApplicationFieldPresence::Optional {
                continue;
            }
            return Err(projection_denial(projection.result_path()));
        };
        if value.value_family() != projection.scalar_family() {
            return Err(projection_denial(projection.result_path()));
        }
        result_buffer
            .claim(value.owned_allocation_capacity_bytes())
            .map_err(|()| result_buffer_denial(projection.result_path()))?;
        projected.push(WorthQueryApplicationProjectedField::new(
            projection,
            value.clone(),
        ));
    }
    Ok(projected)
}

pub(super) fn allocate_claimed_result_vector<T>(
    result_buffer: &mut WorthQueryApplicationResultBufferReservation,
    capacity: usize,
    subject: &str,
) -> Result<Vec<T>, WorthQueryApplicationReadExecutionDenial> {
    let bytes = capacity
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| result_buffer_denial(subject))?;
    result_buffer
        .claim(bytes)
        .map_err(|()| result_buffer_denial(subject))?;
    let vector = Vec::with_capacity(capacity);
    if vector.capacity().checked_mul(std::mem::size_of::<T>()) != Some(bytes) {
        return Err(result_buffer_denial(subject));
    }
    Ok(vector)
}

fn direct_projections<'a>(
    contract: &'a WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parent: &str,
) -> Vec<&'a WorthQueryInstalledGraphProjection> {
    contract
        .projections()
        .iter()
        .filter(|projection| {
            let field = (projection.entity(), projection.aspect(), projection.field());
            let disclosed = governance
                .admit_disclosed_projection(
                    projection.slot_key_identity().as_ref(),
                    field,
                    projection.field_key(),
                )
                .is_some();
            let internal_ordering = contract.ordering().iter().any(|ordering| {
                ordering.collection_path() == parent
                    && ordering.field() == field
                    && governance
                        .admit_internal_projection(
                            ordering.field(),
                            ordering.field_key(),
                            worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence::Ordering,
                        )
                        .is_some()
            });
            projection.parent_path() == parent
                && (disclosed || internal_ordering)
        })
        .collect()
}

fn direct_relations<'a>(
    contract: &'a WorthQueryInstalledGraphReadContract,
    parent: &str,
) -> Vec<&'a WorthQueryInstalledGraphRelation> {
    contract
        .relations()
        .iter()
        .filter(|relation| relation.parent_path() == parent)
        .collect()
}

fn projection_scope(fields: &[&WorthQueryInstalledGraphProjection]) -> ProjectionAspectScope {
    let mut keys = BTreeMap::new();
    for field in fields {
        keys.entry(field.aspect_key().clone())
            .or_insert_with(BTreeSet::new)
            .insert(field.field_key().clone());
    }
    ProjectionAspectScope::from_requirements(
        keys.into_iter()
            .map(|(aspect, fields)| ProjectionAspectRequirement::fields(aspect, fields)),
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

//! Chooses one keyset window of a selected root set before materialization,
//! and optionally materializes its roots incrementally under an exact
//! projected-byte ceiling so oversized rows refuse before decoding `QueryResult`.
use super::bounded_ordering::{admitted_ordering, boundary_after, follows_boundary};
use super::*;
use crate::domain_computation::primary_graph::application_query::root_window::{
    compute_resume_anchor_proof, RootWindowAfter, WorthQueryApplicationRootWindow,
    WorthQueryApplicationRootWindowBoundary,
};

/// The roots of one window, their per-root boundaries, and the work spent choosing them.
pub struct RootWindowSelection {
    pub roots: Vec<EntityId>,
    pub boundaries: Vec<WorthQueryApplicationRootWindowBoundary>,
    pub has_more_after_width: bool,
    pub started_at_beginning: bool,
    pub resume_anchor_proof: Option<[u8; 32]>,
    pub reads: WorthQueryInvariantProjectionWork,
    pub ordering_comparisons: usize,
    pub work_units: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn select_root_window(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    request_binding_digest: &[u8; 32],
    root_ids: &[EntityId],
    window: &WorthQueryApplicationRootWindow,
    interruption: super::super::ReadInterruption<'_>,
    spent: Option<&OneShotReadWorkObservation<'_>>,
) -> Result<RootWindowSelection, WorthQueryApplicationReadExecutionDenial> {
    let read = ResultTreeRead::committed(runtime, snapshot)
        .ok_or_else(|| projection_denial(contract.root_entity()))?;
    let mut work = ResultTreeWork::new(interruption, spent, *execution, None);
    let mut keys = ordering_keys(
        execution, &read, graph, contract, governance, root_ids, &mut work,
    )?;
    order_collection(contract, governance, "root", &mut keys, &mut work)?;
    let ordering = admitted_ordering(contract, governance, "root")?;
    if ordering.is_empty() && keys.len() >= 2 {
        // No declared ordering: the total order is root identity alone, which
        // the boundary search below relies on, whatever order selection used.
        super::fallible_heap::sort_by(&mut keys, |left, right| {
            work.charge_ordering_comparison("result-identity-tie-break")?;
            Ok(left.entity_id().cmp(&right.entity_id()))
        })?;
    }
    let (start, started_at_beginning, resume_anchor_proof) = match window.after_position() {
        Some(RootWindowAfter::Boundary(boundary)) => (
            first_following(
                &keys,
                |row, work| follows_boundary(row, boundary, &ordering, work),
                &mut work,
            )?,
            false,
            None,
        ),
        Some(RootWindowAfter::RootEntity { root, anchor_nonce }) => {
            let anchor = keys
                .iter()
                .find(|row| row.entity_id() == *root)
                .ok_or_else(|| {
                    read_execution_denial(
                        WorthQueryApplicationReadExecutionDenialKind::ContinuationBoundaryRejected,
                        contract.root_entity(),
                    )
                })?;
            let boundary = boundary_after(anchor, &ordering)?;
            let proof = compute_resume_anchor_proof(
                anchor_nonce,
                *root,
                snapshot.version_id(),
                request_binding_digest,
            );
            (
                first_following(
                    &keys,
                    |row, work| follows_boundary(row, &boundary, &ordering, work),
                    &mut work,
                )?,
                false,
                Some(proof),
            )
        }
        None => (0, true, None),
    };
    let end = start.saturating_add(window.width().get()).min(keys.len());
    let page = keys.get(start..end).unwrap_or_default();
    let has_more_after_width = end < keys.len();
    let mut boundaries = Vec::with_capacity(page.len());
    for row in page {
        boundaries.push(boundary_after(row, &ordering)?);
    }
    let roots: Vec<EntityId> = page
        .iter()
        .map(WorthQueryApplicationProjectionNode::entity_id)
        .collect();
    Ok(RootWindowSelection {
        roots,
        boundaries,
        has_more_after_width,
        started_at_beginning,
        resume_anchor_proof,
        reads: std::mem::take(&mut work.reads),
        ordering_comparisons: work.ordering_comparisons,
        work_units: work.work_units,
    })
}

/// Materializes `window_selection.roots` one root at a time while enforcing
/// `max_projected_bytes` against actual disclosed node byte footprints (including
/// heap strings, bytes, and nested relations), stopping BEFORE materializing
/// remaining roots once the budget would be exceeded and refusing even the first
/// row when it alone exceeds `max_projected_bytes`.
#[allow(clippy::too_many_arguments)]
pub fn materialize_window_bounded_tree(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    parameters: &WorthQueryAdmittedApplicationQueryParameters,
    root_ids: &[EntityId],
    selected_predicate_sources: &[crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedFieldRevision],
    root_path_source: Option<&BTreeMap<EntityId, Arc<crate::domain_computation::primary_graph::application_query::observed_source::WorthQueryObservedRootSelection>>>,
    max_projected_bytes: usize,
    window: &WorthQueryApplicationRootWindow,
    interruption: super::super::ReadInterruption<'_>,
    spent: Option<&OneShotReadWorkObservation<'_>>,
    material_source: Option<
        &Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>,
    >,
) -> Result<
    (MaterializedApplicationResultTree, Vec<usize>, bool),
    WorthQueryApplicationReadExecutionDenial,
> {
    let read = ResultTreeRead::committed(runtime, snapshot)
        .ok_or_else(|| projection_denial(contract.root_entity()))?;
    let mut work = ResultTreeWork::new(interruption, spent, *execution, material_source);
    work.checkpoint("root")?;
    let mut accepted_nodes = Vec::new();
    let mut row_bytes_list = Vec::new();
    let mut remaining_budget_bytes = max_projected_bytes;
    let mut truncated_by_bytes = false;

    for &root_id in root_ids {
        let mut selection =
            ActiveResultTreeCollectionSelection::new(ResultTreeCollectionSelection::Complete);
        work.set_window_budget(remaining_budget_bytes);
        let projected = project_nodes(
            execution,
            &read,
            &read.projection,
            graph,
            contract,
            governance,
            parameters,
            None,
            "root",
            contract.root_entity(),
            &[root_id],
            &mut work,
            &mut selection,
        );
        let charged_bytes = work.window_charged_bytes();
        let exceeded_requested = work.window_budget_exceeded();
        work.clear_window_budget();
        let mut single = match projected {
            Ok(nodes) => nodes,
            Err(denial) => {
                if let Some(requested_bytes) = exceeded_requested {
                    if accepted_nodes.is_empty() {
                        window.report_oversized_row(requested_bytes);
                        return Err(denial);
                    }
                    truncated_by_bytes = true;
                    break;
                }
                return Err(denial);
            }
        };
        let Some(node) = single.pop() else {
            return Err(projection_denial(contract.root_entity()));
        };
        let row_bytes = disclosed_node_bytes(&node, governance).max(charged_bytes);
        if row_bytes > remaining_budget_bytes {
            if accepted_nodes.is_empty() {
                window.report_oversized_row(row_bytes);
                return Err(read_execution_denial(
                    WorthQueryApplicationReadExecutionDenialKind::ResultBufferLimitExceeded,
                    contract.root_entity(),
                ));
            }
            truncated_by_bytes = true;
            break;
        }
        remaining_budget_bytes = remaining_budget_bytes.saturating_sub(row_bytes);
        row_bytes_list.push(row_bytes);
        accepted_nodes.push(node);
    }

    let source_footprints = source_footprint::collect_source_footprints(
        execution,
        &read.projection,
        graph,
        contract,
        governance,
        &accepted_nodes,
        selected_predicate_sources,
        root_path_source,
        &mut work,
        material_source,
    )?;
    let rows =
        WorthQueryApplicationWorkingProjectionTree::new(accepted_nodes).into_disclosed(governance);
    Ok((
        MaterializedApplicationResultTree {
            rows,
            source_footprints,
            projected_records: work.projected_records,
            projected_fields: work.projected_fields,
            reads: work.reads,
            ordering_comparisons: work.ordering_comparisons,
            ordered_index_entries_examined: work.ordered_index_entries_examined,
            relation_predicate_work_units: work.relation_predicate_work_units,
            work_units: work.work_units,
            continuation: None,
        },
        row_bytes_list,
        truncated_by_bytes,
    ))
}

/// Computes the actual projected byte footprint of one node (disclosed fields +
/// disclosed nested relations, including heap string/bytes payloads).
pub fn disclosed_node_bytes(
    node: &WorthQueryApplicationProjectionNode,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
) -> usize {
    node.disclosed_retained_bytes(governance)
}

/// Each candidate root with only its admitted root ordering fields.
fn ordering_keys(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read: &ResultTreeRead<'_>,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    root_ids: &[EntityId],
    work: &mut ResultTreeWork,
) -> Result<Vec<WorthQueryApplicationProjectionNode>, WorthQueryApplicationReadExecutionDenial> {
    let fields = direct_projections(contract, governance, "root", work)?
        .into_iter()
        .filter(|field| {
            contract.ordering().iter().any(|term| {
                term.collection_path() == "root" && term.slot_type() == field.slot_type()
            })
        })
        .collect::<Vec<_>>();
    work.charge_projection(root_ids.len(), fields.len(), "root")?;
    let scope = projection_scope(&fields);
    let kind = graph
        .entity_kind(contract.root_entity())
        .ok_or_else(|| projection_denial(contract.root_entity()))?;
    let mut keys = allocate_result_vector(root_ids.len(), "root")?;
    for entity_id in root_ids {
        work.checkpoint("root")?;
        let projected = read
            .projection
            .entity_record_with_projection_scope(execution, *entity_id, scope.clone(), |record| {
                work.reads.record_endpoint();
                (record.kind_id() == kind && record.lifecycle() == RecordLifecycleState::Live).then(
                    || {
                        let projected = allocate_result_vector(fields.len(), "root")?;
                        project_fields(
                            execution,
                            &read.projection,
                            record,
                            governance,
                            &fields,
                            projected,
                            work,
                        )
                    },
                )
            })
            .map_err(|cause| super::super::denial::relational_read_denial(cause, "root"))?
            .ok_or_else(|| projection_denial("root"))??;
        keys.push(WorthQueryApplicationProjectionNode::new(
            *entity_id,
            None,
            false,
            projected,
            Vec::new(),
        ));
    }
    Ok(keys)
}

/// The first index of sorted `rows` that `follows`, by binary search.
fn first_following(
    rows: &[WorthQueryApplicationProjectionNode],
    mut follows: impl FnMut(
        &WorthQueryApplicationProjectionNode,
        &mut ResultTreeWork,
    ) -> Result<bool, WorthQueryApplicationReadExecutionDenial>,
    work: &mut ResultTreeWork,
) -> Result<usize, WorthQueryApplicationReadExecutionDenial> {
    let (mut low, mut high) = (0, rows.len());
    while low < high {
        let middle = low + (high - low) / 2;
        let Some(row) = rows.get(middle) else {
            break;
        };
        if follows(row, work)? {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    Ok(low)
}

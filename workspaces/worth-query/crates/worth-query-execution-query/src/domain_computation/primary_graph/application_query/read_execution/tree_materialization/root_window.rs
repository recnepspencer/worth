//! Chooses one keyset window of a selected root set before materialization.
//!
//! Every candidate root projects only its admitted root ordering fields; the
//! keys are put in the read's total order (the same comparator as the
//! complete read's root ordering), cut strictly after the request's boundary
//! and to its width. The caller materializes only the returned roots.
use super::bounded_ordering::{admitted_ordering, boundary_after, follows_boundary};
use super::*;
use crate::domain_computation::primary_graph::application_query::root_window::WorthQueryApplicationRootWindow;

/// The roots of one window and the work spent choosing them.
pub struct RootWindowSelection {
    pub roots: Vec<EntityId>,
    pub reads: WorthQueryInvariantProjectionWork,
    pub ordering_comparisons: usize,
    pub work_units: usize,
}

pub fn select_root_window(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalPinnedRead,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    contract: &WorthQueryInstalledGraphReadContract,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
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
    if ordering.is_empty() {
        // No declared ordering: the total order is root identity alone, which
        // the boundary search below relies on, whatever order selection used.
        keys.sort_unstable_by_key(WorthQueryApplicationProjectionNode::entity_id);
    }
    let start = match window.boundary() {
        Some(boundary) => first_following(
            &keys,
            |row, work| follows_boundary(row, boundary, &ordering, work),
            &mut work,
        )?,
        None => 0,
    };
    let end = start.saturating_add(window.width().get()).min(keys.len());
    let page = keys.get(start..end).unwrap_or_default();
    let next = match page.last() {
        Some(last) if end < keys.len() => Some(boundary_after(last, &ordering)?),
        _ => None,
    };
    let roots = page
        .iter()
        .map(WorthQueryApplicationProjectionNode::entity_id)
        .collect();
    window.report(next);
    Ok(RootWindowSelection {
        roots,
        reads: std::mem::take(&mut work.reads),
        ordering_comparisons: work.ordering_comparisons,
        work_units: work.work_units,
    })
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

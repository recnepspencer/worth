//! Bounded narrowing of index-selected root candidates.
//!
//! Root selection first seeks candidates structurally: the exact scope root,
//! its `where_equal` guard, or an installed root-path union. This stage then
//! narrows those candidates in canonical identity order, first by root-entity
//! `where_equal` predicates over a path union, then by the residual where
//! filter. Every examined candidate's predicate and filter fields join the
//! result-set evidence, including candidates the stage excludes, so a change
//! to a filter-only field invalidates the result. Cost is
//! O(candidates × (filter fields + expression work)); each step is charged to
//! the request meter and every slice boundary is a cancellation safe point.
use super::{
    evidence::RootPathSourceBuilder, BoundedRootSelection, OneShotReadWorkObservation, ReadPlan,
    RootSelectionWork, WorthQueryApplicationReadExecutionDenial,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::runtime::{RelationalPinnedRead, VisibilityProjectionView};

mod equality;
mod evidence;
mod related;
mod residual;

/// The subject every residual-filter safe point and charge reports.
pub const WHERE_FILTER_SUBJECT: &str = "where_filter";

/// The read projection and ports one narrowing pass runs against.
pub struct NarrowingRead<'read, 'plan, 'exec, 'env, ReadIdentity> {
    pub runtime: &'read RelationalPinnedRead<ReadIdentity>,
    pub projection: &'read VisibilityProjectionView,
    pub execution: &'read worth_execution::ExecutionRequest<'exec, 'env>,
    pub graph: &'read crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    pub plan: &'read ReadPlan<'plan>,
}

/// Whether an installed shape needs narrowing at all. Unfiltered exact-root
/// and unguarded path selections return their selection untouched.
fn narrows(plan: &ReadPlan<'_>) -> bool {
    path_equality(plan)
        || plan.contract.where_filter().is_some()
        || !plan.contract.related_predicates().is_empty()
}

fn path_equality(plan: &ReadPlan<'_>) -> bool {
    !plan.contract.root_paths().is_empty() && !plan.contract.predicates().is_empty()
}

pub fn narrow_selected_roots<ReadIdentity>(
    read: &NarrowingRead<'_, '_, '_, '_, ReadIdentity>,
    mut selection: BoundedRootSelection,
    spent: Option<&OneShotReadWorkObservation<'_>>,
    interruption: super::super::ReadInterruption<'_>,
) -> Result<BoundedRootSelection, WorthQueryApplicationReadExecutionDenial> {
    if !narrows(read.plan) {
        return Ok(selection);
    }
    let mut work = RootSelectionWork::new(interruption, spent);
    // The stage continues the root-selection meter rather than replacing it.
    work.work_units = selection.work_units;
    let mut evidence =
        RootPathSourceBuilder::for_material_source(read.execution, read.plan.material_source)?;
    let mut candidates = std::mem::take(&mut selection.candidates);
    if path_equality(read.plan) {
        candidates = equality::narrow(read, candidates, &mut evidence, &mut work)?;
    }
    if let Some(filter) = read.plan.contract.where_filter() {
        candidates = residual::narrow(read, filter, candidates, &mut evidence, &mut work)?;
    }
    if !read.plan.contract.related_predicates().is_empty() {
        candidates = related::narrow(read, candidates, &mut evidence, &mut work)?;
    }
    let merge_subject = if read.plan.contract.related_predicates().is_empty()
        && read.plan.contract.where_filter().is_some()
    {
        WHERE_FILTER_SUBJECT
    } else {
        "narrowed selection evidence"
    };
    selection.result_set_source = evidence::merge(
        read.execution,
        selection.result_set_source.take(),
        evidence,
        merge_subject,
    )?;
    retain_root_sources(&mut selection, &candidates);
    selection.candidates = candidates;
    selection.examined_candidates = selection
        .examined_candidates
        .saturating_add(work.predicate_records_examined);
    selection.predicate_work_units = selection
        .predicate_work_units
        .saturating_add(work.predicate_work_units);
    selection.reads = selection.reads.combined(std::mem::take(&mut work.reads));
    selection.work_units = work.work_units;
    Ok(selection)
}

/// Per-root path evidence remains only for retained rows; excluded rows are
/// covered by the result-set evidence.
fn retain_root_sources(selection: &mut BoundedRootSelection, retained: &[EntityId]) {
    if let Some(sources) = selection.root_path_source.as_mut() {
        sources.retain(|root, _| retained.binary_search(root).is_ok());
    }
}

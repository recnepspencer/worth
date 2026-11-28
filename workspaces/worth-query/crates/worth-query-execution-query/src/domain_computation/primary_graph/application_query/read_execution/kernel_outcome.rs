use crate::domain_computation::primary_graph::value_decode::ValueDecodeState;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::{RawNonLiveKernelOutcome, RawOneShotRows};
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationProjectionRow,
};

pub struct ProjectedNonLiveKernelOutcome<QueryResult> {
    pub rows: Vec<QueryResult>,
    pub receipt: NonLiveKernelReceiptEvidence,
}

pub struct NonLiveKernelReceiptEvidence {
    pub read: RawOneShotRows,
    pub result_count: usize,
    pub truncation_count: usize,
}

/// Native root identities from the same disclosed tree used by ordinary projection.
/// No application projection or retained-history callback runs here.
pub fn project_non_live_kernel_roots(
    kernel: RawNonLiveKernelOutcome,
) -> ProjectedNonLiveKernelOutcome<worth_relational::facade::identity::EntityId> {
    let RawNonLiveKernelOutcome { mut raw, .. } = kernel;
    let rows: Vec<_> = raw.rows.iter().map(|node| node.entity_id()).collect();
    raw.rows.clear();
    let result_count = rows.len();
    let truncation_count = usize::from(raw.has_more);
    ProjectedNonLiveKernelOutcome {
        rows,
        receipt: NonLiveKernelReceiptEvidence {
            read: raw,
            result_count,
            truncation_count,
        },
    }
}

impl<QueryResult> ProjectedNonLiveKernelOutcome<QueryResult> {
    pub fn into_parts(self) -> (Vec<QueryResult>, NonLiveKernelReceiptEvidence) {
        (self.rows, self.receipt)
    }
}

impl NonLiveKernelReceiptEvidence {
    pub fn observed_graph_read_work(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork {
        crate::domain_computation::provider_session::WorthQueryObservedGraphReadWork {
            examined_candidates: self.read.examined_candidates,
            projected_records: self.read.projected_records,
            projected_fields: self.read.projected_fields,
            relation_records_examined: self.read.reads.adjacency_edges_inspected(),
            ordering_comparisons: self.read.ordering_comparisons,
        }
    }
}

pub fn project_non_live_kernel_with_decode<Schema, Query, QueryResult, Denial>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    decode: &ValueDecodeState,
    kernel: RawNonLiveKernelOutcome,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    validate: impl FnMut() -> Result<(), Denial>,
    map_projection_denial: impl Fn(WorthQueryApplicationProjectionDenial) -> Denial,
) -> Result<ProjectedNonLiveKernelOutcome<QueryResult>, Denial>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let RawNonLiveKernelOutcome {
        mut raw,
        persistent_names,
    } = kernel;
    let max_projected_bytes = raw
        .root_window_request
        .as_ref()
        .and_then(super::super::root_window::WorthQueryApplicationRootWindow::max_projected_bytes);
    let (rows, effective_row_bytes, truncated_in_projection) =
        decode_projection_tree_budgeted::<Schema, Query, QueryResult, Denial>(
            execution,
            decode,
            &raw.rows,
            governance,
            persistent_names.as_ref(),
            &raw.row_projected_bytes,
            max_projected_bytes,
            raw.root_window_request.as_ref(),
            validate,
            map_projection_denial,
        )?;
    raw.rows.clear();
    if !raw.row_projected_bytes.is_empty() || max_projected_bytes.is_some() {
        raw.row_projected_bytes = effective_row_bytes;
    }
    if truncated_in_projection {
        let kept = rows.len();
        let kept_last_boundary = kept
            .checked_sub(1)
            .and_then(|idx| raw.window_boundaries.get(idx).cloned());
        if let Some(progress) = raw.window_progress.as_mut() {
            progress.truncate_to_projected_rows(kept, kept_last_boundary);
            if let Some(window) = raw.root_window_request.as_ref() {
                window.update_progress(progress.clone());
            }
        }
    }
    let result_count = rows.len();
    let truncation_count = usize::from(raw.has_more);
    Ok(ProjectedNonLiveKernelOutcome {
        rows,
        receipt: NonLiveKernelReceiptEvidence {
            read: raw,
            result_count,
            truncation_count,
        },
    })
}

/// One value decoder for committed and applied projection phases. It creates
/// values only; neither a Current source nor an access receipt is issued here.
pub fn decode_projection_tree<Schema, Query, QueryResult, Denial>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    decode: &ValueDecodeState,
    tree: &super::super::projection::WorthQueryApplicationDisclosedProjectionTree,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    names: Option<&worth_query_execution_naming::facade::ObservedEntityNames>,
    validate: impl FnMut() -> Result<(), Denial>,
    map_projection_denial: impl Fn(WorthQueryApplicationProjectionDenial) -> Denial,
) -> Result<Vec<QueryResult>, Denial>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let (rows, _, _) = decode_projection_tree_budgeted::<Schema, Query, QueryResult, Denial>(
        execution,
        decode,
        tree,
        governance,
        names,
        &[],
        None,
        None,
        validate,
        map_projection_denial,
    )?;
    Ok(rows)
}

#[allow(clippy::too_many_arguments)]
fn decode_projection_tree_budgeted<Schema, Query, QueryResult, Denial>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    decode: &ValueDecodeState,
    tree: &super::super::projection::WorthQueryApplicationDisclosedProjectionTree,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    names: Option<&worth_query_execution_naming::facade::ObservedEntityNames>,
    tree_row_bytes: &[usize],
    max_projected_bytes: Option<usize>,
    root_window: Option<&super::super::root_window::WorthQueryApplicationRootWindow>,
    mut validate: impl FnMut() -> Result<(), Denial>,
    map_projection_denial: impl Fn(WorthQueryApplicationProjectionDenial) -> Denial,
) -> Result<(Vec<QueryResult>, Vec<usize>, bool), Denial>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let projected = decode.run(execution, |session| {
        let mut rows = Vec::with_capacity(tree.raw_rows().len());
        let mut effective_bytes = Vec::with_capacity(tree.raw_rows().len());
        let mut acc_bytes = 0usize;
        let mut truncated = false;
        for (idx, node) in tree.iter().enumerate() {
            validate()?;
            let value = QueryResult::project(
                &WorthQueryApplicationProjectionRow::new(node, governance, session)
                    .with_persistent_names(names),
            );
            // The native owner checks its latched refusal before accepting a
            // projector's substituted value or unrelated domain error.
            if session.stopped() {
                break;
            }
            let value = value.map_err(&map_projection_denial)?;
            let tree_bytes = tree_row_bytes.get(idx).copied().unwrap_or(0);
            let typed_bytes = value.projected_row_bytes();
            let row_bytes = tree_bytes.max(typed_bytes);
            if let Some(limit) = max_projected_bytes {
                let next_total = acc_bytes.saturating_add(row_bytes);
                if row_bytes > limit || next_total > limit {
                    if rows.is_empty() {
                        if let Some(window) = root_window {
                            window.report_oversized_row(row_bytes.max(next_total));
                        }
                        return Err(map_projection_denial(
                            WorthQueryApplicationProjectionDenial::reject("streaming-window-bytes"),
                        ));
                    }
                    truncated = true;
                    break;
                }
                acc_bytes = next_total;
            }
            effective_bytes.push(row_bytes);
            rows.push(value);
        }
        Ok((rows, effective_bytes, truncated))
    });
    let map_decode_denial = |cause| {
        map_projection_denial(WorthQueryApplicationProjectionDenial::new(
            WorthQueryApplicationProjectionDenialKind::ValueDecode(cause),
            "value-decode",
        ))
    };
    // A projector may cancel the original request and return a domain error.
    // Poll before accepting either result; a produced value from the native
    // checkpoint scope does not by itself prove that request remained live.
    decode.check(execution).map_err(map_decode_denial)?;
    let projected = projected.map_err(map_decode_denial)?;
    projected
}

/// Internal completion carrier only: () is not the application's result binding.
pub fn project_non_live_kernel_source(
    kernel: RawNonLiveKernelOutcome,
    work: impl FnOnce(usize) -> Result<(), super::super::WorthQueryApplicationOneShotDenial>,
) -> Result<
    (
        ProjectedNonLiveKernelOutcome<()>,
        super::super::projection::WorthQueryApplicationDisclosedProjectionTree,
    ),
    super::super::WorthQueryApplicationOneShotDenial,
> {
    let RawNonLiveKernelOutcome { mut raw, .. } = kernel;
    let result_count = raw.rows.raw_rows().len();
    work(result_count)?;
    let rows = vec![(); result_count];
    let tree = raw.rows.take();
    let truncation_count = usize::from(raw.has_more);
    Ok((
        ProjectedNonLiveKernelOutcome {
            rows,
            receipt: NonLiveKernelReceiptEvidence {
                read: raw,
                result_count,
                truncation_count,
            },
        },
        tree,
    ))
}

/// A single source row carries the complete selection that admitted that row.
pub fn complete_single_source_selection(
    rows: &mut [super::super::observed_source::WorthQueryObservedSourceFootprint],
    selection: &std::sync::Arc<super::super::observed_source::WorthQueryObservedRootSelection>,
) {
    if let [row] = rows {
        row.root_selection = Some(std::sync::Arc::clone(selection));
    }
}

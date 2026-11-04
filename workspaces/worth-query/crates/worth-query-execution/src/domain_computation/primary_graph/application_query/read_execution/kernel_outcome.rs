use crate::domain_computation::primary_graph::value_decode::ValueDecodeState;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

use super::{RawNonLiveKernelOutcome, RawOneShotRows};
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionDenialKind, WorthQueryApplicationProjectionRow,
};

pub(in crate::domain_computation::primary_graph::application_query) struct ProjectedNonLiveKernelOutcome<
    QueryResult,
> {
    pub(super) rows: Vec<QueryResult>,
    pub(super) receipt: NonLiveKernelReceiptEvidence,
}

pub(in crate::domain_computation::primary_graph::application_query) struct NonLiveKernelReceiptEvidence
{
    pub(in crate::domain_computation::primary_graph::application_query) read: RawOneShotRows,
    pub(in crate::domain_computation::primary_graph::application_query) result_count: usize,
    pub(in crate::domain_computation::primary_graph::application_query) truncation_count: usize,
}

/// Native root identities from the same disclosed tree used by ordinary projection.
/// No application projection or retained-history callback runs here.
pub(in crate::domain_computation::primary_graph::application_query) fn project_non_live_kernel_roots(
    kernel: RawNonLiveKernelOutcome,
) -> ProjectedNonLiveKernelOutcome<worth_relational::facade::identity::EntityId> {
    let RawNonLiveKernelOutcome { mut raw } = kernel;
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
    pub(in crate::domain_computation::primary_graph::application_query) fn into_parts(
        self,
    ) -> (Vec<QueryResult>, NonLiveKernelReceiptEvidence) {
        (self.rows, self.receipt)
    }
}

impl NonLiveKernelReceiptEvidence {
    pub(in crate::domain_computation::primary_graph::application_query) fn observed_graph_read_work(
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

pub(in crate::domain_computation::primary_graph::application_query) fn project_non_live_kernel_with_decode<
    Schema,
    Query,
    QueryResult,
    Denial,
>(
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
    let RawNonLiveKernelOutcome { mut raw } = kernel;
    let rows = decode_projection_tree::<Schema, Query, QueryResult, Denial>(
        execution,
        decode,
        &raw.rows,
        governance,
        validate,
        map_projection_denial,
    )?;
    raw.rows.clear();
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
pub(super) fn decode_projection_tree<Schema, Query, QueryResult, Denial>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    decode: &ValueDecodeState,
    tree: &super::super::projection::WorthQueryApplicationDisclosedProjectionTree,
    governance: &crate::domain_computation::primary_graph::application_query::disclosure::WorthQueryApplicationQueryGovernance,
    mut validate: impl FnMut() -> Result<(), Denial>,
    map_projection_denial: impl Fn(WorthQueryApplicationProjectionDenial) -> Denial,
) -> Result<Vec<QueryResult>, Denial>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let projected = decode.run(execution, |session| {
        let mut rows = Vec::with_capacity(tree.raw_rows().len());
        for node in tree.iter() {
            validate()?;
            let value = QueryResult::project(&WorthQueryApplicationProjectionRow::new(
                node, governance, session,
            ));
            // The native owner checks its latched refusal before accepting a
            // projector's substituted value or unrelated domain error.
            if session.stopped() {
                break;
            }
            rows.push(value.map_err(&map_projection_denial)?);
        }
        Ok(rows)
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
    let rows = projected?;
    Ok(rows)
}

/// Internal completion carrier only: () is not the application's result binding.
pub(in crate::domain_computation::primary_graph::application_query) fn project_non_live_kernel_source(
    kernel: RawNonLiveKernelOutcome,
    work: impl FnOnce(usize) -> Result<(), super::super::WorthQueryApplicationOneShotDenial>,
) -> Result<
    (
        ProjectedNonLiveKernelOutcome<()>,
        super::super::projection::WorthQueryApplicationDisclosedProjectionTree,
    ),
    super::super::WorthQueryApplicationOneShotDenial,
> {
    let RawNonLiveKernelOutcome { mut raw } = kernel;
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
pub(in crate::domain_computation::primary_graph::application_query) fn complete_single_source_selection(
    rows: &mut [super::super::observed_source::WorthQueryObservedSourceFootprint],
    selection: &std::sync::Arc<super::super::observed_source::WorthQueryObservedRootSelection>,
) {
    if let [row] = rows {
        row.root_selection = Some(std::sync::Arc::clone(selection));
    }
}

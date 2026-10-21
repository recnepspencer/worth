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
            relation_records_examined: self.read.relation_records_examined,
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
    mut validate: impl FnMut() -> Result<(), Denial>,
    map_projection_denial: impl Fn(WorthQueryApplicationProjectionDenial) -> Denial,
) -> Result<ProjectedNonLiveKernelOutcome<QueryResult>, Denial>
where
    Schema: ApplicationSchema,
    QueryResult: WorthQueryApplicationProjection<Schema, Query>,
{
    let RawNonLiveKernelOutcome { mut raw } = kernel;
    let projected = decode.run(execution, |session| {
        let mut rows = Vec::with_capacity(raw.rows.raw_rows().len());
        for node in raw.rows.iter() {
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

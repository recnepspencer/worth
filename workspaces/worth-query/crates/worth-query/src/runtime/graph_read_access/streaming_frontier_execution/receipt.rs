use super::{
    WorthQueryGraphReadFrontierCursor, WorthQueryGraphReadStreamingCounters,
    WorthQueryGraphReadStreamingCursorSession, WorthQueryGraphReadStreamingPageReceipt,
};
use crate::identity::hash_parts;
use crate::runtime::WorthQueryAdmittedGraphReadAccessPlan;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadStreamingReceipt {
    digest: String,
    streaming_plan_digest: String,
    snapshot_identity_digest: String,
    convergence_result_digest: String,
    page_receipts: Vec<WorthQueryGraphReadStreamingPageReceipt>,
    counters: WorthQueryGraphReadStreamingCounters,
}

impl WorthQueryGraphReadStreamingReceipt {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn streaming_plan_digest(&self) -> &str {
        &self.streaming_plan_digest
    }

    pub fn snapshot_identity_digest(&self) -> &str {
        &self.snapshot_identity_digest
    }

    pub fn convergence_result_digest(&self) -> &str {
        &self.convergence_result_digest
    }

    pub fn page_receipts(&self) -> &[WorthQueryGraphReadStreamingPageReceipt] {
        &self.page_receipts
    }

    pub fn first_page_receipt(&self) -> Option<&WorthQueryGraphReadStreamingPageReceipt> {
        self.page_receipts.first()
    }

    pub fn last_page_receipt(&self) -> Option<&WorthQueryGraphReadStreamingPageReceipt> {
        self.page_receipts.last()
    }

    pub fn counters(&self) -> &WorthQueryGraphReadStreamingCounters {
        &self.counters
    }

    pub fn open_cursor_session(&self) -> WorthQueryGraphReadStreamingCursorSession {
        WorthQueryGraphReadStreamingCursorSession::from_receipt(self)
    }

    pub(crate) fn from_plan_and_result(
        plan: &WorthQueryAdmittedGraphReadAccessPlan,
        snapshot_identity_digest: impl Into<String>,
        convergence_result_digest: impl Into<String>,
        emitted_row_count: usize,
    ) -> Option<Self> {
        let streaming_plan = plan.streaming_plan()?;
        let snapshot_identity_digest = snapshot_identity_digest.into();
        let convergence_result_digest = convergence_result_digest.into();
        let page_width = streaming_plan.page_budget().max_page_width().max(1);
        let emitted_page_count = emitted_row_count.div_ceil(page_width).max(1);
        let page_count = emitted_page_count;
        let mut page_receipts = Vec::with_capacity(page_count);

        for page_ordinal in 0..page_count {
            let remaining = emitted_row_count.saturating_sub(page_ordinal * page_width);
            let page_row_count = remaining.min(page_width);
            let page = WorthQueryGraphReadStreamingPageReceipt::new(
                streaming_plan.digest(),
                snapshot_identity_digest.clone(),
                page_ordinal,
                page_row_count,
                planned_resident_frontier_for_page(
                    page_row_count,
                    streaming_plan.page_budget().max_resident_frontier(),
                ),
                planned_resident_visited_for_page(
                    page_row_count,
                    streaming_plan.page_budget().max_resident_visited(),
                ),
            );
            let page = if page_ordinal + 1 < page_count {
                let cursor = WorthQueryGraphReadFrontierCursor::new(
                    streaming_plan.digest(),
                    snapshot_identity_digest.clone(),
                    page_ordinal + 1,
                    page.digest(),
                    hash_parts(&[
                        "worth_query_graph_read_frontier_continuation_v1".to_string(),
                        format!("streaming_plan:{}", streaming_plan.digest()),
                        format!("page_ordinal:{}", page_ordinal + 1),
                        format!("result:{convergence_result_digest}"),
                    ]),
                    hash_parts(&[
                        "worth_query_graph_read_streaming_visited_set_v1".to_string(),
                        format!("streaming_plan:{}", streaming_plan.digest()),
                        format!("page_ordinal:{page_ordinal}"),
                        format!("snapshot_identity:{snapshot_identity_digest}"),
                    ]),
                );
                page.with_next_cursor(cursor)
            } else {
                page
            };
            page_receipts.push(page);
        }

        let max_resident_frontier_observed = page_receipts
            .iter()
            .map(WorthQueryGraphReadStreamingPageReceipt::max_resident_frontier_observed)
            .max()
            .unwrap_or(0);
        let max_resident_visited_observed = page_receipts
            .iter()
            .map(WorthQueryGraphReadStreamingPageReceipt::max_resident_visited_observed)
            .max()
            .unwrap_or(0);
        let counters = WorthQueryGraphReadStreamingCounters::from_execution(
            page_count,
            emitted_row_count,
            max_resident_frontier_observed,
            max_resident_visited_observed,
        );
        let mut digest_parts = vec![
            "worth_query_graph_read_streaming_receipt_v1".to_string(),
            format!("streaming_plan:{}", streaming_plan.digest()),
            format!("snapshot_identity:{snapshot_identity_digest}"),
            format!("convergence_result:{convergence_result_digest}"),
            counters.digest_part(),
        ];
        digest_parts.extend(page_receipts.iter().map(|page| page.digest_part()));
        let digest = hash_parts(&digest_parts);
        Some(Self {
            digest,
            streaming_plan_digest: streaming_plan.digest().to_string(),
            snapshot_identity_digest,
            convergence_result_digest,
            page_receipts,
            counters,
        })
    }
}

fn planned_resident_frontier_for_page(page_row_count: usize, page_budget: usize) -> usize {
    page_row_count.max(1).min(page_budget)
}

fn planned_resident_visited_for_page(page_row_count: usize, page_budget: usize) -> usize {
    page_row_count.max(1).min(page_budget)
}

pub(crate) fn streaming_receipt_for_admitted_read_result(
    plan: &WorthQueryAdmittedGraphReadAccessPlan,
    snapshot_identity_digest: &str,
    result_digest: &str,
    emitted_row_count: usize,
) -> Option<WorthQueryGraphReadStreamingReceipt> {
    WorthQueryGraphReadStreamingReceipt::from_plan_and_result(
        plan,
        snapshot_identity_digest,
        result_digest,
        emitted_row_count,
    )
}

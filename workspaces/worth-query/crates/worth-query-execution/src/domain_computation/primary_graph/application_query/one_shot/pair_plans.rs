//! Owner-held application plans for a closed, pinned pair read.

use super::super::read_execution::prepared_pair::batch::PreparedBatchRead;
use super::super::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryBatchAdmission,
};

/// Two admitted public reads whose rooted identity is bound by Query before
/// dispatch. Application projection stays on the caller's thread. The second
/// scope must be an actual node's declared identity in the first projected tree,
/// on the same basis. Its declared scope family may also differ from the first.
/// No callback can select a new basis or issue another read on a worker.
pub struct WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C, SC = C> {
    pub(in crate::domain_computation::primary_graph::application_query) first:
        WorthQueryAdmittedApplicationQueryPlan<'a, S, FQ, FP, FR, A, I, C>,
    pub(in crate::domain_computation::primary_graph::application_query) second:
        WorthQueryAdmittedApplicationQueryPlan<'a, S, SQ, SP, SR, A, I, SC>,
    pub(in crate::domain_computation::primary_graph::application_query) batch:
        Option<WorthQueryApplicationQueryBatchAdmission>,
    pub(in crate::domain_computation::primary_graph::application_query) first_batch:
        Option<PreparedBatchRead>,
    pub(in crate::domain_computation::primary_graph::application_query) second_batch:
        Option<PreparedBatchRead>,
}

#[cfg(test)]
mod scope_tests;

impl<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C, SC>
    WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C, SC>
{
    /// Carry both issued plans to the request's owner-side preparation stage.
    /// Query checks their identities and constructs the closed worker binding
    /// only inside the reconstruction request.
    pub fn new(
        first: WorthQueryAdmittedApplicationQueryPlan<'a, S, FQ, FP, FR, A, I, C>,
        second: WorthQueryAdmittedApplicationQueryPlan<'a, S, SQ, SP, SR, A, I, SC>,
    ) -> Self {
        Self {
            first,
            second,
            batch: None,
            first_batch: None,
            second_batch: None,
        }
    }
    /// Carry the same genuine shared read loan into closed pair preparation.
    /// Issued planned items are checked against this loan before dispatch.
    pub fn in_batch(
        first: WorthQueryAdmittedApplicationQueryPlan<'a, S, FQ, FP, FR, A, I, C>,
        second: WorthQueryAdmittedApplicationQueryPlan<'a, S, SQ, SP, SR, A, I, SC>,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Self {
        Self {
            first,
            second,
            batch: Some(batch.retained_meter()),
            first_batch: None,
            second_batch: None,
        }
    }
}

/// Owner-held pins for the fixed first/second query contracts of one map.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph::application_query) struct WorthQueryPairReadPins {
    pub(in crate::domain_computation::primary_graph::application_query) first: Option<
        std::sync::Arc<crate::domain_computation::provider_session::WorthQueryPreparedQueryPin>,
    >,
    pub(in crate::domain_computation::primary_graph::application_query) second: Option<
        std::sync::Arc<crate::domain_computation::provider_session::WorthQueryPreparedQueryPin>,
    >,
}

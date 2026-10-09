//! Owner-held application plans for a closed, pinned pair read.

use super::super::read_execution::prepared_pair::batch::PreparedBatchRead;
use super::super::{
    WorthQueryAdmittedApplicationQueryPlan, WorthQueryApplicationQueryBatchAdmission,
};

/// Two admitted public reads whose rooted identity is bound by Query before
/// dispatch. Application projection stays on the caller's thread. The second
/// scope must be the first row's declared identity field on the same basis.
/// No callback can select a new basis or issue another read on a worker.
pub struct WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C> {
    pub(in crate::domain_computation::primary_graph::application_query) first:
        WorthQueryAdmittedApplicationQueryPlan<'a, S, FQ, FP, FR, A, I, C>,
    pub(in crate::domain_computation::primary_graph::application_query) second:
        WorthQueryAdmittedApplicationQueryPlan<'a, S, SQ, SP, SR, A, I, C>,
    pub(in crate::domain_computation::primary_graph::application_query) batch:
        Option<WorthQueryApplicationQueryBatchAdmission>,
    pub(in crate::domain_computation::primary_graph::application_query) first_batch:
        Option<PreparedBatchRead>,
    pub(in crate::domain_computation::primary_graph::application_query) second_batch:
        Option<PreparedBatchRead>,
}

impl<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C>
    WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C>
{
    /// Carry both issued plans to the request's owner-side preparation stage.
    /// Query checks their identities and constructs the closed worker binding
    /// only inside the reconstruction request.
    pub fn new(
        first: WorthQueryAdmittedApplicationQueryPlan<'a, S, FQ, FP, FR, A, I, C>,
        second: WorthQueryAdmittedApplicationQueryPlan<'a, S, SQ, SP, SR, A, I, C>,
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
    #[cfg(test)]
    pub(crate) fn in_batch(
        first: WorthQueryAdmittedApplicationQueryPlan<'a, S, FQ, FP, FR, A, I, C>,
        second: WorthQueryAdmittedApplicationQueryPlan<'a, S, SQ, SP, SR, A, I, C>,
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

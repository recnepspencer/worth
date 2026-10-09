//! Owner-held application plans for a closed, pinned pair read.

use super::super::WorthQueryAdmittedApplicationQueryPlan;

/// Two admitted public reads whose rooted identity is bound by Query before
/// dispatch. Application projection stays on the caller's thread. The second
/// scope must be the first row's declared identity field on the same basis.
/// No callback can select a new basis or issue another read on a worker.
pub struct WorthQueryDerivedPairReadPlans<'a, S, FQ, FP, FR, SQ, SP, SR, A, I, C> {
    pub(in crate::domain_computation::primary_graph::application_query) first:
        WorthQueryAdmittedApplicationQueryPlan<'a, S, FQ, FP, FR, A, I, C>,
    pub(in crate::domain_computation::primary_graph::application_query) second:
        WorthQueryAdmittedApplicationQueryPlan<'a, S, SQ, SP, SR, A, I, C>,
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
        Self { first, second }
    }
}

//! Work from this request attempt, separate from retained commit evidence.
use worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork;

/// A framework-issued outcome and the decision projection performed by this attempt.
/// Replay can return a historical commit while this attempt's work is `NotStarted`.
/// Captured work grants no graph read, mutation, publication or reuse authority.
///
/// Callers cannot attach invented work to an outcome:
/// ```compile_fail,E0451
/// use worth_query_publication::facade::application_entry::WorthQueryApplicationMutationAttemptReport;
/// use worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork;
/// let _report = WorthQueryApplicationMutationAttemptReport {
///     outcome: (), decision_work: WorthQueryMutationHandlerWork::NotStarted,
/// };
/// ```
///
/// Reporting preserves the required-source preparation phase:
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::WorthQueryApplicationMutationRequestWithIdempotency;
/// use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
/// use worth_query_declaration::facade::{application_operation::ApplicationMutationIntent,
///     application_program::ApplicationProgramDefinition};
/// use worth_query_installation::facade::ApplicationSchema;
/// fn cannot_publish_without_source<'a, Schema, Intent, Program, Root>(
///     request: WorthQueryApplicationMutationRequestWithIdempotency<'a, 'a, 'a, 'a, Schema, Intent>,
///     application: &'a WorthQueryProgramApplicationRuntime<Schema, Program>,
/// ) where Schema: ApplicationSchema, Intent: ApplicationMutationIntent<Schema>,
///     Program: ApplicationProgramDefinition<Schema> {
///     let _ = request.execute_performed_report::<Program, Root>(application);
/// }
/// ```
#[derive(Debug)]
pub struct WorthQueryApplicationMutationAttemptReport<Outcome> {
    outcome: Outcome,
    decision_work: WorthQueryMutationHandlerWork,
}
impl<Outcome> WorthQueryApplicationMutationAttemptReport<Outcome> {
    pub(super) fn new(outcome: Outcome, decision_work: WorthQueryMutationHandlerWork) -> Self {
        Self {
            outcome,
            decision_work,
        }
    }
    pub fn outcome(&self) -> &Outcome {
        &self.outcome
    }
    pub const fn decision_work(&self) -> &WorthQueryMutationHandlerWork {
        &self.decision_work
    }
    pub fn into_parts(self) -> (Outcome, WorthQueryMutationHandlerWork) {
        (self.outcome, self.decision_work)
    }
    pub fn into_outcome(self) -> Outcome {
        self.outcome
    }
    pub(super) fn map<Next>(
        self,
        map: impl FnOnce(Outcome) -> Next,
    ) -> WorthQueryApplicationMutationAttemptReport<Next> {
        WorthQueryApplicationMutationAttemptReport::new(map(self.outcome), self.decision_work)
    }
}

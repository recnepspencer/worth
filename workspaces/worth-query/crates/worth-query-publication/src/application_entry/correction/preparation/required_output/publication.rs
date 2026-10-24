//! One corrected source publication hands back the existing performed-source owners.
use super::*;
use crate::application_entry::correction::publication::PreparedCorrectionAction;
/// An admitted correction action and its demand retained until the target publishes.
/// Dropping it releases the unpublished preparation.
pub struct WorthQueryPreparedRequiredCorrection<'a, S, B, P, R>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>
        + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) action: PreparedCorrectionAction<'a, S, B>,
    pub(super) application: &'a WorthQueryProgramApplicationRuntime<S, P>,
    pub(super) demand: Demand<S, R>,
}
impl<S, B, P, R> WorthQueryPreparedRequiredCorrection<'_, S, B, P, R>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>
        + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    B::Input: Clone + Send + Sync,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn publish(self) -> Result<Outcome<S, B, P, R>, WorthQueryApplicationCorrectionDenial> {
        let Self {
            action,
            application,
            demand,
        } = self;
        let PreparedCorrectionAction {
            application: runtime,
            owner,
            program,
            identities,
            result,
            source,
        } = action;
        let scope = program.request_scope().clone();
        runtime.with_application_advancement(&scope, |phase| {
            let execution = phase.checked_execution_request_for(runtime.product_runtime())
                .map_err(WorthQueryApplicationCorrectionDenial::ExecutionRequest)?;
            let custody = PerformedSourceCommit::default();
            let commit = custody.record(application.commit_required_output_correction::<R, B>(
                &phase, &worth_query_execution::publication_boundary::program_publication_access(),
                &owner, program, &identities.borrow(), source, execution.allocation_policy(),
            ));
            let outcome = match commit.landed() {
                Ok((receipt, false)) => crate::application_entry::WorthQueryApplicationMutationOutcome::Committed { receipt, result },
                Ok((receipt, true)) => crate::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
                Err(custody) => crate::application_entry::WorthQueryApplicationMutationOutcome::Commit(custody),
            };
            Ok(performed_outcome::<S, B, P, R>(demand, outcome, custody)
                .retain_recorded_input(identities))
        }).map_err(WorthQueryApplicationCorrectionDenial::ExecutionRequest)?
    }
}

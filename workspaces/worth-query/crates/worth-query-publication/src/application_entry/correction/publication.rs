//! The prepared inverse publishes through its selected native program owner.
use super::{outcome::*, WorthQueryApplicationCorrectionDenial};
use crate::application_entry::mutation::preparation::CandidateProgram;
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationOwnedMutationIdentities,
};
use worth_query_execution::facade::{
    application_installation::{WorthQueryProgramOwner, WorthQuerySelectedProgramOwner},
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_execution::publication_boundary::recorded_correction::WorthQueryPreparedCorrectionSource;
use worth_query_installation::facade::ApplicationSchema;

/// An inverse candidate retaining its exact historical source and selected target.
/// Dropping the candidate discards it without publishing effects.
pub struct WorthQueryPreparedApplicationCorrection<'application> {
    pub(super) publication: Box<dyn CorrectionPublication + 'application>,
}
impl WorthQueryPreparedApplicationCorrection<'_> {
    pub fn publish(
        self,
    ) -> Result<WorthQueryApplicationCorrectionOutcome, WorthQueryApplicationCorrectionDenial> {
        self.publication.publish()
    }
}
pub(super) trait CorrectionPublication {
    fn publish(
        self: Box<Self>,
    ) -> Result<WorthQueryApplicationCorrectionOutcome, WorthQueryApplicationCorrectionDenial>;
}
pub(super) struct PreparedInverse<
    'application,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
> {
    pub(super) application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) owner: WorthQuerySelectedProgramOwner<'application, Schema>,
    pub(super) program: CandidateProgram<Schema, Binding>,
    pub(super) identities: ApplicationOwnedMutationIdentities<Schema, Binding>,
    pub(super) result: Binding::Result,
    pub(super) source: Arc<WorthQueryPreparedCorrectionSource>,
}
impl<Schema, Binding> CorrectionPublication for PreparedInverse<'_, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Binding::Input: Clone + Send + Sync,
{
    fn publish(
        self: Box<Self>,
    ) -> Result<WorthQueryApplicationCorrectionOutcome, WorthQueryApplicationCorrectionDenial> {
        let Self {
            application,
            owner,
            program,
            identities,
            result,
            source,
        } = *self;
        let scope = program.request_scope().clone();
        application.with_application_advancement(&scope, |phase| {
            let execution = phase.checked_execution_request_for(application.product_runtime())
                .map_err(WorthQueryApplicationCorrectionDenial::ExecutionRequest)?;
            let outcome = owner.commit_program_correction_in_advancement(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &phase, program, &identities.borrow(), source, execution.allocation_policy(),
            );
            let outcome = match outcome.landed() {
                Ok((receipt, false)) => crate::application_entry::WorthQueryApplicationMutationOutcome::Committed { receipt, result },
                Ok((receipt, true)) => crate::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(receipt),
                Err(custody) => crate::application_entry::WorthQueryApplicationMutationOutcome::Commit(custody),
            };
            Ok(erase::<Schema, Binding>(outcome))
        }).map_err(WorthQueryApplicationCorrectionDenial::ExecutionRequest)?
    }
}

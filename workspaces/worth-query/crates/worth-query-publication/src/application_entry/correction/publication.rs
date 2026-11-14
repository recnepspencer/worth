//! The prepared correction action publishes through its selected native program owner.
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

/// A correction candidate retaining its exact historical source and selected target.
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
pub(in crate::application_entry) struct PreparedCorrectionAction<
    'application,
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Identities = ApplicationOwnedMutationIdentities<Schema, Binding>,
> {
    pub(in crate::application_entry) application:
        &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(in crate::application_entry) owner: WorthQuerySelectedProgramOwner<'application, Schema>,
    pub(in crate::application_entry) program: CandidateProgram<Schema, Binding>,
    pub(in crate::application_entry) identities: Identities,
    pub(in crate::application_entry) result: Binding::Result,
    pub(in crate::application_entry) source: Arc<WorthQueryPreparedCorrectionSource>,
}

impl<S, B>
    PreparedCorrectionAction<
        '_,
        S,
        B,
        worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<
            '_,
            S,
            B,
        >,
    >
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: Clone + Send + Sync,
{
    pub(in crate::application_entry) fn commit_envelope<P, R>(
        self,
        program: &worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime<S, P>,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
        binding: worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
        unseen: &worth_query_execution::publication_boundary::authored_edit::UnseenAuthoredEnvelope<'_>,
        allocation: worth_query_execution::facade::runtime::ExecutionAllocationPolicy<'_, '_>,
    ) -> (worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitOutcome, B::Result)
    where
        P: worth_query_declaration::facade::application_program::ApplicationProgramDefinition<S>,
        R: worth_query_declaration::facade::application_program::ApplicationOutputGraphShape<S>,
        <R::RootConnection as worth_query_declaration::facade::application_program::ApplicationConnectionShape<S>>::Binding:
            worth_query_declaration::facade::application_program::WorthQueryApplicationRequiredOutputConnection<S>,
        B: worth_query_declaration::facade::application_program::WorthQueryApplicationRequiredOutputSource<S,
            <R::RootConnection as worth_query_declaration::facade::application_program::ApplicationConnectionShape<S>>::Binding>,
{
        let outcome = program.compare_and_commit_selected_corrective_envelope::<R, B>(
            phase,
            &worth_query_execution::publication_boundary::program_publication_access(),
            &self.owner,
            self.program,
            &self.identities,
            self.source,
            binding,
            unseen,
            allocation,
        );
        (outcome, self.result)
    }
}
impl<Schema, Binding> CorrectionPublication for PreparedCorrectionAction<'_, Schema, Binding>
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
                Ok(worth_query_execution::facade::primary_graph::WorthQueryApplicationLandedCommit::Live { receipt }) => crate::application_entry::WorthQueryApplicationMutationOutcome::Committed { receipt, result },
                Ok(worth_query_execution::facade::primary_graph::WorthQueryApplicationLandedCommit::Recovered(receipt)) => crate::application_entry::WorthQueryApplicationMutationOutcome::RecoveredCommitted(receipt),
                Err(custody) => crate::application_entry::WorthQueryApplicationMutationOutcome::Commit(custody),
            };
            Ok(erase::<Schema, Binding>(outcome))
        }).map_err(WorthQueryApplicationCorrectionDenial::ExecutionRequest)?
    }
}

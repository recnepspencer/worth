//! Exact installed custody before funding and inside one borrowed advancement.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryCapabilityRevocationProgram, WorthQueryDelegationActivationProgram,
    WorthQueryElevationApprovalProgram, WorthQueryElevationCloseProgram,
    WorthQueryElevationRequestProgram, WorthQueryMandatoryReviewProgram,
};
mod carrier;
pub(in crate::domain_computation::primary_graph) use carrier::CommitCarrier;

/// The exact program whose installed seal and current authority were readmitted.
/// Entry callers can neither construct nor extract its owned program.
pub struct ReadmittedCommitProgram<'a, Schema, Program> {
    application: &'a dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    program: Program,
}

/// Commit work borrows the exact advancement that checked its current authority.
pub struct CurrentCommitProgram<'a, 'phase, 'request, Schema, Program> {
    application: &'a dyn crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>,
    program: Program,
    phase: &'phase WorthQueryAdvancementPhase<'request>,
}

pub(in crate::domain_computation) trait EntryCustodyScopedOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    #[inline]
    fn validate_commit_program<Program: CommitCarrier<Schema>>(
        &self,
        program: Program,
    ) -> Result<ReadmittedCommitProgram<'_, Schema, Program>, Program::Refusal> {
        let (core, tag) = program.admission().reentry_parts();
        if let Err(outcome) = crate::domain_computation::primary_graph::application_attempt::effect_program::validate_installed_commit_program(
            self.application_attempt_host(),
            None,
            core,
            tag,
            &self.installed_schema().binding_identity(),
        ) {
            return Err(program.refuse(outcome));
        }
        Ok(ReadmittedCommitProgram {
            application: self.application_attempt_host(),
            program,
        })
    }

    #[inline]
    fn validate_commit_program_in_advancement<
        'a,
        'phase,
        'request,
        Program: CommitCarrier<Schema>,
    >(
        &'a self,
        phase: &'phase WorthQueryAdvancementPhase<'request>,
        program: Program,
    ) -> Result<CurrentCommitProgram<'a, 'phase, 'request, Schema, Program>, Program::Refusal> {
        let (core, tag) = program.admission().reentry_parts();
        if let Err(outcome) = crate::domain_computation::primary_graph::application_attempt::effect_program::validate_installed_commit_program(
            self.application_attempt_host(),
            Some(phase),
            core,
            tag,
            &self.installed_schema().binding_identity(),
        ) {
            return Err(program.refuse(outcome));
        }
        Ok(CurrentCommitProgram {
            application: self.application_attempt_host(),
            program,
            phase,
        })
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> EntryCustodyScopedOps<Schema> for HostRuntime {}

impl<'a, Schema: ApplicationSchema, Program: CommitCarrier<Schema>>
    ReadmittedCommitProgram<'a, Schema, Program>
{
    pub(in crate::domain_computation::primary_graph) fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.program.admission().publication_request()
    }
    pub(in crate::domain_computation::primary_graph) fn refuse_advancement(
        self,
        cause: crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
    ) -> Program::Refusal {
        self.program.refuse(cause.into_commit_outcome())
    }
    /// Immutable seal facts are carried; lapse is refreshed after advancement opens.
    pub(in crate::domain_computation::primary_graph) fn in_advancement<'phase, 'request>(
        self,
        phase: &'phase WorthQueryAdvancementPhase<'request>,
    ) -> Result<CurrentCommitProgram<'a, 'phase, 'request, Schema, Program>, Program::Refusal> {
        let (core, _) = self.program.admission().reentry_parts();
        if let Err(outcome) = crate::domain_computation::primary_graph::application_attempt::effect_program::validate_commit_lapse(
            self.application,
            phase,
            core,
        )
        {
            return Err(self.program.refuse(outcome));
        }
        Ok(CurrentCommitProgram {
            application: self.application,
            program: self.program,
            phase,
        })
    }
}
mod application_commit;
mod conditional_operation;
mod output_source;

mod forwarding;
pub(crate) mod preparation;
mod progression;
pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) use preparation::prepare_application_commit;

mod capability_revocation;
mod delegation_activation;
mod elevation_approval;
mod elevation_close;
mod elevation_request;
mod mandatory_review;

pub(crate) mod court_doors_for_test;
pub use court_doors_for_test::EntryCustodyDoorsOps;
pub use court_doors_for_test::EntryCustodyEntryCustodyDoorsOps;

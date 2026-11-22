//! Exact installed custody before funding and inside one borrowed advancement.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryCapabilityRevocationProgram, WorthQueryDelegationActivationProgram,
    WorthQueryElevationApprovalProgram, WorthQueryElevationCloseProgram,
    WorthQueryElevationRequestProgram, WorthQueryMandatoryReviewProgram,
};
pub mod carrier;

/// The exact program whose installed seal and current authority were readmitted.
/// Entry callers can neither construct nor extract its owned program.
pub struct ReadmittedCommitProgram<'a, Schema, Program> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    program: Program,
}

/// Commit work borrows the exact advancement that checked its current authority.
pub struct CurrentCommitProgram<'a, 'phase, 'request, Schema, Program> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    program: Program,
    phase: &'phase WorthQueryAdvancementPhase<'request>,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[inline]
    pub fn validate_commit_program<Program: CommitCarrier<Schema>>(
        &self,
        program: Program,
    ) -> Result<ReadmittedCommitProgram<'_, Schema, Program>, Program::Refusal> {
        let (core, tag) = program.admission().reentry_parts();
        if let Err(outcome) = self.runtime.validate_commit_program(
            None,
            core,
            tag,
            &self.installed_schema.binding_identity(),
        ) {
            return Err(program.refuse(outcome));
        }
        Ok(ReadmittedCommitProgram {
            application: self,
            program,
        })
    }

    #[inline]
    pub fn validate_commit_program_in_advancement<
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
        if let Err(outcome) = self.runtime.validate_commit_program(
            Some(phase),
            core,
            tag,
            &self.installed_schema.binding_identity(),
        ) {
            return Err(program.refuse(outcome));
        }
        Ok(CurrentCommitProgram {
            application: self,
            program,
            phase,
        })
    }
}

impl<'a, Schema: ApplicationSchema, Program: CommitCarrier<Schema>>
    ReadmittedCommitProgram<'a, Schema, Program>
{
    pub fn request_scope(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.program.admission().publication_request()
    }
    pub fn refuse_advancement(
        self,
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
    ) -> Program::Refusal {
        self.program.refuse(crate::domain_computation::primary_graph::application_attempt::effect_program::WorthQueryCommitEntryRefusal::from_advancement(cause))
    }
    /// Immutable seal facts are carried; lapse is refreshed after advancement opens.
    pub fn in_advancement<'phase, 'request>(
        self,
        phase: &'phase WorthQueryAdvancementPhase<'request>,
    ) -> Result<CurrentCommitProgram<'a, 'phase, 'request, Schema, Program>, Program::Refusal> {
        let (core, _) = self.program.admission().reentry_parts();
        if let Err(outcome) = self
            .application
            .runtime
            .validate_commit_lapse_in_advancement(phase, core)
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
pub mod application_commit;
pub mod conditional_operation;
pub mod output_source;

pub mod forwarding;
pub mod managed_start;
pub mod preparation;
pub mod progression;
pub use preparation::prepare_application_commit;

pub mod capability_revocation;
pub mod delegation_activation;
pub mod elevation_approval;
pub mod elevation_close;
pub mod elevation_request;
pub mod mandatory_review;

mod court_doors_for_test;

pub use carrier::CommitCarrier;

//! Sealed test doors over this module's private members.
use super::*;

pub trait EntryCustodyDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    #[doc(hidden)]
    fn validate_commit_program_for_test<
        Refusal,
        Program: CommitCarrier<Schema, Refusal = Refusal>,
    >(
        &self,
        program: Program,
    ) -> Result<ReadmittedCommitProgram<'_, Schema, Program>, Refusal> {
        Self::validate_commit_program::<Program>(self, program)
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> EntryCustodyDoorsOps<Schema> for HostRuntime {}

pub trait EntryCustodyEntryCustodyDoorsOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema>
{
    #[doc(hidden)]
    fn validate_commit_program_in_advancement_for_test<
        'a,
        'phase,
        'request,
        Refusal,
        Program: CommitCarrier<Schema, Refusal = Refusal>,
    >(
        &'a self,
        phase: &'phase WorthQueryAdvancementPhase<'request>,
        program: Program,
    ) -> Result<CurrentCommitProgram<'a, 'phase, 'request, Schema, Program>, Refusal> {
        Self::validate_commit_program_in_advancement::<Program>(self, phase, program)
    }
}

impl<Schema: ApplicationSchema, HostRuntime: crate::domain_computation::primary_graph::application_attempt::host::ApplicationAttemptHost<Schema> + ?Sized> EntryCustodyEntryCustodyDoorsOps<Schema> for HostRuntime {}

impl<'a, Schema: ApplicationSchema, Refusal, Program: CommitCarrier<Schema, Refusal = Refusal>>
    ReadmittedCommitProgram<'a, Schema, Program>
{
    #[doc(hidden)]
    pub fn in_advancement_for_test<'phase, 'request>(
        self,
        phase: &'phase WorthQueryAdvancementPhase<'request>,
    ) -> Result<CurrentCommitProgram<'a, 'phase, 'request, Schema, Program>, Refusal> {
        Self::in_advancement(self, phase)
    }
}

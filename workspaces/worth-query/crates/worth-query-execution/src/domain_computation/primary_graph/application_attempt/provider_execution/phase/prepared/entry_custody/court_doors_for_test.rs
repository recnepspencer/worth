//! Sealed test doors over this module's private members.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn validate_commit_program_for_test<
        Refusal,
        Program: CommitCarrier<Schema, Refusal = Refusal>,
    >(
        &self,
        program: Program,
    ) -> Result<ReadmittedCommitProgram<'_, Schema, Program>, Refusal> {
        Self::validate_commit_program::<Program>(self, program)
    }
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    #[doc(hidden)]
    pub fn validate_commit_program_in_advancement_for_test<
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

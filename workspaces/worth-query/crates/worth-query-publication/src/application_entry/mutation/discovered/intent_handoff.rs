//! Consuming nominal-intent transfer preserves the exact binding and all native custody.
use super::*;
impl<Schema, Intent, Program, Root>
    WorthQueryPerformedDiscoveredApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    /// Moves the same-binding evidence; it neither retains an intent borrow nor admits work.
    pub(in crate::application_entry) fn into_same_binding_intent<Original>(
        self,
    ) -> WorthQueryPerformedDiscoveredApplicationMutation<Schema, Original, Program, Root>
    where
        Original: ApplicationMutationIntent<Schema, Binding = Intent::Binding>,
    {
        WorthQueryPerformedDiscoveredApplicationMutation {
            receipt: self.receipt,
            result: self.result,
            discovery: self.discovery,
            prepared: self.prepared,
            program: self.program,
            retained_source: self.retained_source,
        }
    }
}
impl<Schema, Intent, Program, Root>
    WorthQueryStartedDiscoveredOutputs<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    /// Moves the same-binding evidence; it neither retains an intent borrow nor admits work.
    pub(in crate::application_entry) fn into_same_binding_intent<Original>(
        self,
    ) -> WorthQueryStartedDiscoveredOutputs<Schema, Original, Program, Root>
    where
        Original: ApplicationMutationIntent<Schema, Binding = Intent::Binding>,
    {
        WorthQueryStartedDiscoveredOutputs {
            receipt: self.receipt,
            result: self.result,
            required_output: self.required_output,
        }
    }
}
impl<Schema, Intent, Program, Root>
    WorthQueryDiscoveredOutputRetentionFailure<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    /// Moves the same-binding evidence; it neither retains an intent borrow nor admits work.
    pub(in crate::application_entry) fn into_same_binding_intent<Original>(
        self,
    ) -> WorthQueryDiscoveredOutputRetentionFailure<Schema, Original, Program, Root>
    where
        Original: ApplicationMutationIntent<Schema, Binding = Intent::Binding>,
    {
        WorthQueryDiscoveredOutputRetentionFailure {
            receipt: self.receipt,
            result: self.result,
            denial: self.denial,
            custody: self.custody,
            discovery: self.discovery,
            program: self.program,
        }
    }
}
impl<Schema, Intent, Program, Root>
    WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    /// Moves the same-binding evidence; it neither retains an intent borrow nor admits work.
    pub(in crate::application_entry) fn into_same_binding_intent<Original>(
        self,
    ) -> WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Original, Program, Root>
    where
        Original: ApplicationMutationIntent<Schema, Binding = Intent::Binding>,
    {
        WorthQueryUnpublishedDiscoveredApplicationMutation {
            source: self.source,
            discovery: self.discovery,
            phase: self.phase,
            prior_cleanup: self.prior_cleanup,
            initial_cause: self.initial_cause,
            program: std::marker::PhantomData,
        }
    }
}
impl<Schema, Intent, Program, Root>
    WorthQueryApplicationDiscoveredMutationOutcome<Schema, Intent, Program, Root>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
{
    /// Moves the same-binding evidence; it neither retains an intent borrow nor admits work.
    pub(in crate::application_entry) fn into_same_binding_intent<Original>(
        self,
    ) -> WorthQueryApplicationDiscoveredMutationOutcome<Schema, Original, Program, Root>
    where
        Original: ApplicationMutationIntent<Schema, Binding = Intent::Binding>,
    {
        match self {
            Self::CleanupRequired { cleanup, outcome } => {
                WorthQueryApplicationDiscoveredMutationOutcome::CleanupRequired {
                    cleanup,
                    outcome: Box::new(outcome.into_same_binding_intent()),
                }
            }
            Self::ProductUnpublished(partial) => {
                WorthQueryApplicationDiscoveredMutationOutcome::ProductUnpublished(
                    partial.into_same_binding_intent(),
                )
            }
            Self::Performed(performed) => {
                WorthQueryApplicationDiscoveredMutationOutcome::Performed(
                    performed.into_same_binding_intent(),
                )
            }
            Self::RequiredOutputDenied(failure) => {
                WorthQueryApplicationDiscoveredMutationOutcome::RequiredOutputDenied(
                    failure.into_same_binding_intent(),
                )
            }
            Self::Blocked(blocked) => {
                WorthQueryApplicationDiscoveredMutationOutcome::Blocked(blocked)
            }
            Self::NotPerformed(outcome) => {
                WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(outcome)
            }
        }
    }
}

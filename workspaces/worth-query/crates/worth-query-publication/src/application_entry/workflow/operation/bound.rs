//! Native workflow controls require explicit advanced selection; ordinary callers retain an owner.
use super::*;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeResolution;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;

/// An exact workflow-bound operation. Execute through the installed program or
/// prepare recovery from the original native owner. Pass this view directly to
/// `accept_operation_from_owner` or `accept_recovered_operation_from_owner` on
/// the prepared workflow advance; no native request extraction is available.
/// Ordinary code uses the retained workflow operation and fresh-request advancement.
#[must_use = "execute the bound workflow operation or recover its original owner custody"]
pub struct WorthQueryBoundWorkflowOperation<
    'application,
    'principal,
    'scope,
    'key,
    Schema,
    Intent,
    SourcePreparation,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(super) native: WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >,
}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryAdvancedWorkflowOperation<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Runs the bound operation through the single native installed-program funnel.
    pub fn execute_in_program<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        crate::application_entry::WorthQueryApplicationRequestMutationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        self.native
            .execute_in_program(application, allocation_policy)
    }
}

impl<'a, 'p, 's, 'k, S, I, Source> WorthQueryBoundWorkflowOperation<'a, 'p, 's, 'k, S, I, Source>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
{
    pub(in crate::application_entry) fn from_owned_view(
        native: WorthQueryApplicationMutationRequestWithIdempotency<'a, 'p, 's, 'k, S, I, Source>,
    ) -> Self {
        Self { native }
    }
}

/// Explicit native execution controls; no ordinary owning lifecycle is implied.
pub struct WorthQueryAdvancedWorkflowOperation<'a, 'p, 's, 'k, S, I, Source>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
{
    pub(super) native:
        WorthQueryApplicationMutationRequestWithIdempotency<'a, 'p, 's, 'k, S, I, Source>,
}
impl<'a, 'p, 's, 'k, S, I, Source> WorthQueryBoundWorkflowOperation<'a, 'p, 's, 'k, S, I, Source>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
{
    pub fn advanced(self) -> WorthQueryAdvancedWorkflowOperation<'a, 'p, 's, 'k, S, I, Source> {
        WorthQueryAdvancedWorkflowOperation {
            native: self.native,
        }
    }
}

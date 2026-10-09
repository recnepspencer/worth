//! Fresh request admission for an ordinary, no-source unpublished application.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
    NoApplicationMutationSource,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner,
};
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery;
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial,
    WorthQueryManagedApplicationRecoveryDenial, WorthQueryManagedApplicationRecoveryOutcome,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{authorization::PreparedMutation, WorthQueryApplicationMutationRequestWithIdempotency};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

/// A refusal before recovery performs a new product publication. The caller
/// retains its original recovery handle. Performed recovery and any subsequent
/// read, publication or cleanup failure remain in the recovery outcome instead.
#[derive(Debug)]
pub enum WorthQueryApplicationRecoveryRequestDenial {
    Request(WorthQueryApplicationRequestMutationDenial),
    Interrupted(WorthQueryRequestInterruption),
    /// Workflow control and transition-bound requests need their workflow owner.
    WorkflowUnsupported,
    Recovery(WorthQueryManagedApplicationRecoveryDenial),
    Idempotency(WorthQueryApplicationIdempotencyResolutionDenial),
}

impl From<WorthQueryApplicationRequestMutationDenial>
    for WorthQueryApplicationRecoveryRequestDenial
{
    fn from(denial: WorthQueryApplicationRequestMutationDenial) -> Self {
        Self::Request(denial)
    }
}

impl std::fmt::Display for WorthQueryApplicationRecoveryRequestDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application recovery request denied: {self:?}")
    }
}

impl std::error::Error for WorthQueryApplicationRecoveryRequestDenial {}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
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
    Intent: ApplicationMutationIntent<Schema>,
    Intent::Binding:
        ApplicationMutationBinding<Schema, SourceExpectation = NoApplicationMutationSource>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Continues an exact ordinary unpublished application under fresh current
    /// authorization. Supply the original intent, key and preconditions. The
    /// selected program must still own this action; no initial-owner fallback
    /// is used. This entrance never prepares a candidate or calls its handler.
    ///
    /// A performed outcome remains performed even if its receipt read or
    /// subsequent publication/cleanup reports a failure. Retain that posture
    /// and use [`Self::resolve_idempotency_in_program`] for a fresh read.
    pub fn recover_unpublished_in_program<Program>(
        mut self,
        recovery: &WorthQueryProductUnpublishedRecovery,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryApplicationRecoveryRequestDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let prepared = self.authorize_recovery(application)?;
        self.check_recovery_liveness()?;
        self.request
            .application
            .recover_admitted_unpublished_application(
                recovery,
                &prepared.admission,
                prepared.idempotency,
            )
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Recovery)
    }

    /// Reads the original keyed outcome through fresh selected-program and
    /// request authorization, without preparing or executing a mutation.
    pub fn resolve_idempotency_in_program<Program>(
        mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationRecoveryRequestDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let prepared = self.authorize_recovery(application)?;
        self.check_recovery_liveness()?;
        self.request
            .application
            .resolve_admitted_application_idempotency(&prepared.admission, prepared.idempotency)
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Idempotency)
    }

    fn authorize_recovery<Program>(
        &mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<PreparedMutation<Schema, Intent::Binding>, WorthQueryApplicationRecoveryRequestDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        if <Intent::Binding as ApplicationMutationBinding<Schema>>::WORKFLOW_CONTROL
            || <Intent::Binding as ApplicationMutationBinding<Schema>>::REQUIRES_WORKFLOW_AUTHORITY
            || self.workflow_transition_identity.is_some()
            || self.workflow_input.is_some()
            || self.workflow_authority.is_some()
        {
            return Err(WorthQueryApplicationRecoveryRequestDenial::WorkflowUnsupported);
        }
        self.check_recovery_liveness()?;
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(
                WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch.into(),
            );
        }
        let selected = self
            .request
            .application
            .on_branch(self.request.branch)
            .select()
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
        let owner = application
            .selected_program_owner(&selected)
            .map_err(super::selected_program::map_selected_program_owner_denial)?;
        if !owner.contains_action::<Intent::Binding>()
            || owner.owns_output_source(std::any::TypeId::of::<Intent::Binding>())
        {
            return Err(
                WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired.into(),
            );
        }
        let staged = self.stage()?;
        let identities = self.identities()?;
        super::authorization::prepare_selected(self, &identities, staged, &selected)
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Request)
    }

    fn check_recovery_liveness(&self) -> Result<(), WorthQueryApplicationRecoveryRequestDenial> {
        match self.request.scope.interruption() {
            Some(interruption) => Err(WorthQueryApplicationRecoveryRequestDenial::Interrupted(
                interruption,
            )),
            None => Ok(()),
        }
    }
}

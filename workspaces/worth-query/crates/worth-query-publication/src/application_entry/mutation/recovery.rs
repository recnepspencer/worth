//! Fresh request admission for an ordinary unpublished application.

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
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

impl WorthQueryApplicationRecoveryRequestDenial {
    pub(in crate::application_entry::mutation) fn advancement(
        cause: worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ) -> Self {
        Self::Recovery(WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied(
            cause,
        ))
    }
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
    /// Source-bound actions also require their original checked row or result-set
    /// observation identity. A different source binding cannot replace it.
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
        let runtime = self.request.application;
        let scope = self.request.scope.clone();
        runtime
            .with_application_advancement(&scope, |phase| {
                let prepared = self.authorize_recovery(&phase, application)?;
                self.check_recovery_liveness()?;
                self.request
                    .application
                    .recover_admitted_unpublished_application_in_advancement(
                        &phase,
                        recovery,
                        &prepared.admission,
                        prepared.idempotency,
                    )
                    .map_err(WorthQueryApplicationRecoveryRequestDenial::Recovery)
            })
            .map_err(WorthQueryApplicationRecoveryRequestDenial::advancement)?
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
        let runtime = self.request.application;
        let scope = self.request.scope.clone();
        runtime
            .with_application_advancement(&scope, |phase| {
                let prepared = self.authorize_recovery(&phase, application)?;
                self.check_recovery_liveness()?;
                self.read_recovery_idempotency(&phase, &prepared)
            })
            .map_err(WorthQueryApplicationRecoveryRequestDenial::advancement)?
    }

    fn authorize_recovery<Program>(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<PreparedMutation<Schema, Intent::Binding>, WorthQueryApplicationRecoveryRequestDenial>
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        self.request
            .application
            .validate_application_advancement(phase)
            .map_err(|cause| {
                WorthQueryApplicationRecoveryRequestDenial::advancement(cause.into())
            })?;

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
        // Bind the original source identity and partition for recovery matching.
        // The retained attempt already owns its source facts: do not consume the
        // pending expectation into a fresh candidate or invoke the handler.
        super::authorization::prepare_selected(phase, self, &identities, staged, &selected)
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Request)
    }

    pub(in crate::application_entry::mutation) fn read_recovery_idempotency(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        prepared: &PreparedMutation<Schema, Intent::Binding>,
    ) -> Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationRecoveryRequestDenial> {
        self.request
            .application
            .validate_application_advancement(phase)
            .map_err(|cause| {
                WorthQueryApplicationRecoveryRequestDenial::advancement(cause.into())
            })?;
        self.request
            .application
            .resolve_admitted_application_idempotency(&prepared.admission, prepared.idempotency)
            .map_err(WorthQueryApplicationRecoveryRequestDenial::Idempotency)
    }

    pub(in crate::application_entry::mutation) fn check_recovery_liveness(
        &self,
    ) -> Result<(), WorthQueryApplicationRecoveryRequestDenial> {
        match self.request.scope.interruption() {
            Some(stop) => Err(WorthQueryApplicationRecoveryRequestDenial::advancement(
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(match stop {
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled => worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled,
                    worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::DeadlineExceeded => worth_query_execution::facade::application_contribution::WorthQueryManagedComputationInterruption::DeadlineExceeded,
                }),
            )),
            None => Ok(()),
        }
    }
}

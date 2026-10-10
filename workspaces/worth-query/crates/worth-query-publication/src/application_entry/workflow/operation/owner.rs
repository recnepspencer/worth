//! Recover one performed operation from its owner instead of a client receipt.

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_execution::publication_boundary::workflow_advance::{
    RequiredWorkflowOperation, WorkflowProgressOutcome, WorthQueryGuardedWorkflowOperationCustody,
    WorthQueryWorkflowAdvanceAdapter,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::mutation::{
    authorization, WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryApplicationRecoveryRequestDenial,
};
use crate::application_entry::{
    WorthQueryApplicationRequestMutationDenial, WorthQueryWorkflowAdvanceRequest,
};

use super::{
    WorthQueryWorkflowOperationAcceptanceDenial, WorthQueryWorkflowOperationBindingDenial,
};

/// Where the owner of a workflow operation's idempotency key stands when it has not
/// committed the operation.
#[derive(Debug)]
pub enum WorthQueryWorkflowOperationOwnerPosture {
    Unseen,
    IntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
    DispatchPending,
    Indeterminate(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
}

/// Why an operation recovered from its owner was not accepted. `RecoveryNotRequired` means
/// the owner already committed it.
#[derive(Debug)]
pub enum WorthQueryWorkflowOperationOwnerAcceptanceDenial {
    Binding(WorthQueryWorkflowOperationBindingDenial),
    Request(WorthQueryApplicationRequestMutationDenial),
    Opening(WorthQueryApplicationRecoveryRequestDenial),
    Inspection(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    Owner(WorthQueryWorkflowOperationOwnerPosture),
    RecoveryNotRequired,
    Acceptance(WorthQueryWorkflowOperationAcceptanceDenial),
}

impl std::fmt::Display for WorthQueryWorkflowOperationOwnerAcceptanceDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binding(denial) => {
                write!(formatter, "workflow operation binding denied: {denial:?}")
            }
            Self::Request(denial) => denial.fmt(formatter),
            Self::Opening(denial) => denial.fmt(formatter),
            Self::Inspection(denial) => denial.fmt(formatter),
            Self::Owner(posture) => {
                write!(formatter, "workflow operation owner custody: {posture:?}")
            }
            Self::RecoveryNotRequired => {
                formatter.write_str("workflow operation recovery is not required")
            }
            Self::Acceptance(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorthQueryWorkflowOperationOwnerAcceptanceDenial {}

impl<'application, 'principal, 'scope, Schema, Spec, Operation, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    pub fn accept_operation_from_owner<Intent, SourcePreparation>(
        self,
        required: &RequiredWorkflowOperation,
        mut operation: WorthQueryApplicationMutationRequestWithIdempotency<
            '_,
            '_,
            '_,
            '_,
            Schema,
            Intent,
            SourcePreparation,
        >,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowOperationOwnerAcceptanceDenial>
    where
        Intent: ApplicationMutationIntent<Schema>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync + 'static,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let application = self.application;
        let scope = self.scope.clone();
        application
            .with_application_advancement(&scope, |phase| {
                let (prepared, custody) = resolve_owner(&phase, &self, required, &mut operation)?;
                match custody {
                    WorthQueryGuardedWorkflowOperationCustody::Committed(receipt)
                    | WorthQueryGuardedWorkflowOperationCustody::ExternallySettled(receipt) => self
                        .accept_custody::<Intent::Binding, _, _, _>(
                            &phase,
                            required,
                            &receipt,
                            None,
                            &prepared.admission,
                            prepared.idempotency,
                        )
                        .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Acceptance),
                    WorthQueryGuardedWorkflowOperationCustody::DispatchPending(_) => {
                        Err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                            WorthQueryWorkflowOperationOwnerPosture::DispatchPending,
                        ))
                    }
                    other => Err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                        other_custody(other),
                    )),
                }
            })
            .map_err(|cause| {
                WorthQueryWorkflowOperationOwnerAcceptanceDenial::Opening(
                    WorthQueryApplicationRecoveryRequestDenial::advancement(cause),
                )
            })?
    }

    pub fn accept_recovered_operation_from_owner<Intent, SourcePreparation>(
        self,
        required: &RequiredWorkflowOperation,
        mut operation: WorthQueryApplicationMutationRequestWithIdempotency<
            '_,
            '_,
            '_,
            '_,
            Schema,
            Intent,
            SourcePreparation,
        >,
        recovery: &worth_query_execution::facade::primary_graph::WorthQueryRecoverySafeRetryAdmission,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowOperationOwnerAcceptanceDenial>
    where
        Intent: ApplicationMutationIntent<Schema>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync + 'static,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
            >,
    {
        let application = self.application;
        let scope = self.scope.clone();
        application
            .with_application_advancement(&scope, |phase| {
                let (prepared, custody) = resolve_owner(&phase, &self, required, &mut operation)?;
                match custody {
                    WorthQueryGuardedWorkflowOperationCustody::DispatchPending(receipt)
                    | WorthQueryGuardedWorkflowOperationCustody::ExternallySettled(receipt) => self
                        .accept_custody::<Intent::Binding, _, _, _>(
                            &phase,
                            required,
                            &receipt,
                            Some(recovery),
                            &prepared.admission,
                            prepared.idempotency,
                        )
                        .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Acceptance),
                    WorthQueryGuardedWorkflowOperationCustody::Committed(_) => {
                        Err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::RecoveryNotRequired)
                    }
                    other => Err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                        other_custody(other),
                    )),
                }
            })
            .map_err(|cause| {
                WorthQueryWorkflowOperationOwnerAcceptanceDenial::Opening(
                    WorthQueryApplicationRecoveryRequestDenial::advancement(cause),
                )
            })?
    }
}

fn resolve_owner<Schema, Spec, Operation, Input, Scope, Intent, SourcePreparation>(
    phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    advance: &WorthQueryWorkflowAdvanceRequest<'_, '_, '_, Schema, Spec, Operation, Input, Scope>,
    required: &RequiredWorkflowOperation,
    operation: &mut WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >,
) -> Result<
    (
        authorization::PreparedMutation<Schema, Intent::Binding>,
        WorthQueryGuardedWorkflowOperationCustody,
    ),
    WorthQueryWorkflowOperationOwnerAcceptanceDenial,
>
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + 'static,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    let staged = operation
        .stage()
        .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Request)?;
    let identities = operation
        .identities()
        .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Request)?;
    operation
        .validate_workflow_operation_binding(
            advance.workflow,
            required,
            identities.input_identity(),
        )
        .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Binding)?;
    if operation.workflow_transition_identity() != Some(*required.transition_identity_bytes()) {
        return Err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Binding(
            WorthQueryWorkflowOperationBindingDenial::RequirementMismatch,
        ));
    }
    let prepared = authorization::prepare(phase, operation, &identities, staged)
        .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Request)?;
    let custody = WorthQueryWorkflowAdvanceAdapter::resolve_guarded_operation_custody(
        advance.application,
        &prepared.admission,
        prepared.idempotency,
        required.transition_identity_bytes(),
    )
    .map_err(WorthQueryWorkflowOperationOwnerAcceptanceDenial::Inspection)?;
    Ok((prepared, custody))
}

pub(super) fn other_custody(
    custody: WorthQueryGuardedWorkflowOperationCustody,
) -> WorthQueryWorkflowOperationOwnerPosture {
    use WorthQueryGuardedWorkflowOperationCustody as Custody;
    use WorthQueryWorkflowOperationOwnerPosture as Posture;
    match custody {
        Custody::Unseen => Posture::Unseen,
        Custody::IntentDrift => Posture::IntentDrift,
        Custody::PublicationPending => Posture::PublicationPending,
        Custody::ProductUnpublished(handle) => Posture::ProductUnpublished(handle),
        Custody::Indeterminate(denial) => Posture::Indeterminate(denial),
        Custody::Committed(_) | Custody::ExternallySettled(_) | Custody::DispatchPending(_) => {
            unreachable!("committed custody is handled by the accepting method")
        }
    }
}

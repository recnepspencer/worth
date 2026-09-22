use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationScopeResolution,
};
use worth_query_execution::facade::primary_graph::{
    safe_retry_recovery_handle, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationCommitReceipt, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial, WorthQueryRecoveryHandleDenialKind,
    WorthQueryRecoverySafeRetryAdmission,
};
use worth_query_execution::facade::workflow_advance::WorthQueryWorkflowAdvanceAdapter;
use worth_query_installation::facade::ApplicationSchema;

use crate::application_entry::mutation::{
    authorization, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

type IntentBinding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type MutationScope<Schema, Binding> =
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope;

#[derive(Debug)]
pub enum WorthQueryWorkflowOperationRecoveryPreparationDenial {
    NotWorkflowBound,
    RecoveryNotRequired,
    Request(WorthQueryApplicationRequestMutationDenial),
    Recovery(WorthQueryRecoveryHandleDenial),
}

impl std::fmt::Display for WorthQueryWorkflowOperationRecoveryPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotWorkflowBound => {
                formatter.write_str("operation recovery request is not workflow-bound")
            }
            Self::RecoveryNotRequired => {
                formatter.write_str("operation receipt has no unresolved external custody")
            }
            Self::Request(denial) => denial.fmt(formatter),
            Self::Recovery(denial) => write!(formatter, "{denial:?}"),
        }
    }
}

impl std::error::Error for WorthQueryWorkflowOperationRecoveryPreparationDenial {}

pub struct WorthQueryPreparedWorkflowOperationRecovery<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        MutationScope<Schema, Binding>,
    >,
    handle: WorthQueryRecoveryHandle,
}

pub struct WorthQueryWorkflowOperationRecoveryDenial<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    denial: WorthQueryRecoveryHandleDenial,
    recovery:
        Option<Box<WorthQueryPreparedWorkflowOperationRecovery<'application, Schema, Binding>>>,
}

impl<'application, Schema, Binding>
    WorthQueryWorkflowOperationRecoveryDenial<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub const fn denial(&self) -> &WorthQueryRecoveryHandleDenial {
        &self.denial
    }

    pub fn into_recovery(
        self,
    ) -> Option<WorthQueryPreparedWorkflowOperationRecovery<'application, Schema, Binding>> {
        self.recovery.map(|recovery| *recovery)
    }
}

impl<'application, Schema, Binding> std::fmt::Debug
    for WorthQueryWorkflowOperationRecoveryDenial<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryWorkflowOperationRecoveryDenial")
            .field("denial", &self.denial)
            .field("recovery_retained", &self.recovery.is_some())
            .finish()
    }
}

impl<'application, Schema, Binding> std::fmt::Display
    for WorthQueryWorkflowOperationRecoveryDenial<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self.denial)
    }
}

impl<'application, Schema, Binding> std::error::Error
    for WorthQueryWorkflowOperationRecoveryDenial<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
}

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
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn prepare_workflow_operation_recovery(
        mut self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<
        WorthQueryPreparedWorkflowOperationRecovery<'application, Schema, IntentBinding<Schema, Intent>>,
        WorthQueryWorkflowOperationRecoveryPreparationDenial,
    > {
        if self.workflow_transition_identity().is_none() {
            return Err(WorthQueryWorkflowOperationRecoveryPreparationDenial::NotWorkflowBound);
        }
        if !WorthQueryWorkflowAdvanceAdapter::operation_receipt_requires_recovery(receipt) {
            return Err(
                WorthQueryWorkflowOperationRecoveryPreparationDenial::RecoveryNotRequired,
            );
        }
        let application = self.application_runtime();
        let prepared = authorization::prepare(&mut self)
            .map_err(WorthQueryWorkflowOperationRecoveryPreparationDenial::Request)?;
        if !WorthQueryWorkflowAdvanceAdapter::recovery_request_matches_receipt(
            prepared.idempotency,
            receipt.authority_binding().idempotency_binding(),
        ) {
            return Err(WorthQueryWorkflowOperationRecoveryPreparationDenial::Recovery(
                WorthQueryRecoveryHandleDenial::new(
                    WorthQueryRecoveryHandleDenialKind::IdempotencyMismatch,
                ),
            ));
        }
        let handle = application
            .mint_recovery_handle(receipt)
            .map_err(WorthQueryWorkflowOperationRecoveryPreparationDenial::Recovery)?;
        Ok(WorthQueryPreparedWorkflowOperationRecovery {
            application,
            admission: prepared.admission,
            handle,
        })
    }
}

impl<'application, Schema, Binding>
    WorthQueryPreparedWorkflowOperationRecovery<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    pub fn safe_retry(
        self,
    ) -> Result<
        WorthQueryRecoverySafeRetryAdmission,
        WorthQueryWorkflowOperationRecoveryDenial<'application, Schema, Binding>,
    > {
        let Self {
            application,
            admission,
            handle,
        } = self;
        let authority = match application.admit_recovery_effect_authority(&handle, &admission) {
            Ok(authority) => authority,
            Err(denial) => {
                return Err(retained(denial, application, admission, handle));
            }
        };
        let redispatch = match application
            .redispatch_admitted_external_effect(&handle, &authority, &admission)
        {
            Ok(redispatch) => redispatch,
            Err(denial) => {
                return Err(retained(denial.into(), application, admission, handle));
            }
        };
        safe_retry_recovery_handle(handle, &authority, redispatch).map_err(|denial| {
            let (denial, handle) = denial.into_parts();
            WorthQueryWorkflowOperationRecoveryDenial {
                denial,
                recovery: handle.map(|handle| {
                    Box::new(WorthQueryPreparedWorkflowOperationRecovery {
                        application,
                        admission,
                        handle,
                    })
                }),
            }
        })
    }
}

fn retained<'application, Schema, Binding>(
    denial: WorthQueryRecoveryHandleDenial,
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: WorthQueryAdmittedApplicationOperation<
        Schema,
        Binding::Operation,
        Binding::Input,
        MutationScope<Schema, Binding>,
    >,
    handle: WorthQueryRecoveryHandle,
) -> WorthQueryWorkflowOperationRecoveryDenial<'application, Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    let terminal = denial.kind() == WorthQueryRecoveryHandleDenialKind::AlreadyTerminal;
    WorthQueryWorkflowOperationRecoveryDenial {
        denial,
        recovery: (!terminal).then(|| {
            Box::new(WorthQueryPreparedWorkflowOperationRecovery {
                application,
                admission,
                handle,
            })
        }),
    }
}

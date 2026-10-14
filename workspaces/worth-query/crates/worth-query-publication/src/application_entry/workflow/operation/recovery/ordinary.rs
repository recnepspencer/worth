//! Restricted ordinary delegation to the existing native recovery owner.
use super::*;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    super::super::WorthQueryBoundWorkflowOperation<
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
    pub fn prepare_workflow_operation_recovery_from_owner(
        self,
        required: &RequiredWorkflowOperation,
    ) -> Result<
        WorthQueryPreparedWorkflowOperationRecovery<'application, Schema, IntentBinding<Schema, Intent>>,
        WorthQueryWorkflowOperationRecoveryPreparationDenial,
    >
    where
        <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Input:
            Clone + Send + Sync + 'static,
    {
        self.native.prepare_workflow_operation_recovery_from_owner(required)
    }
}

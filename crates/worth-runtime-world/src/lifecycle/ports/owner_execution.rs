//! One original owner progression, with an optional borrowed Native handoff.
use super::*;
use crate::lifecycle::owner::RelationalCutoverHandoff;

pub(crate) trait RuntimeWorldOwnerExecutionService {
    type SignalDefinition: Copy + Ord + std::fmt::Debug + 'static;
    type SignalIdentity: Copy + Ord;
    type SignalEvent;
    type SignalContext;
    type SignalTransactionKey: Copy + Ord;

    #[cfg(test)]
    fn execute_without_signal(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithoutSignal,
        cancellation: &RuntimeWorldCancellationToken,
    ) -> OwnerExecutionOutcome {
        self.execute_without_signal_with_cutover(execution, prepared, cancellation, None)
    }

    fn execute_without_signal_with_cutover(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithoutSignal,
        cancellation: &RuntimeWorldCancellationToken,
        cutover: Option<&mut RelationalCutoverHandoff<'_>>,
    ) -> OwnerExecutionOutcome;

    #[cfg(test)]
    fn execute_with_signal<F>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithSignal,
        runtime_ctx: &mut Self::SignalContext,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
    ) -> OwnerExecutionOutcome
    where
        F: FnOnce(
            &mut SignalTransaction<
                '_,
                Self::SignalDefinition,
                Self::SignalIdentity,
                Self::SignalEvent,
                Self::SignalContext,
                Self::SignalTransactionKey,
            >,
        ) -> Result<(), SignalError>,
    {
        self.execute_with_signal_with_cutover(
            execution,
            prepared,
            runtime_ctx,
            cancellation,
            apply,
            None,
        )
    }

    fn execute_with_signal_with_cutover<F>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithSignal,
        runtime_ctx: &mut Self::SignalContext,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
        cutover: Option<&mut RelationalCutoverHandoff<'_>>,
    ) -> OwnerExecutionOutcome
    where
        F: FnOnce(
            &mut SignalTransaction<
                '_,
                Self::SignalDefinition,
                Self::SignalIdentity,
                Self::SignalEvent,
                Self::SignalContext,
                Self::SignalTransactionKey,
            >,
        ) -> Result<(), SignalError>;

    fn execute_authored_conditional_definition_with_signal<F>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithSignal,
        publication: std::sync::Arc<
            crate::publication::AuthoredConditionalDefinitionAttemptCustody,
        >,
        runtime_ctx: &mut Self::SignalContext,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
    ) -> OwnerExecutionOutcome
    where
        F: FnOnce(
            &mut SignalTransaction<
                '_,
                Self::SignalDefinition,
                Self::SignalIdentity,
                Self::SignalEvent,
                Self::SignalContext,
                Self::SignalTransactionKey,
            >,
        ) -> Result<(), SignalError>;

    fn execute_conditional_definition_with_signal_and_cutover<F, H>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        prepared: PreparedCompositePublicationWithSignal,
        publication: worth_signal::facade::branch::SignalConditionalDefinitionPublicationOperation,
        runtime_ctx: &mut Self::SignalContext,
        cancellation: &RuntimeWorldCancellationToken,
        apply: F,
        admit_activation: H,
        cutover: Option<&mut RelationalCutoverHandoff<'_>>,
    ) -> OwnerExecutionOutcome
    where
        F: FnOnce(
            &mut SignalTransaction<
                '_,
                Self::SignalDefinition,
                Self::SignalIdentity,
                Self::SignalEvent,
                Self::SignalContext,
                Self::SignalTransactionKey,
            >,
        ) -> Result<(), SignalError>,
        H: FnOnce() -> Result<(), SignalError>;
}

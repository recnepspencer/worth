//! Deliver an already prepared result through the original C and lifetime owners.
use super::*;

impl HistoryProducerInvocation<'_> {
    pub(in crate::domain_computation::primary_graph) fn confirm_read<S: ApplicationSchema, T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, WorthQueryHistoryNavigationDenial> {
        let execution = phase.execution_request_for(&runtime.runtime.product_runtime)
            .map_err(|_| WorthQueryHistoryNavigationDenial::Document(
                crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial::ForeignPhase,
            ))?;
        self.confirm_in_execution(&execution, runtime, value)
    }

    pub(in crate::domain_computation::primary_graph) fn confirm_in_execution<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, WorthQueryHistoryNavigationDenial> {
        self.confirm_in_execution_preserving(execution, runtime, value)
            .map_err(|(_value, cause)| cause)
    }

    pub(in crate::domain_computation::primary_graph) fn confirm_in_execution_preserving<
        S: ApplicationSchema,
        T,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        value: T,
    ) -> Result<T, (T, WorthQueryHistoryNavigationDenial)> {
        use WorthQueryHistoryNavigationDenial as Denial;
        let mut value = Some(value);
        let result = self.selection.while_live(|| self.callback.while_live(|| {
            let original = value.take().expect("one original result handoff");
            match crate::domain_computation::primary_graph::product_operation::document_history::current_head::confirm_in_execution(
                runtime, execution, self.current, Some(self.request), original,
            ) {
                Ok(delivered) => Ok(delivered),
                Err((original, cause)) => { value = Some(original); Err(cause) }
            }
        }));
        // No owning result cleanup or public-denial allocation runs under the seals.
        match result {
            Ok(Ok(Ok((delivered, None)))) => Ok(delivered),
            Ok(Ok(Ok((refused, Some(lapse))))) => Err((
                refused,
                Denial::Authorization(WorthQueryOperationAuthorizationDenial::new(
                    lapse.denial_kind(),
                    "history output request",
                )),
            )),
            Ok(Ok(Err(cause))) => Err((
                value.take().expect("refused result remains owned"),
                Denial::Document(cause),
            )),
            _ => Err((
                value.take().expect("unentered result remains owned"),
                Denial::HeadChanged,
            )),
        }
    }
}

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
        let result = self.selection.while_live(|| {
            self.callback.while_live(|| {
                let original = value.take().expect("one original result handoff");
                match self
                    .read_control
                    .confirm_in_execution_preserving(execution, runtime, original)
                {
                    Ok(delivered) => Ok(delivered),
                    Err((original, cause)) => {
                        value = Some(original);
                        Err(cause)
                    }
                }
            })
        });
        // No owning result cleanup or public-denial allocation runs under the seals.
        match result {
            Ok(Ok(Ok(delivered))) => Ok(delivered),
            Ok(Ok(Err(cause))) => Err((
                value.take().expect("refused result remains owned"),
                read_denial(cause, "history output request"),
            )),
            _ => Err((
                value.take().expect("unentered result remains owned"),
                Denial::HeadChanged,
            )),
        }
    }
}

/// Allocate public history diagnostics only after all current/lifetime guards exit.
pub(super) fn read_denial(
    cause: crate::domain_computation::primary_graph::product_operation::current_read::CurrentReadDenial,
    subject: &'static str,
) -> WorthQueryHistoryNavigationDenial {
    use crate::domain_computation::primary_graph::product_operation::{
        current_read::CurrentReadDenial as Read, document_history::current_head,
    };
    use WorthQueryHistoryNavigationDenial as Denial;
    match cause {
        Read::Authorization(lapse) => Denial::Authorization(
            WorthQueryOperationAuthorizationDenial::new(lapse.denial_kind(), subject),
        ),
        Read::Busy => unreachable!("acquiring confirmation never tries a lane"),
        cause => Denial::Document(current_head::document_denial(cause)),
    }
}

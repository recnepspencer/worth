//! Consuming disposal removes business advancement while retaining actual original cleanup.
use super::*;

pub(in crate::application_entry) struct WorthQueryClosingSourceOperation<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
    M,
> {
    operation: WorthQuerySourceOnlyOperation<S, I, P, A, M>,
    closed: Option<crate::application_entry::WorthQueryClosedOperation>,
}

impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) fn into_original_disposal(
        mut self,
    ) -> Result<
        WorthQueryClosingSourceOperation<S, I, P, A, M>,
        (
            Self,
            crate::application_entry::WorthQueryOperationCloseDenial,
        ),
    > {
        let prepared = match &mut self.state {
            SourceState::<S, I>::Unpublished(recovery) => {
                self.cleanup.retain_original_recovery(recovery.clone())
            }
            SourceState::<S, I>::Performed(performed) => self.cleanup.absorb_recovered(performed),
            SourceState::<S, I>::RetainedPending(_) => Ok(()),
            state if native_posture::can_close::<S, I>(state) => Ok(()),
            _ => Err(
                crate::application_entry::WorthQueryOperationCloseDenial::NativeObligationRetained,
            ),
        };
        match prepared {
            Ok(()) => Ok(WorthQueryClosingSourceOperation {
                operation: self,
                closed: None,
            }),
            Err(cause) => Err((self, cause)),
        }
    }
}

impl<S, I, P, A, M> WorthQueryClosingSourceOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in crate::application_entry) fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        crate::application_entry::WorthQueryClosedOperation,
        (
            Self,
            crate::application_entry::WorthQueryOperationCloseDenial,
        ),
    > {
        use crate::application_entry::WorthQueryOperationCloseDenial as Denial;
        let runtime = self.operation.admission.runtime.runtime();
        if !std::ptr::eq(runtime, request.application)
            || request.branch != self.operation.admission.description.branch
        {
            return Err((self, Denial::FreshRequestMismatch));
        }
        let outcome = runtime.with_application_advancement(request.scope, |phase| {
            if self.closed.is_none() {
                self.closed = Some(
                    crate::application_entry::WorthQueryClosedOperation::from_cleanup(
                        self.operation
                            .cleanup
                            .close_original_in_advancement(&phase, runtime)?,
                    ),
                );
            }
            Ok(())
        });
        match outcome {
            Ok(Ok(())) => Ok(self
                .closed
                .take()
                .expect("actual original cleanup completed")),
            Ok(Err(cause)) => Err((self, cause)),
            Err(cause) => Err((self, Denial::Execution(cause))),
        }
    }

    pub(in crate::application_entry) fn target(
        &self,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.operation.admission.description.branch
    }
}

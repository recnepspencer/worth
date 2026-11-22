//! Input preparation retains the original request's work and stop lineage.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::{
    request_execution::advancement::advancement_denial, WorthQueryAdvancementDenial,
};

impl<S, B, C> WorthQueryProducerInputReader<'_, '_, '_, S, B, C>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    C: WorthQueryProducerInputCaptureDefinition<S>,
{
    /// Numerical input admission borrows this same request; it opens no writer.
    pub fn execution(&self) -> &QueryRequestExecution<'_> {
        self.context_use
            .set(self.context_use.get().request_context());
        self.execution
    }

    /// Interruption stays sticky even when the owner catches the poll result.
    pub fn checkpoint(&self) -> Result<(), HandlerInterruption> {
        let interruption = self.execution.interruption().map(|stop| match stop {
            WorthQueryManagedComputationInterruption::Cancelled => HandlerInterruption::Cancelled,
            WorthQueryManagedComputationInterruption::DeadlineExceeded => {
                HandlerInterruption::DeadlineExceeded
            }
        });
        if let Some(stop) = interruption {
            self.stopped
                .borrow_mut()
                .get_or_insert(InputCaptureStop::Interrupted(stop));
            return Err(stop);
        }
        match self.stopped.borrow().as_ref() {
            Some(InputCaptureStop::Interrupted(stop)) => Err(*stop),
            _ => Ok(()),
        }
    }

    pub(crate) fn charge(&self, work: u64) -> Result<(), HandlerExecutionDenial> {
        self.execution
            .decision_stage(|context| {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                    context, work,
                )
            })
            .map_err(WorthQueryAdvancementDenial::from)
            .and_then(|result| {
                result.map_err(|cause| {
                    advancement_denial(worth_execution::WorkCeilingDenial::Stopped(cause))
                })
            })
            .map_err(|cause| {
                self.stop(match cause {
                    WorthQueryAdvancementDenial::Resource(cause) => {
                        InputCaptureStop::Resource(cause)
                    }
                    WorthQueryAdvancementDenial::Interrupted(stop) => {
                        InputCaptureStop::Interrupted(match stop {
                            WorthQueryManagedComputationInterruption::Cancelled => {
                                HandlerInterruption::Cancelled
                            }
                            WorthQueryManagedComputationInterruption::DeadlineExceeded => {
                                HandlerInterruption::DeadlineExceeded
                            }
                        })
                    }
                    cause => InputCaptureStop::Work(cause),
                })
            })
    }
}

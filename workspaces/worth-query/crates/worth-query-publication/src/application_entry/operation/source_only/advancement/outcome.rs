//! Move the one original Source owner into its actual returned advancement fate.
use super::*;
impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(super) fn into_advance(
        self,
        outcome: Result<Step, WorthQuerySourceOnlyDenial>,
    ) -> Advance<S, I, P, A, M> {
        match outcome {
            Ok(Step::Progress) => WorthQuerySourceOnlyAdvance::Progress(self),
            Ok(Step::Completed) => {
                WorthQuerySourceOnlyAdvance::Completed(WorthQueryCompletedSourceOperation {
                    operation: self,
                })
            }
            Ok(Step::Terminal) => {
                WorthQuerySourceOnlyAdvance::Terminal(WorthQueryTerminalSourceOperation {
                    operation: self,
                })
            }
            Err(denial) => WorthQuerySourceOnlyAdvance::Refused {
                operation: self,
                denial,
            },
        }
    }
}

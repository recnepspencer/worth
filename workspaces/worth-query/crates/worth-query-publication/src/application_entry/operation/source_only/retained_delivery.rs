//! Exact World history selection retains native recovered evidence on refusal.
use super::*;
impl<S, I, P, A, M> WorthQuerySourceOnlyOperation<S, I, P, A, M>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(super) fn deliver_retained(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        maximum_history_work: Option<std::num::NonZeroUsize>,
    ) -> Result<(), WorthQuerySourceOnlyDenial> {
        let SourceState::<S, I>::RetainedPending(evidence) = &self.state else {
            unreachable!("retained delivery owns recovered evidence")
        };
        let retained = match evidence {
            WorthQuerySourceCompletion::Recovered { receipt, .. } => {
                request.at_commit(phase, receipt, maximum_history_work)
            }
            WorthQuerySourceCompletion::Recorded { receipt, .. } => {
                request.at_original_receipt(phase, receipt, maximum_history_work)
            }
            WorthQuerySourceCompletion::Fresh { .. } => {
                unreachable!("fresh completion already retains its exact observation")
            }
        }
        .map_err(WorthQuerySourceOnlyDenial::History)?
        .retained_observation();
        let SourceState::<S, I>::RetainedPending(evidence) = &mut self.state else {
            unreachable!()
        };
        match evidence {
            WorthQuerySourceCompletion::Recovered {
                retained: destination,
                ..
            }
            | WorthQuerySourceCompletion::Recorded {
                retained: destination,
                ..
            } => *destination = Some(retained),
            WorthQuerySourceCompletion::Fresh { .. } => {
                unreachable!("fresh evidence has its observation")
            }
        }
        Ok(())
    }
}

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
        maximum_history_work: std::num::NonZeroUsize,
    ) -> Result<
        WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>,
        WorthQuerySourceOnlyDenial,
    > {
        let SourceState::<S, I>::RetainedPending(evidence) = &self.state else {
            unreachable!("retained delivery owns recovered evidence")
        };
        let WorthQuerySourceCompletion::Recovered { receipt, .. } = evidence else {
            unreachable!("fresh completion already retains its exact observation")
        };
        let retained = request
            .at_commit(phase, receipt, maximum_history_work)
            .map_err(WorthQuerySourceOnlyDenial::History)?
            .retained_observation();
        let SourceState::<S, I>::RetainedPending(mut evidence) =
            std::mem::replace(&mut self.state, SourceState::<S, I>::Initial)
        else {
            unreachable!()
        };
        if let WorthQuerySourceCompletion::Recovered {
            retained: destination,
            ..
        } = &mut evidence
        {
            *destination = Some(retained);
        }
        Ok(evidence)
    }
}

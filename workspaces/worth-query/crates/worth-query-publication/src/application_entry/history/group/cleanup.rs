//! An unresolved original key may advance one recovery step, never a new business attempt.
use super::*;

impl<'runtime, S, H, I, P, A> WorthQueryHistoryGroupExecution<'runtime, S, H, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub fn resume_cleanup<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        self.validate_document(document, request)?;
        if self.posture != WorthQueryHistoryGroupPosture::Aborting {
            return Err(WorthQueryHistoryGroupDenial::NotReady);
        }
        let Some(Member::Operation(Some(operation))) = self.members.get(self.closing) else {
            return Err(WorthQueryHistoryGroupDenial::NotReady);
        };
        // One source step in these postures can inspect/deliver the original
        // fate only. Recheck each call; no-effect must return to explicit close.
        if !Mode::<S, I, A>::can_resume_group_cleanup(operation) {
            return Err(WorthQueryHistoryGroupDenial::NotReady);
        }
        let Member::Operation(operation) = &mut self.members[self.closing] else {
            unreachable!()
        };
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::target(operation.as_ref().unwrap()),
        };
        match Mode::<S, I, A>::advance(operation.take().unwrap(), &current) {
            HistoryOperationAdvance::Progress(next) => *operation = Some(next),
            HistoryOperationAdvance::Refused {
                operation: next,
                denial,
            } => {
                *operation = Some(next);
                return Err(WorthQueryHistoryGroupDenial::Member(denial));
            }
            HistoryOperationAdvance::Completed(completed) => {
                self.members[self.closing] = Member::Completed(Some(completed))
            }
            HistoryOperationAdvance::Terminal(terminal) => {
                self.members[self.closing] = Member::Terminal(Some(terminal))
            }
        }
        Ok(())
    }
}

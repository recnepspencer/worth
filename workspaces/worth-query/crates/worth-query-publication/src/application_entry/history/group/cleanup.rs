//! An unresolved original key may advance one recovery step, never a new business attempt.
use super::*;

impl<'runtime, S, H, P, M> WorthQueryHistoryGroupExecution<'runtime, S, H, P, M>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
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
        if self.closing >= self.members.len() {
            return Err(WorthQueryHistoryGroupDenial::NotReady);
        }
        self.members
            .member_mut(self.closing)
            .resume_cleanup(request)?;
        Ok(())
    }
}

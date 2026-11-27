//! One actual owner release needs ownership, without disclosing model payload.
use super::*;
impl<S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'_, S, H, Payload, Outputs>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
{
    pub fn maintain_history(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryReleaseOutcome, WorthQueryApplicationHistoryDenial> {
        DocumentRequestContext::bind(&self.owner, request)?.advance(|phase| {
            self.owner
                .release_history_owner_after_output_cleanup(
                    phase,
                    &program_publication_access(),
                    |branch| self.outputs.release_for_branch(phase, branch),
                )
                .map_err(WorthQueryApplicationHistoryDenial::Document)?
                .map_err(WorthQueryApplicationHistoryDenial::Outputs)
        })
    }
}

//! Finish the installed supersession stop without a new request refusal.

use super::super::supersession::superseded_denial;

use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
impl DemandRegistryState {
    pub(in super::super) fn finish_superseded_row(
        &mut self,
        key: &WorthQueryOutputDemandKey,
        subject: &str,
    ) -> (
        WorthQueryOutputDemandDenial,
        Vec<Arc<WorthQueryOutputDemandKey>>,
    ) {
        let mut record = self
            .records
            .get_mut(key)
            .expect("superseded demand retains its owner record");
        match &record.state {
            DemandState::Failed(existing)
                if existing.kind() == WorthQueryOutputDemandDenialKind::Closed =>
            {
                return (existing.clone(), Vec::new());
            }
            DemandState::Output(output) => {
                if let super::WorthQueryOutputAdvancement::Stopped { denial, .. } =
                    &output.advancement()
                {
                    return (denial.clone(), Vec::new());
                }
            }
            _ => {}
        }
        let denial = superseded_denial(subject);
        let unpublished = record.unpublished_new_key();
        match &mut record.state {
            DemandState::Output(output) => output.stop(denial.clone()),
            _ => record.state = DemandState::Failed(denial.clone()),
        }
        record.performed_source = None;
        record.release_obligations();
        record.wake.notify();
        // The supersession stop is installed above; no World receipt is carried here.
        record.finish();
        if unpublished {
            // The row ends without an output of its own: the newest Ready it
            // replaced answers for the occurrence again.

            self.revive_replaced_ready(key);
        }

        let released_prerequisites = self.release_record_prerequisites(key);
        (denial, released_prerequisites)
    }
}

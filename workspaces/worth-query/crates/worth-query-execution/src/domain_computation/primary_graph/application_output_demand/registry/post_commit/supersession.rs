//! Finish the installed supersession stop without a new request refusal.
use super::super::refreshed_rejoin::infallible;
use super::super::supersession::superseded_denial;
use super::walk_charge::PostCommitWalk;
use super::*;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_superseded(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        subject: &str,
    ) -> WorthQueryOutputDemandDenial {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("superseded demand retains its owner record");
        match &record.state {
            DemandState::Failed(existing)
                if existing.kind() == WorthQueryOutputDemandDenialKind::Closed =>
            {
                return existing.clone();
            }
            DemandState::Output(output) => {
                if let super::WorthQueryOutputAdvancement::Stopped { denial, .. } =
                    &output.advancement
                {
                    return denial.clone();
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
        let released = record.release_obligations();
        record.wake.notify();
        // The supersession stop is installed above; no World receipt is carried here.
        if unpublished {
            // The row ends without an output of its own: the newest Ready it
            // replaced answers for the occurrence again.
            let mut payer = PostCommitWalk::after_effect(&state.records[&interest.key].state);
            infallible(state.revive_replaced_ready(&interest.key, &mut payer));
        }
        let mut payer = PostCommitWalk::after_effect(&state.records[&interest.key].state);
        let released_prerequisites =
            infallible(state.release_record_prerequisites(&interest.key, &mut payer));
        state.obligation_reserved_bytes = state.obligation_reserved_bytes.saturating_sub(released);
        let mut payer = PostCommitWalk::after_effect(&state.records[&interest.key].state);
        infallible(state.remove_required_member_if_released(&interest.key, &mut payer));
        drop(state);
        drop(released_prerequisites);
        denial
    }
}

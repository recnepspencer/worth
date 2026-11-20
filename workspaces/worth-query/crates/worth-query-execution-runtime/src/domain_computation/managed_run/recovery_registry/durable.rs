//! Live slots refer weakly to their actual journal; Drop queues funded disposition.
use super::*;
use worth_query_execution_contracts::facade::{QueryDispatchJournalDenial, QueryRecordedRecoveryClaim};
use worth_foundational::facade::DurableRecoveryClaimIdentity;

#[derive(Clone, Copy, Debug)]
pub struct DeferredRecoveryDisposition {
    pub recorded: QueryRecordedRecoveryClaim,
    pub terminal: RecoveryClaimDisposition,
}
impl WorthQueryRecoveryHandleRegistry {
    pub fn register_durable_once(
        &self,
        claim: WorthQueryRecoveryMintClaim,
        recorded: QueryRecordedRecoveryClaim,
    ) -> Result<WorthQueryRecoveryRegistrySlot, WorthQueryRecoveryMintAlreadyClaimed> {
        let mut state = self.state.lock().expect("recovery registry lock");
        if !state.claimed_commits.insert(claim.clone()) {
            return Err(WorthQueryRecoveryMintAlreadyClaimed);
        }
        let slot = WorthQueryRecoveryRegistrySlot(state.next_slot);
        state.next_slot = state.next_slot.saturating_add(1);
        state.claims_by_slot.insert(slot, claim);
        state.live.insert(slot, ());
        state.durable.insert(slot, recorded);
        Ok(slot)
    }
    pub fn publish_terminal(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
        terminal: RecoveryClaimDisposition,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        let recorded = self
            .state
            .lock()
            .expect("recovery registry lock")
            .durable
            .get(&slot)
            .copied();
        let Some(recorded) = recorded else {
            return Ok(());
        };
        let journal = self
            .journal
            .upgrade()
            .ok_or(QueryDispatchJournalDenial::OwnerUnavailable)?;
        journal.dispose_recovery_claim(
            &recorded.claim.original,
            recorded.identity,
            terminal,
            request,
        )
    }
    pub fn deferred_dispositions(
        &self,
    ) -> Vec<DeferredRecoveryDisposition> {
        self.state
            .lock()
            .expect("recovery registry lock")
            .deferred
            .values()
            .copied()
            .collect()
    }
    pub fn acknowledge_deferred(
        &self,
        claim: DurableRecoveryClaimIdentity,
    ) {
        self.state
            .lock()
            .expect("recovery registry lock")
            .deferred
            .remove(&claim);
    }
    pub fn journal_claim_state(
        &self,
        claim: &WorthQueryRecoveryMintClaim,
    ) -> Result<Option<(bool, Option<RecoveryClaimDisposition>)>, QueryDispatchJournalDenial> {
        let WorthQueryRecoveryMintClaim::Home(original) = claim else {
            return Ok(None);
        };
        let journal = self
            .journal
            .upgrade()
            .ok_or(QueryDispatchJournalDenial::OwnerUnavailable)?;
        let recorded = journal.recorded_recovery_claim(original)?;
        Ok(Some(match recorded {
            Some(recorded) => (true, recorded.terminal),
            None => (false, None),
        }))
    }
    pub fn recorded_terminal(
        &self,
        slot: WorthQueryRecoveryRegistrySlot,
    ) -> Result<Option<RecoveryClaimDisposition>, QueryDispatchJournalDenial> {
        let recorded = self
            .state
            .lock()
            .expect("recovery registry lock")
            .durable
            .get(&slot)
            .copied();
        let Some(recorded) = recorded else {
            return Ok(None);
        };
        let journal = self
            .journal
            .upgrade()
            .ok_or(QueryDispatchJournalDenial::OwnerUnavailable)?;
        let current = journal
            .recorded_recovery_claim(&recorded.claim.original)?
            .ok_or(QueryDispatchJournalDenial::MalformedFrame)?;
        if current.identity != recorded.identity {
            return Err(QueryDispatchJournalDenial::StaleIncarnation);
        }
        Ok(current.terminal)
    }
}

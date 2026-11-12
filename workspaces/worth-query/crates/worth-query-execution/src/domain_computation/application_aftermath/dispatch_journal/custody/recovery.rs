//! One funded original driver serializes claim acquisition and disposition.
use super::*;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableRecoveryClaimIdentity, RecoveryClaimDisposition,
};

pub(in crate::domain_computation) struct QueryRecoveryJournalAdmission<'a> {
    journal: &'a WorthQueryDispatchJournal,
    driver: DispatchJournalDriver<'a>,
}
impl WorthQueryDispatchJournal {
    pub(in crate::domain_computation) fn begin_recovery_admission(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<QueryRecoveryJournalAdmission<'_>, QueryDispatchJournalDenial> {
        let driver = DispatchJournalDriver::lock(self, request)?;
        let medium = self.installed_medium()?;
        driver.settle(medium.as_ref(), request)?;
        Ok(QueryRecoveryJournalAdmission {
            journal: self,
            driver,
        })
    }
    pub(in crate::domain_computation) fn recorded_recovery_claim(
        &self,
        original: &DurableCompositeCommitIdentity,
    ) -> Result<Option<QueryRecordedRecoveryClaim>, QueryDispatchJournalDenial> {
        let medium = self.installed_medium()?;
        if original.owner().home() != medium.home() {
            return Err(QueryDispatchJournalDenial::ForeignHome);
        }
        let state = self.state();
        let state = state
            .as_ref()
            .ok_or(QueryDispatchJournalDenial::JournalUnavailable)?;
        if state.pending.is_some() {
            return Err(QueryDispatchJournalDenial::PublicationPending);
        }
        Ok(state.ledger.recovery.claim(original))
    }
    pub(in crate::domain_computation) fn recorded_recovery_claims(
        &self,
    ) -> Result<Vec<QueryRecordedRecoveryClaim>, QueryDispatchJournalDenial> {
        self.installed_medium()?;
        let state = self.state();
        let state = state
            .as_ref()
            .ok_or(QueryDispatchJournalDenial::JournalUnavailable)?;
        if state.pending.is_some() {
            return Err(QueryDispatchJournalDenial::PublicationPending);
        }
        Ok(state.ledger.recovery.claims().collect())
    }
}
impl QueryRecoveryJournalAdmission<'_> {
    pub(in crate::domain_computation) fn claim(
        &self,
        original: &DurableCompositeCommitIdentity,
    ) -> Result<Option<QueryRecordedRecoveryClaim>, QueryDispatchJournalDenial> {
        self.journal.recorded_recovery_claim(original)
    }
    pub(in crate::domain_computation) fn acquire(
        &self,
        claim: QueryRecoveryClaim,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<QueryRecordedRecoveryClaim, QueryDispatchJournalDenial> {
        let medium = self.journal.installed_medium()?;
        self.driver.publish(
            medium.as_ref(),
            QueryDispatchRecordIntent::Recovery(QueryRecoveryRecord::Claim(claim)),
            request,
        )?;
        self.claim(&claim.original)?
            .ok_or(QueryDispatchJournalDenial::OwnerUnavailable)
    }
    pub(in crate::domain_computation) fn dispose(
        &self,
        original: &DurableCompositeCommitIdentity,
        claim: DurableRecoveryClaimIdentity,
        terminal: RecoveryClaimDisposition,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        let current = self
            .claim(original)?
            .ok_or(QueryDispatchJournalDenial::MalformedFrame)?;
        if current.identity != claim {
            return Err(QueryDispatchJournalDenial::StaleIncarnation);
        }
        if let Some(recorded) = current.terminal {
            return if recorded == terminal {
                Ok(())
            } else {
                Err(QueryDispatchJournalDenial::MalformedFrame)
            };
        }
        let medium = self.journal.installed_medium()?;
        self.driver
            .publish(
                medium.as_ref(),
                QueryDispatchRecordIntent::Recovery(QueryRecoveryRecord::Disposition {
                    claim,
                    terminal,
                }),
                request,
            )
            .map(|_| ())
    }
}

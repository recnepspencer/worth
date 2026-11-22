//! Request-aware serialization of recorded acceptance and live-custody cutover.
use super::*;

pub struct QueryInboundJournalAdmission<'a> {
    pub journal: &'a WorthQueryDispatchJournal,
    pub driver: DispatchJournalDriver<'a>,
}
impl WorthQueryDispatchJournal {
    pub fn begin_inbound_admission(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<QueryInboundJournalAdmission<'_>, QueryDispatchJournalDenial> {
        let driver = DispatchJournalDriver::lock(self, request)?;
        if let Some(medium) = self.medium.get() {
            driver.settle(medium.as_ref(), request)?;
        }
        Ok(QueryInboundJournalAdmission {
            journal: self,
            driver,
        })
    }
}
impl QueryInboundJournalAdmission<'_> {
    pub fn next_message_after(&self, position: u64) -> Option<QueryInboundRecordedMessage> {
        self.journal
            .state()
            .as_ref()?
            .ledger
            .inbound
            .next_after(position)
    }
    pub fn message(&self, key: &QueryInboundMessageKey) -> Option<QueryInboundRecordedMessage> {
        self.journal
            .state()
            .as_ref()?
            .ledger
            .inbound
            .messages()
            .get(key)
            .cloned()
    }
    pub fn acknowledge(
        &self,
        key: QueryInboundMessageKey,
        terminal: &crate::domain_computation::primary_graph::provider::WorthQueryCanonicalInboundCompletion,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        if !self.journal.is_durable() {
            return Ok(());
        }
        let recorded = self
            .message(&key)
            .ok_or(QueryDispatchJournalDenial::MalformedFrame)?;
        let home = self.journal.installed_medium()?.home();
        if !terminal.matches_recorded_message(&recorded.accepted, home) {
            return Err(QueryDispatchJournalDenial::MalformedFrame);
        }
        let completion_world = terminal.completion_world_commit().ordinal();
        if let Some(existing) = recorded.acknowledgement {
            return if existing == completion_world {
                Ok(())
            } else {
                Err(QueryDispatchJournalDenial::MalformedFrame)
            };
        }
        self.publish(
            QueryInboundRecord::Acknowledged {
                message: key,
                completion_world,
            },
            request,
        )
    }
    pub fn expire(
        &self,
        key: QueryInboundMessageKey,
        terminal: &crate::domain_computation::primary_graph::provider::WorthQueryCanonicalInboundCompletion,
        now_unix_seconds: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        let Some(recorded) = self.message(&key) else {
            return Ok(());
        };
        if recorded.accepted.claims.expires_at_unix_seconds >= now_unix_seconds
            || recorded.acknowledgement != Some(terminal.completion_world_commit().ordinal())
            || !terminal.matches_recorded_message(
                &recorded.accepted,
                self.journal.installed_medium()?.home(),
            )
        {
            return Err(QueryDispatchJournalDenial::MalformedFrame);
        }
        self.publish(QueryInboundRecord::Expired { message: key }, request)
    }
    pub fn publish(
        &self,
        record: QueryInboundRecord,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial> {
        let Some(medium) = self.journal.medium.get() else {
            return Ok(());
        };
        self.driver
            .publish(
                medium.as_ref(),
                QueryDispatchRecordIntent::Inbound(record),
                request,
            )
            .map(drop)
    }
}

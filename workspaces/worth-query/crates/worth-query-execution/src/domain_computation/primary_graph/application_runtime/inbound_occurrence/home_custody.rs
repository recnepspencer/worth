//! Synchronize committed acceptances before any live admission decision.
use super::*;
use crate::domain_computation::application_aftermath::dispatch_journal::QueryInboundJournalAdmission;
use crate::domain_computation::application_aftermath::WorthQueryInboundCustodyAdmission as Admission;
use receive::WorthQueryInboundAdmissionDenial as Denial;
mod dispatch_readmission;
mod message_readmission;
mod terminal_sources;
mod unpublished;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn synchronize_durable_inbound(
        &self,
        driver: &QueryInboundJournalAdmission<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        loop {
            let cursor = self
                .runtime
                .inbound_custody
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .durable_admission_cursor;
            let Some(recorded) = driver.next_message_after(cursor) else {
                return Ok(());
            };
            let owner = self.read_recorded_inbound_owner(execution, &recorded.accepted)?;
            self.restore_recorded_inbound(driver, execution, &recorded, owner)?;
            self.runtime
                .inbound_custody
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .durable_admission_cursor = recorded.acceptance_position;
        }
    }
}

pub(super) fn recorded_admission_denial(admission: Admission) -> Denial {
    match admission {
        Admission::CapacityExhausted => Denial::CapacityExhausted,
        Admission::MessageIdentityConflict => Denial::MessageIdentityConflict,
        Admission::CorrelationAlreadyOwned => Denial::CorrelationAlreadyOwned,
        Admission::New(_, _) | Admission::Duplicate(_) | Admission::CompactTerminalReplay(_) => {
            Denial::RetryBeforeAcceptance
        }
    }
}

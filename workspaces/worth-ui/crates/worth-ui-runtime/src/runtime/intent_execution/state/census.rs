use super::{UiIntentExecutionSlotPhase, UiIntentExecutionState};

pub(crate) struct UiIntentExecutionAdmissionCensus {
    pub(crate) execution_entries: usize,
    pub(crate) active_attempts: usize,
    pub(crate) active_occupancy: usize,
    pub(crate) retained_candidates: usize,
    pub(crate) retained_payloads: usize,
    pub(crate) retained_owner_references: usize,
    pub(crate) retained_payload_bytes: usize,
    pub(crate) prepared_attempts: usize,
    pub(crate) running_attempts: usize,
    pub(crate) recovering_attempts: usize,
    pub(crate) consequence_pending_attempts: usize,
}

impl UiIntentExecutionState {
    pub(crate) fn census(&self) -> UiIntentExecutionAdmissionCensus {
        let mut census = UiIntentExecutionAdmissionCensus {
            execution_entries: 0,
            active_attempts: 0,
            active_occupancy: self.occupancy.active_count(),
            retained_candidates: 0,
            retained_payloads: 0,
            retained_owner_references: 0,
            retained_payload_bytes: 0,
            prepared_attempts: 0,
            running_attempts: 0,
            recovering_attempts: 0,
            consequence_pending_attempts: 0,
        };
        for phase in self.slots.iter().filter_map(|slot| slot.phase.as_ref()) {
            census.execution_entries += 1;
            match phase {
                UiIntentExecutionSlotPhase::ConsequencePending(settled) => {
                    let _terminal_identity = (settled.attempt, settled.idempotency);
                    let _outcome_schema = settled.outcome.schema();
                    census.consequence_pending_attempts += 1;
                    continue;
                }
                UiIntentExecutionSlotPhase::ConsequenceReady(prepared) => {
                    let _terminal_identity = (prepared.attempt, prepared.idempotency);
                    census.consequence_pending_attempts += 1;
                    continue;
                }
                UiIntentExecutionSlotPhase::ConsequenceHandoff(marker) => {
                    let _terminal_identity = (marker.attempt, marker.idempotency);
                    census.consequence_pending_attempts += 1;
                    continue;
                }
                UiIntentExecutionSlotPhase::Admitted(_)
                | UiIntentExecutionSlotPhase::AttemptPrepared(_)
                | UiIntentExecutionSlotPhase::Running(_)
                | UiIntentExecutionSlotPhase::Recovery(_) => {}
            }
            let Some(reservation) = phase.reservation() else {
                continue;
            };
            census.active_attempts += 1;
            census.retained_candidates +=
                usize::from(matches!(phase, UiIntentExecutionSlotPhase::Admitted(_)));
            census.retained_payloads += reservation.retained_payloads;
            census.retained_owner_references += reservation.retained_owner_references;
            census.retained_payload_bytes += reservation.basis.retained_payload_bytes();
            census.prepared_attempts += usize::from(matches!(
                phase,
                UiIntentExecutionSlotPhase::AttemptPrepared(_)
            ));
            census.running_attempts +=
                usize::from(matches!(phase, UiIntentExecutionSlotPhase::Running(_)));
            census.recovering_attempts +=
                usize::from(matches!(phase, UiIntentExecutionSlotPhase::Recovery(_)));
        }
        census
    }
}

use super::{
    WorthServerScheduledOperationOutcome, WorthServerSchedulerCancellationPosture,
    WorthServerSchedulerFailurePosture,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthServerScheduledOperationTraceEntry {
    slot_ordinal: usize,
    scheduler_lane: String,
    execution_digest: Result<
        Option<String>,
        worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
    >,
    failure_posture: Option<WorthServerSchedulerFailurePosture>,
    cancellation_posture: Option<WorthServerSchedulerCancellationPosture>,
}

impl WorthServerScheduledOperationTraceEntry {
    pub(crate) fn from_outcome(outcome: &WorthServerScheduledOperationOutcome) -> Self {
        Self {
            slot_ordinal: outcome.slot().ordinal(),
            scheduler_lane: outcome.slot().scheduler_lane(),
            execution_digest: outcome
                .execution_digest()
                .map(|digest| digest.map(str::to_string)),
            failure_posture: outcome.failure_posture().cloned(),
            cancellation_posture: outcome.cancellation_posture(),
        }
    }

    pub fn slot_ordinal(&self) -> usize {
        self.slot_ordinal
    }

    pub fn scheduler_lane(&self) -> &str {
        &self.scheduler_lane
    }

    pub fn execution_digest(
        &self,
    ) -> Result<
        Option<&str>,
        worth_query::facade::runtime::WorthQueryEvidenceIdentityDerivationDenial,
    > {
        self.execution_digest
            .as_ref()
            .map(|digest| digest.as_deref())
            .map_err(|cause| *cause)
    }

    pub fn failure_posture(&self) -> Option<&WorthServerSchedulerFailurePosture> {
        self.failure_posture.as_ref()
    }

    pub fn cancellation_posture(&self) -> Option<WorthServerSchedulerCancellationPosture> {
        self.cancellation_posture
    }
}

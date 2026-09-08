use worth_proof::TransitionOutcome;
use worth_store_physical_format::PhysicalArtifactReadRange;

use super::{CanonicalRecordReadPort, RecordReadPartition};
use crate::physical_runtime::{
    PhysicalExecutorCommand, PhysicalReadWorkRequest, PhysicalSchedulerDemand,
    PhysicalWorkScheduler, PhysicalWorkScope, PhysicalWorkSettlementEvidence,
};

#[derive(Debug, Clone, Copy)]
pub enum PhysicalIntegrityScrubReadDeferral {
    RuntimeReleased,
    Submission,
    DependencyBlocked,
    PreEffect(crate::physical_runtime::PhysicalWorkPreEffectDenial),
    Capacity(
        worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial,
    ),
    Pacing(worth_store_io_scheduler::BackgroundPacingDenial),
    PacingDeferred,
    Scheduler(crate::physical_runtime::PhysicalSchedulerDenial),
    Command(crate::physical_runtime::PhysicalExecutorCommandDenial),
}

use PhysicalIntegrityScrubReadDeferral as ScrubReadDeferral;

impl CanonicalRecordReadPort {
    /// Acquires diagnostic bytes through the ordinary C5.1 submission, Signal,
    /// scheduler, C4, and settlement chain. It never obtains projection-failure
    /// authority: observed corruption is not a serving-state transition.
    pub(in crate::physical_runtime) fn inspect(
        &self,
        range: PhysicalArtifactReadRange,
        destination: Box<[u8]>,
    ) -> Result<PhysicalWorkSettlementEvidence, ScrubReadDeferral> {
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(ScrubReadDeferral::RuntimeReleased)?;
        let request = PhysicalReadWorkRequest::new(
            PhysicalWorkScope::inspection(range),
            self.record.read_basis(RecordReadPartition::Artifact),
            self.record.security(),
        )
        .map_err(|_| ScrubReadDeferral::Submission)?;
        let receipt = match self.submission.submit(request).into_raw() {
            TransitionOutcome::Success(receipt) => receipt,
            _ => return Err(ScrubReadDeferral::Submission),
        };
        let (ready, _) =
            super::scheduler_preparation::admit_ready(&runtime, receipt, &self.physical).map_err(
                |failure| match failure.failure() {
                    super::CanonicalRecordReadFailure::PreEffect(cause) => {
                        ScrubReadDeferral::PreEffect(cause)
                    }
                    _ => ScrubReadDeferral::DependencyBlocked,
                },
            )?;
        let (lease, capacity, backend, policy) = self.scheduler.scrub_background(
            self.record.scheduler_security(), range.length() as u64,
        ).map_err(|failure| {
            use crate::physical_runtime::instance::PhysicalScrubSchedulerAdmissionDenial as Denial;
            match failure {
                Denial::Capacity(cause) => ScrubReadDeferral::Capacity(cause),
                Denial::Pacing(cause) => ScrubReadDeferral::Pacing(cause),
                Denial::Deferred => ScrubReadDeferral::PacingDeferred,
            }
        })?;
        let demand = PhysicalSchedulerDemand::scrub_background(ready, lease, capacity)
            .map_err(ScrubReadDeferral::Scheduler)?;
        let work = PhysicalWorkScheduler::admit(demand, &backend, policy)
            .map_err(ScrubReadDeferral::Scheduler)?;
        let command = PhysicalExecutorCommand::inspection(work, destination)
            .map_err(ScrubReadDeferral::Command)?;
        drop(runtime);
        self.execution
            .execute_physical_work(command)
            .map(|outcome| outcome.into_settled().into_evidence())
            .map_err(ScrubReadDeferral::PreEffect)
    }
}

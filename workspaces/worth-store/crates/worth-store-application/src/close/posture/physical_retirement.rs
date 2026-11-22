//! The original physical close task and terminal outcome retain their custody.
use super::*;
enum PhysicalReleaseStep {
    Pending,
    Closed(Box<worth_store::physical_runtime::ServingShutdownOutcome<worth_store::physical_runtime::ClosedRuntime>>),
    Refused(ApplicationHomeCloseDenial),
}

impl ApplicationHomeRecoveryCustody {
    #[inline(never)]
    pub(super) fn finish_physical_release(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
        start: PhysicalCloseStart,
    ) -> ApplicationHomeCloseOutcome {
        if let Err(denial) = self.start_physical_close_if_retained(execution, start) {
            return self.refused(denial);
        }
        loop {
            match self.poll_physical_close_step() {
                PhysicalReleaseStep::Pending => {}
                PhysicalReleaseStep::Closed(shutdown) => return self.into_closed(*shutdown),
                PhysicalReleaseStep::Refused(denial) => return self.refused(denial),
            }
            if let Err(cause) = interrupted(execution) {
                return self.refused(ApplicationHomeCloseDenial::Execution(cause));
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    #[inline(never)]
    fn start_physical_close_if_retained(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        start: PhysicalCloseStart,
    ) -> Result<(), ApplicationHomeCloseDenial> {
        if !matches!(self.inner.physical, Some(PhysicalCloseCustody::Retained(_))) {
            return Ok(());
        }
        let PhysicalCloseCustody::Retained(serving) =
            self.inner.physical.take().expect("live physical custody")
        else {
            unreachable!("checked retained posture above")
        };
        if let Err(cause) = interrupted(execution) {
            self.inner.physical = Some(PhysicalCloseCustody::Retained(serving));
            return Err(ApplicationHomeCloseDenial::Execution(cause));
        }
        let serving = match Arc::try_unwrap(serving) {
            Ok(serving) => serving,
            Err(serving) => {
                let holders = super::super::physical_holders::observe(
                    &serving,
                    self.inner.signal_medium.as_ref(),
                );
                self.inner.physical = Some(PhysicalCloseCustody::Retained(serving));
                return Err(ApplicationHomeCloseDenial::PhysicalCustodyRetained(holders));
            }
        };
        match start.start(serving) {
            Ok(task) => {
                self.inner.physical = Some(PhysicalCloseCustody::Running(Box::new(task)));
                Ok(())
            }
            Err(refused) => {
                let (serving, error) = refused.into_parts();
                self.inner.physical = Some(PhysicalCloseCustody::Retained(Arc::new(serving)));
                Err(ApplicationHomeCloseDenial::PhysicalCloseStartFailed {
                    kind: error.kind(),
                    detail: error.to_string(),
                })
            }
        }
    }

    #[inline(never)]
    fn poll_physical_close_step(&mut self) -> PhysicalReleaseStep {
        match self.inner.physical.take().expect("live physical custody") {
            PhysicalCloseCustody::Running(task) => match (*task).poll() {
                PhysicalStoreCloseTaskPoll::Pending(task) => {
                    self.inner.physical = Some(PhysicalCloseCustody::Running(Box::new(task)));
                    PhysicalReleaseStep::Pending
                }
                PhysicalStoreCloseTaskPoll::Failed(failure) => {
                    let cause = *failure.cause();
                    self.inner.physical = Some(PhysicalCloseCustody::Failed(Box::new(failure)));
                    PhysicalReleaseStep::Refused(ApplicationHomeCloseDenial::PhysicalCloseFailed(
                        cause,
                    ))
                }
                PhysicalStoreCloseTaskPoll::Completed(outcome) => match outcome {
                    PhysicalStoreCloseOutcome::Closed { shutdown, .. } => {
                        PhysicalReleaseStep::Closed(Box::new(shutdown))
                    }
                    outcome @ PhysicalStoreCloseOutcome::InspectionRequired { .. } => {
                        self.inner.physical =
                            Some(PhysicalCloseCustody::InspectionRequired(Box::new(outcome)));
                        PhysicalReleaseStep::Refused(
                            ApplicationHomeCloseDenial::PhysicalInspectionRequired,
                        )
                    }
                },
            },
            PhysicalCloseCustody::InspectionRequired(outcome) => {
                self.inner.physical = Some(PhysicalCloseCustody::InspectionRequired(outcome));
                PhysicalReleaseStep::Refused(ApplicationHomeCloseDenial::PhysicalInspectionRequired)
            }
            PhysicalCloseCustody::Failed(failure) => {
                let cause = *failure.cause();
                self.inner.physical = Some(PhysicalCloseCustody::Failed(failure));
                PhysicalReleaseStep::Refused(ApplicationHomeCloseDenial::PhysicalCloseFailed(cause))
            }
            PhysicalCloseCustody::Retained(_) => {
                unreachable!("retained physical owner was classified before waiting")
            }
        }
    }

    #[inline(never)]
    fn into_closed(
        self,
        shutdown: worth_store::physical_runtime::ServingShutdownOutcome<
            worth_store::physical_runtime::ClosedRuntime,
        >,
    ) -> ApplicationHomeCloseOutcome {
        let body = *self.inner;
        let allocation = super::super::RetiredApplicationHomeAllocation {
            _location: body.location,
            _descriptor: body.descriptor,
            _world: body.world_composition.map(|composition| {
                composition
                    .into_retired()
                    .unwrap_or_else(|_| unreachable!("physical close follows actual Bridge retirement"))
            }),
            _signal: body.signal_owner_retirement,
            _world_owner: body.world_owner.into_retired(),
            _world_origin: body.world_origin_retirement,
            _world_report: body.world_close_report,
            _checkpoint: body.checkpoint_owner,
            _publications: body.world_publications,
            _native: match body.native {
                NativeCloseCustody::Retired(original) => Some(original),
                NativeCloseCustody::NeverAttached => None,
                NativeCloseCustody::Serving(_) | NativeCloseCustody::Prepared(_) => {
                    unreachable!("physical close follows complete original Native retirement")
                }
            },
        };
        ApplicationHomeCloseOutcome::Closed {
            store: shutdown,
            checkpoint: body.checkpoint.into_result(),
            allocation,
        }
    }
}

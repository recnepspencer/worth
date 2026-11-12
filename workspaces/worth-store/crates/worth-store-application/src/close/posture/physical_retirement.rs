//! The original physical close task and terminal outcome retain their custody.
use super::*;
impl ApplicationHomeRecoveryCustody {
    pub(super) fn finish_physical_release(
        mut self,
        execution: &ExecutionRequest<'_, '_>,
        start: PhysicalCloseStart,
    ) -> ApplicationHomeCloseOutcome {
        if let PhysicalCloseCustody::Retained(serving) = self.physical {
            if let Err(cause) = interrupted(execution) {
                return Self {
                    physical: PhysicalCloseCustody::Retained(serving),
                    ..self
                }
                .refused(ApplicationHomeCloseDenial::Execution(cause));
            }
            let serving = match Arc::try_unwrap(serving) {
                Ok(serving) => serving,
                Err(serving) => {
                    let holders = super::super::physical_holders::observe(
                        &serving,
                        self.signal_medium.as_ref(),
                    );
                    return Self {
                        physical: PhysicalCloseCustody::Retained(serving),
                        ..self
                    }
                    .refused(ApplicationHomeCloseDenial::PhysicalCustodyRetained(holders));
                }
            };
            self.physical = match start.start(serving) {
                Ok(task) => PhysicalCloseCustody::Running(task),
                Err(refused) => {
                    let (serving, error) = refused.into_parts();
                    return Self {
                        physical: PhysicalCloseCustody::Retained(Arc::new(serving)),
                        ..self
                    }
                    .refused(
                        ApplicationHomeCloseDenial::PhysicalCloseStartFailed {
                            kind: error.kind(),
                            detail: error.to_string(),
                        },
                    );
                }
            };
        }
        loop {
            self.physical = match self.physical {
                PhysicalCloseCustody::Running(task) => match task.poll() {
                    PhysicalStoreCloseTaskPoll::Pending(task) => {
                        PhysicalCloseCustody::Running(task)
                    }
                    PhysicalStoreCloseTaskPoll::Failed(failure) => {
                        let cause = *failure.cause();
                        return Self {
                            physical: PhysicalCloseCustody::Failed(failure),
                            ..self
                        }
                        .refused(ApplicationHomeCloseDenial::PhysicalCloseFailed(cause));
                    }
                    PhysicalStoreCloseTaskPoll::Completed(outcome) => match outcome {
                        PhysicalStoreCloseOutcome::Closed { shutdown, .. } => {
                            let allocation = super::super::RetiredApplicationHomeAllocation {
                                _location: self.location,
                                _descriptor: self.descriptor,
                                _world: self.world_composition.map(|composition| {
                                    composition.into_retired().unwrap_or_else(|_| {
                                        unreachable!(
                                            "physical close follows actual Bridge retirement"
                                        )
                                    })
                                }),
                                _signal: self.signal_owner_retirement,
                                _world_owner: self.world_owner.into_retired(),
                                _world_origin: self.world_origin_retirement,
                                _world_report: self.world_close_report,
                                _checkpoint: self.checkpoint_owner,
                                _publications: self.world_publications,
                                _native: match self.native {
                                    NativeCloseCustody::Retired(original) => Some(original),
                                    NativeCloseCustody::NeverAttached => None,
                                    NativeCloseCustody::Serving(_) | NativeCloseCustody::Prepared(_) => unreachable!("physical close follows complete original Native retirement"),
                                },
                            };
                            return ApplicationHomeCloseOutcome::Closed {
                                store: shutdown,
                                checkpoint: self.checkpoint.into_result(),
                                allocation,
                            };
                        }
                        outcome @ PhysicalStoreCloseOutcome::InspectionRequired { .. } => {
                            return Self {
                                physical: PhysicalCloseCustody::InspectionRequired(outcome),
                                ..self
                            }
                            .refused(ApplicationHomeCloseDenial::PhysicalInspectionRequired);
                        }
                    },
                },
                PhysicalCloseCustody::InspectionRequired(outcome) => {
                    return Self {
                        physical: PhysicalCloseCustody::InspectionRequired(outcome),
                        ..self
                    }
                    .refused(ApplicationHomeCloseDenial::PhysicalInspectionRequired);
                }
                PhysicalCloseCustody::Failed(failure) => {
                    let cause = *failure.cause();
                    return Self {
                        physical: PhysicalCloseCustody::Failed(failure),
                        ..self
                    }
                    .refused(ApplicationHomeCloseDenial::PhysicalCloseFailed(cause));
                }
                PhysicalCloseCustody::Retained(_) => {
                    unreachable!("retained physical owner was classified before waiting")
                }
            };
            if let Err(cause) = interrupted(execution) {
                return self.refused(ApplicationHomeCloseDenial::Execution(cause));
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

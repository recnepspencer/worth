use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::ArtifactGenerationRollbackFailureCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::PanicDropCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::PanicProbeAndDropCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::PanicProbeCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::RestoreExecutionDropPanicCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::RestoreFailureCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::RestorePanicAfterAdmissionCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::RestorePanicCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::RestoreRejectAfterAdmissionCheckpoint;
use crate::domain_computation::managed_run::tests::yield_checkpoint_fixture::YieldCheckpoint;
use crate::domain_computation::managed_run::tests::*;

#[derive(Clone)]
pub enum YieldSuspension {
    Checkpoint {
        retained_bytes: u64,
    },
    CheckpointProbePanic,
    CheckpointDropPanic {
        retained_bytes: u64,
    },
    CheckpointMemoryMismatch {
        governed_retained_bytes: u64,
        reported_retained_bytes: u64,
        drop_panics: bool,
    },
    CheckpointProbeAndDropPanic,
    CheckpointRestoreFailure {
        retained_bytes: u64,
    },
    CheckpointRestorePanic {
        retained_bytes: u64,
    },
    CheckpointRestoreRejectAfterAdmission {
        retained_bytes: u64,
    },
    CheckpointRestorePanicAfterAdmission {
        retained_bytes: u64,
    },
    CheckpointRestoreExecutionDropPanic {
        retained_bytes: u64,
        checkpoint_drop_panics: bool,
    },
    CheckpointArtifactGenerationRollbackFailure {
        retained_bytes: u64,
        registry: Arc<
            std::sync::Mutex<
                Option<
                    Arc<
                        worth_query_execution::facade::domain_computation::WorthQueryWorkflowArtifactRegistry,
                    >,
                >,
            >,
        >,
    },
    Failure,
    Panic,
}

impl YieldSuspension {
    pub fn governed_retained_bytes(&self) -> usize {
        let reported = match self {
            Self::Checkpoint { retained_bytes }
            | Self::CheckpointDropPanic { retained_bytes }
            | Self::CheckpointRestoreFailure { retained_bytes }
            | Self::CheckpointRestorePanic { retained_bytes }
            | Self::CheckpointRestoreRejectAfterAdmission { retained_bytes }
            | Self::CheckpointRestorePanicAfterAdmission { retained_bytes }
            | Self::CheckpointRestoreExecutionDropPanic { retained_bytes, .. }
            | Self::CheckpointArtifactGenerationRollbackFailure { retained_bytes, .. } => {
                *retained_bytes
            }
            Self::CheckpointMemoryMismatch {
                governed_retained_bytes,
                ..
            } => *governed_retained_bytes,
            Self::CheckpointProbePanic
            | Self::CheckpointProbeAndDropPanic
            | Self::Failure
            | Self::Panic => 3,
        };
        usize::try_from(reported).unwrap()
    }
}

#[derive(Clone)]
pub struct YieldProvider {
    pub yield_installed: bool,
    pub checkpoint_available: bool,
    pub record_effect: bool,
    pub suspension: YieldSuspension,
    pub execution_drop_panics: bool,
}

pub struct YieldExecution {
    pub step_ordinal: u8,
    pub checkpoint_available: bool,
    pub record_effect: bool,
    pub suspension: YieldSuspension,
    pub execution_drop_panics: bool,
    pub retained: Option<WorthQueryGraphProviderRetainedMemory>,
}

impl YieldExecution {
    pub fn restored(retained: WorthQueryGraphProviderRetainedMemory) -> Self {
        Self {
            step_ordinal: 1,
            checkpoint_available: false,
            record_effect: false,
            suspension: YieldSuspension::Failure,
            execution_drop_panics: false,
            retained: Some(retained),
        }
    }

    pub fn restored_with_drop_panic(retained: WorthQueryGraphProviderRetainedMemory) -> Self {
        Self {
            step_ordinal: 1,
            checkpoint_available: false,
            record_effect: false,
            suspension: YieldSuspension::Failure,
            execution_drop_panics: true,
            retained: Some(retained),
        }
    }
}

impl Drop for YieldExecution {
    fn drop(&mut self) {
        if self.execution_drop_panics {
            panic!("yield fixture provider execution destructor panicked")
        }
    }
}

impl WorthQueryGraphProviderExecution for YieldExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        for _ in 0..2 {
            step.perform_work_unit(|| Ok(()))?;
        }
        if self.step_ordinal == 0 {
            if self.record_effect {
                step.apply_effect(|| Ok(()))?;
            }
            self.retained = Some(
                step.retain_bytes(self.suspension.governed_retained_bytes())
                    .map_err(step_failure)?,
            );
            if self.checkpoint_available {
                step.record_checkpoint_available().map_err(step_failure)?;
            }
            self.step_ordinal = 1;
            return Ok(WorthQueryGraphProviderStepDisposition::continue_work());
        }
        WorthQueryGraphProviderStepDisposition::complete("yield-fixture-complete")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn suspend(
        &mut self,
    ) -> Result<
        Box<dyn worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint>,
        WorthQueryGraphProviderFailure,
    >{
        match self.suspension.clone() {
            YieldSuspension::Checkpoint { retained_bytes } => Ok(Box::new(YieldCheckpoint {
                retained_bytes,
                retained: self.take_checkpoint_memory(),
            })),
            YieldSuspension::CheckpointProbePanic => Ok(Box::new(PanicProbeCheckpoint {
                _retained: self.take_checkpoint_memory(),
            })),
            YieldSuspension::CheckpointDropPanic { retained_bytes } => {
                Ok(Box::new(PanicDropCheckpoint {
                    retained_bytes,
                    retained: self.take_checkpoint_memory(),
                }))
            }
            YieldSuspension::CheckpointMemoryMismatch {
                reported_retained_bytes,
                drop_panics,
                ..
            } => {
                let retained = self.take_checkpoint_memory();
                if drop_panics {
                    Ok(Box::new(PanicDropCheckpoint {
                        retained_bytes: reported_retained_bytes,
                        retained,
                    }))
                } else {
                    Ok(Box::new(YieldCheckpoint {
                        retained_bytes: reported_retained_bytes,
                        retained,
                    }))
                }
            }
            YieldSuspension::CheckpointProbeAndDropPanic => {
                Ok(Box::new(PanicProbeAndDropCheckpoint {
                    _retained: self.take_checkpoint_memory(),
                }))
            }
            YieldSuspension::CheckpointRestoreFailure { retained_bytes } => {
                Ok(Box::new(RestoreFailureCheckpoint {
                    retained_bytes,
                    _retained: self.take_checkpoint_memory(),
                }))
            }
            YieldSuspension::CheckpointRestorePanic { retained_bytes } => {
                Ok(Box::new(RestorePanicCheckpoint {
                    retained_bytes,
                    _retained: self.take_checkpoint_memory(),
                }))
            }
            YieldSuspension::CheckpointRestoreRejectAfterAdmission { retained_bytes } => {
                Ok(Box::new(RestoreRejectAfterAdmissionCheckpoint {
                    retained_bytes,
                    retained: self.take_checkpoint_memory(),
                }))
            }
            YieldSuspension::CheckpointRestorePanicAfterAdmission { retained_bytes } => {
                Ok(Box::new(RestorePanicAfterAdmissionCheckpoint {
                    retained_bytes,
                    retained: self.take_checkpoint_memory(),
                }))
            }
            YieldSuspension::CheckpointRestoreExecutionDropPanic {
                retained_bytes,
                checkpoint_drop_panics,
            } => Ok(Box::new(RestoreExecutionDropPanicCheckpoint {
                retained_bytes,
                checkpoint_drop_panics,
                retained: self.take_checkpoint_memory(),
            })),
            YieldSuspension::CheckpointArtifactGenerationRollbackFailure {
                retained_bytes,
                registry,
            } => Ok(Box::new(ArtifactGenerationRollbackFailureCheckpoint {
                retained_bytes,
                retained: self.take_checkpoint_memory(),
                registry,
            })),
            YieldSuspension::Failure => Err(WorthQueryGraphProviderFailure::new(
                "yield fixture suspension failed",
            )),
            YieldSuspension::Panic => panic!("yield fixture suspension panicked"),
        }
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for YieldProvider {
    type Execution = YieldExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        if self.record_effect {
            crate::domain_computation::provider_session::tests::execution_resource_support_with_yield_and_partial_effects("yield-fixture", 8)
        } else if self.yield_installed {
            crate::domain_computation::provider_session::tests::execution_resource_support_with_yield(
                "yield-fixture",
                8,
            )
        } else {
            crate::domain_computation::provider_session::tests::execution_resource_support(
                "yield-fixture",
                8,
            )
        }
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(
            start,
            YieldExecution {
                step_ordinal: 0,
                checkpoint_available: self.checkpoint_available,
                record_effect: self.record_effect,
                suspension: self.suspension.clone(),
                execution_drop_panics: self.execution_drop_panics,
                retained: None,
            },
        )
    }
}

impl YieldExecution {
    pub fn take_checkpoint_memory(&mut self) -> WorthQueryGraphProviderRetainedMemory {
        self.retained
            .take()
            .expect("checkpointable execution transfers governed retained memory once")
    }
}

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}

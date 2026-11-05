use crate::domain_computation::managed_run::tests::yield_fixture::YieldProvider;
use crate::domain_computation::managed_run::tests::yield_fixture::YieldSuspension;

impl YieldProvider {
    pub fn artifact_generation_rollback_failure(
        retained_bytes: u64,
    ) -> (
        Self,
        std::sync::Arc<
            std::sync::Mutex<
                Option<
                    std::sync::Arc<
                        worth_query_execution::facade::domain_computation::WorthQueryWorkflowArtifactRegistry,
                    >,
                >,
            >,
        >,
    ){
        let registry = std::sync::Arc::new(std::sync::Mutex::new(None));
        (
            Self {
                yield_installed: true,
                checkpoint_available: true,
                record_effect: false,
                suspension: YieldSuspension::CheckpointArtifactGenerationRollbackFailure {
                    retained_bytes,
                    registry: std::sync::Arc::clone(&registry),
                },
                execution_drop_panics: false,
            },
            registry,
        )
    }

    pub const fn installed(retained_bytes: u64) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::Checkpoint { retained_bytes },
            execution_drop_panics: false,
        }
    }

    pub const fn installed_with_partial_effect(retained_bytes: u64) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: true,
            suspension: YieldSuspension::Checkpoint { retained_bytes },
            execution_drop_panics: false,
        }
    }

    pub const fn without_installed_yield() -> Self {
        Self {
            yield_installed: false,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::Checkpoint { retained_bytes: 3 },
            execution_drop_panics: false,
        }
    }

    pub const fn without_checkpoint_evidence() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: false,
            record_effect: false,
            suspension: YieldSuspension::Checkpoint { retained_bytes: 3 },
            execution_drop_panics: false,
        }
    }

    pub const fn suspension_failure() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::Failure,
            execution_drop_panics: false,
        }
    }

    pub const fn suspension_panic() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::Panic,
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_probe_panic() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointProbePanic,
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_drop_panic() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointDropPanic { retained_bytes: 3 },
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_restore_failure(retained_bytes: u64) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointRestoreFailure { retained_bytes },
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_restore_panic(retained_bytes: u64) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointRestorePanic { retained_bytes },
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_restore_reject_after_admission(retained_bytes: u64) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointRestoreRejectAfterAdmission { retained_bytes },
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_restore_panic_after_admission(retained_bytes: u64) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointRestorePanicAfterAdmission { retained_bytes },
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_and_restored_execution_drop_panic(retained_bytes: u64) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointRestoreExecutionDropPanic {
                retained_bytes,
                checkpoint_drop_panics: true,
            },
            execution_drop_panics: false,
        }
    }

    pub const fn checkpoint_probe_and_drop_panic() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointProbeAndDropPanic,
            execution_drop_panics: false,
        }
    }

    pub const fn suspension_and_execution_drop_panic() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::Panic,
            execution_drop_panics: true,
        }
    }

    pub const fn suspension_failure_and_execution_drop_panic() -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::Failure,
            execution_drop_panics: true,
        }
    }

    pub const fn checkpoint_and_execution_drop_panic(checkpoint_drop_panics: bool) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: if checkpoint_drop_panics {
                YieldSuspension::CheckpointDropPanic { retained_bytes: 3 }
            } else {
                YieldSuspension::Checkpoint { retained_bytes: 3 }
            },
            execution_drop_panics: true,
        }
    }

    pub const fn checkpoint_memory_mismatch(
        governed_retained_bytes: u64,
        reported_retained_bytes: u64,
        drop_panics: bool,
    ) -> Self {
        Self {
            yield_installed: true,
            checkpoint_available: true,
            record_effect: false,
            suspension: YieldSuspension::CheckpointMemoryMismatch {
                governed_retained_bytes,
                reported_retained_bytes,
                drop_panics,
            },
            execution_drop_panics: false,
        }
    }
}

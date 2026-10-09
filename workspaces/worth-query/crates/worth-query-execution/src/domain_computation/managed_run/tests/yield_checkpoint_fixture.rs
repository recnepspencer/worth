use super::yield_fixture::YieldExecution;
use super::*;

pub(super) struct YieldCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct PanicProbeCheckpoint {
    pub(super) _retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct PanicDropCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct PanicProbeAndDropCheckpoint {
    pub(super) _retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct RestoreFailureCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) _retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct RestorePanicCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) _retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct RestoreRejectAfterAdmissionCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct RestorePanicAfterAdmissionCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct RestoreExecutionDropPanicCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) checkpoint_drop_panics: bool,
    pub(super) retained: WorthQueryGraphProviderRetainedMemory,
}

pub(super) struct ArtifactGenerationRollbackFailureCheckpoint {
    pub(super) retained_bytes: u64,
    pub(super) retained: WorthQueryGraphProviderRetainedMemory,
    pub(super) registry: Arc<
        std::sync::Mutex<
            Option<
                Arc<crate::domain_computation::artifact_owner::WorthQueryWorkflowArtifactRegistry>,
            >,
        >,
    >,
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for PanicProbeCheckpoint {
    fn retained_bytes(&self) -> u64 {
        panic!("yield fixture checkpoint probe panicked")
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        Err(WorthQueryGraphProviderFailure::new(
            "panicking checkpoint probe must never restore",
        ))
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for PanicDropCheckpoint {
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        let execution = Box::new(YieldExecution::restored(
            memory
                .rebind(&self.retained)
                .map_err(restore_memory_failure)?,
        )) as Box<dyn WorthQueryGraphProviderExecution>;
        admit_restored_provider_execution(memory, execution)
    }
}

impl Drop for PanicDropCheckpoint {
    fn drop(&mut self) {
        panic!("yield fixture checkpoint drop panicked")
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for PanicProbeAndDropCheckpoint {
    fn retained_bytes(&self) -> u64 {
        panic!("yield fixture checkpoint probe panicked before panicking drop")
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        Err(WorthQueryGraphProviderFailure::new(
            "double-panicking checkpoint must never restore",
        ))
    }
}

impl Drop for PanicProbeAndDropCheckpoint {
    fn drop(&mut self) {
        panic!("yield fixture checkpoint drop panicked after panicking probe")
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for RestoreFailureCheckpoint {
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        Err(WorthQueryGraphProviderFailure::new(
            "yield fixture restore denied",
        ))
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for RestorePanicCheckpoint {
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        panic!("yield fixture restore panicked")
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint
    for RestoreRejectAfterAdmissionCheckpoint
{
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        let retained = memory
            .rebind(&self.retained)
            .map_err(restore_memory_failure)?;
        let _admission = memory
            .admit_cooperative_execution(|| {
                Box::new(YieldExecution::restored(retained))
                    as Box<dyn WorthQueryGraphProviderExecution>
            })
            .map_err(restore_memory_failure)?;
        Err(WorthQueryGraphProviderFailure::new(
            "provider rejected after restored execution admission",
        ))
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint
    for RestorePanicAfterAdmissionCheckpoint
{
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        let retained = memory
            .rebind(&self.retained)
            .map_err(restore_memory_failure)?;
        let _admission = memory
            .admit_cooperative_execution(|| {
                Box::new(YieldExecution::restored_with_drop_panic(retained))
                    as Box<dyn WorthQueryGraphProviderExecution>
            })
            .map_err(restore_memory_failure)?;
        panic!("provider panicked after restored execution admission")
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint
    for RestoreExecutionDropPanicCheckpoint
{
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        let execution = Box::new(YieldExecution::restored_with_drop_panic(
            memory
                .rebind(&self.retained)
                .map_err(restore_memory_failure)?,
        )) as Box<dyn WorthQueryGraphProviderExecution>;
        admit_restored_provider_execution(memory, execution)
    }
}

impl Drop for RestoreExecutionDropPanicCheckpoint {
    fn drop(&mut self) {
        if self.checkpoint_drop_panics {
            panic!("yield fixture checkpoint and restored execution drop panicked")
        }
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint
    for ArtifactGenerationRollbackFailureCheckpoint
{
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        let execution = Box::new(YieldExecution::restored(
            memory
                .rebind(&self.retained)
                .map_err(restore_memory_failure)?,
        )) as Box<dyn WorthQueryGraphProviderExecution>;
        admit_restored_provider_execution(memory, execution)
    }
}

impl Drop for ArtifactGenerationRollbackFailureCheckpoint {
    fn drop(&mut self) {
        let registry = self
            .registry
            .lock()
            .expect("artifact-generation rollback fixture registry lock must remain available")
            .take()
            .expect("workflow test must bind the yielded artifact registry before readmission");
        registry.close_cancelled();
        panic!("checkpoint release disrupted artifact-generation rollback")
    }
}

impl crate::domain_computation::WorthQueryGraphProviderCheckpoint for YieldCheckpoint {
    fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        #[cfg(feature = "test-query-execution-observer")]
        super::readmission_custody::record_restore();
        let execution = Box::new(YieldExecution::restored(
            memory
                .rebind(&self.retained)
                .map_err(restore_memory_failure)?,
        )) as Box<dyn WorthQueryGraphProviderExecution>;
        admit_restored_provider_execution(memory, execution)
    }
}

fn restore_memory_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}

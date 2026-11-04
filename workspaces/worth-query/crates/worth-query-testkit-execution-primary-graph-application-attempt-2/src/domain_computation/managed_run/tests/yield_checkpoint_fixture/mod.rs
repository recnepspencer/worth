use crate::domain_computation::managed_run::tests::yield_fixture::YieldExecution;
use crate::domain_computation::managed_run::tests::*;

pub struct YieldCheckpoint {
    pub retained_bytes: u64,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct PanicProbeCheckpoint {
    pub _retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct PanicDropCheckpoint {
    pub retained_bytes: u64,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct PanicProbeAndDropCheckpoint {
    pub _retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct RestoreFailureCheckpoint {
    pub retained_bytes: u64,
    pub _retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct RestorePanicCheckpoint {
    pub retained_bytes: u64,
    pub _retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct RestoreRejectAfterAdmissionCheckpoint {
    pub retained_bytes: u64,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct RestorePanicAfterAdmissionCheckpoint {
    pub retained_bytes: u64,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct RestoreExecutionDropPanicCheckpoint {
    pub retained_bytes: u64,
    pub checkpoint_drop_panics: bool,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

pub struct ArtifactGenerationRollbackFailureCheckpoint {
    pub retained_bytes: u64,
    pub retained: WorthQueryGraphProviderRetainedMemory,
    pub registry: Arc<
        std::sync::Mutex<
            Option<
                Arc<worth_query_execution::facade::domain_computation::WorthQueryWorkflowArtifactRegistry>,
            >,
        >,
    >,
}

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for PanicProbeCheckpoint
{
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for PanicDropCheckpoint
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

impl Drop for PanicDropCheckpoint {
    fn drop(&mut self) {
        panic!("yield fixture checkpoint drop panicked")
    }
}

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for PanicProbeAndDropCheckpoint
{
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for RestoreFailureCheckpoint
{
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for RestorePanicCheckpoint
{
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
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

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for YieldCheckpoint
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
        crate::domain_computation::managed_run::tests::readmission_custody::record_restore();
        let execution = Box::new(YieldExecution::restored(
            memory
                .rebind(&self.retained)
                .map_err(restore_memory_failure)?,
        )) as Box<dyn WorthQueryGraphProviderExecution>;
        admit_restored_provider_execution(memory, execution)
    }
}

pub fn restore_memory_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}

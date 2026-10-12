use super::{
    WorthQueryBatchWriteReceipt, WorthQueryBatchWriteReceiptInspection, WorthQueryEffectHandle,
    WorthQueryEffectIntentReceipt, WorthQueryInspection, WorthQueryIntentDeclaration,
    WorthQueryIntentReceipt, WorthQueryMutationBatchBuilder, WorthQueryRuntimeError,
    WorthQueryRuntimeFacadeFamily, WorthQueryWorkspace, WorthQueryWriteCommand,
    WorthQueryWriteReceipt, WorthQueryWriteReceiptInspection,
};

pub struct WorthQueryWorkspaceSubmissionLane<'a> {
    runtime: &'a mut super::WorthQueryRuntime,
}

impl<'a> WorthQueryWorkspaceSubmissionLane<'a> {
    pub(crate) fn new(runtime: &'a mut super::WorthQueryRuntime) -> Self {
        Self { runtime }
    }

    pub fn submit(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        command: WorthQueryWriteCommand,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.runtime.write(execution, command)
    }

    pub fn submit_with_inspection(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        command: WorthQueryWriteCommand,
    ) -> Result<(WorthQueryWriteReceipt, WorthQueryWriteReceiptInspection), WorthQueryRuntimeError>
    {
        let receipt = self.runtime.write(execution, command)?;
        let inspection = match self.runtime.inspect(execution, &receipt)? {
            WorthQueryInspection::WriteReceipt(inspection) => inspection,
            other => panic!("expected write receipt inspection, got {other:?}"),
        };
        Ok((receipt, inspection))
    }

    pub fn submit_batch(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        self.runtime.write_batch_intent(commands).execute(execution)
    }

    pub fn submit_batch_with_inspection(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> Result<
        (
            WorthQueryBatchWriteReceipt,
            WorthQueryBatchWriteReceiptInspection,
        ),
        WorthQueryRuntimeError,
    > {
        let receipt = self
            .runtime
            .write_batch_intent(commands)
            .execute(execution)?;
        let inspection = match self.runtime.inspect(execution, &receipt)? {
            WorthQueryInspection::BatchWriteReceipt(inspection) => inspection,
            other => panic!("expected batch write receipt inspection, got {other:?}"),
        };
        Ok((receipt, inspection))
    }

    pub fn submit_batch_builder(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        builder: WorthQueryMutationBatchBuilder,
    ) -> Result<super::WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        self.submit_batch(execution, builder.finish()?)
    }

    pub fn submit_intent(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentReceipt, WorthQueryRuntimeError> {
        self.runtime.execute_intent(execution, declaration)
    }

    pub fn submit_effect_intent<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: &WorthQueryEffectHandle<T>,
        strategy_version: impl Into<String>,
        input_contract: impl Into<String>,
    ) -> Result<WorthQueryEffectIntentReceipt, WorthQueryRuntimeError> {
        self.runtime.execute_next_effect_write_intent(
            execution,
            effect,
            strategy_version,
            input_contract,
        )
    }

    pub fn write_intent(
        &'a mut self,
        command: WorthQueryWriteCommand,
    ) -> crate::intent_admission::WorthQueryRuntimeWriteIntentAuthoring<'a> {
        self.runtime.write_intent(command)
    }

    pub fn write_batch_intent(
        &'a mut self,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> crate::intent_admission::WorthQueryRuntimeWriteBatchIntentAuthoring<'a> {
        self.runtime.write_batch_intent(commands)
    }
}

impl WorthQueryWorkspace {
    pub fn submissions(
        &mut self,
    ) -> Result<WorthQueryWorkspaceSubmissionLane<'_>, WorthQueryRuntimeError> {
        self.runtime
            .admit_facade_family(WorthQueryRuntimeFacadeFamily::Submission)?;
        Ok(WorthQueryWorkspaceSubmissionLane::new(&mut self.runtime))
    }
}

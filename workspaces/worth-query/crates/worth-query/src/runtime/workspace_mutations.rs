use super::workspace::WorthQueryWorkspace;
#[cfg(test)]
use super::WorthQueryMutationMetadata;
use super::{
    WorthQueryAspectMutationBuilder, WorthQueryDeleteMutationBuilder,
    WorthQueryExistingEntityTarget, WorthQueryExistingTruthTargetBinding, WorthQueryRuntimeError,
    WorthQueryWriteCommand, WorthQueryWriteReceipt,
};
#[cfg(test)]
use super::{
    WorthQueryBatchWriteReceipt, WorthQueryExistingRelationTarget, WorthQueryMutationBatchBuilder,
};
use crate::memory_workspace::WorthQueryEntityIdentity;

impl WorthQueryWorkspace {
    pub(crate) fn write(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        command: WorthQueryWriteCommand,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.write_intent(command).execute(execution)
    }

    pub fn write_intent(
        &mut self,
        command: WorthQueryWriteCommand,
    ) -> crate::intent_admission::WorthQueryRuntimeWriteIntentAuthoring<'_> {
        self.runtime.write_intent(command)
    }

    pub fn write_batch_intent(
        &mut self,
        commands: Vec<WorthQueryWriteCommand>,
    ) -> crate::intent_admission::WorthQueryRuntimeWriteBatchIntentAuthoring<'_> {
        self.runtime.write_batch_intent(commands)
    }

    pub fn insert(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        collection: impl Into<String>,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_insert(collection)?;
        self.write(execution, command)
    }

    pub fn update(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_identity: WorthQueryEntityIdentity,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_update(entity_identity)?;
        self.write(execution, command)
    }
    pub(crate) fn bind_existing_entity(
        &self,
        target: WorthQueryExistingEntityTarget,
    ) -> Result<WorthQueryExistingTruthTargetBinding, WorthQueryRuntimeError> {
        Ok(WorthQueryExistingTruthTargetBinding::from_entity_target(
            target,
        )?)
    }
    #[cfg(test)]
    pub(crate) fn bind_existing_relation(
        &self,
        target: WorthQueryExistingRelationTarget,
    ) -> Result<WorthQueryExistingTruthTargetBinding, WorthQueryRuntimeError> {
        Ok(WorthQueryExistingTruthTargetBinding::from_relation_target(
            target,
        )?)
    }

    pub(crate) fn probe_existing<I>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
        aspect_touches: I,
    ) -> Result<super::WorthQueryExistingTruthProbe, WorthQueryRuntimeError>
    where
        I: IntoIterator<Item = super::WorthQueryAspectTouch>,
    {
        Ok(self
            .runtime
            .probe_existing_intent(super::WorthQueryExistingTruthProbeRequest::new(
                binding,
                aspect_touches,
            )?)
            .execute(execution)?
            .probe()
            .clone())
    }

    pub fn probe_existing_intent(
        &self,
        request: super::WorthQueryExistingTruthProbeRequest,
    ) -> crate::intent_admission::WorthQueryRuntimeExistingTruthProbeIntentAuthoring<'_> {
        self.runtime.probe_existing_intent(request)
    }

    #[cfg(test)]
    pub(crate) fn update_existing(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_update_existing(binding)?;
        self.write(request_execution, command)
    }

    #[cfg(test)]
    pub(crate) fn assert_existing(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_assert_existing(binding)?;
        self.write(request_execution, command)
    }

    #[cfg(test)]
    pub(crate) fn verify_existing(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryAspectMutationBuilder::new()).build_verify_existing(binding)?;
        self.write(request_execution, command)
    }

    #[cfg(test)]
    pub(crate) fn update_existing_verified(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
        verify: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
        update: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let asserted_aspects = verify(WorthQueryAspectMutationBuilder::new())
            .finish_existing_truth_verification_aspects("backend-verified existing-truth update")?;
        let command = update(WorthQueryAspectMutationBuilder::new())
            .build_update_existing_verified(binding, asserted_aspects)?;
        self.write(request_execution, command)
    }

    pub fn delete(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_identity: WorthQueryEntityIdentity,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.write(
            execution,
            WorthQueryWriteCommand::Delete { entity_identity },
        )
    }

    pub fn delete_with(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity_identity: WorthQueryEntityIdentity,
        declaration: impl FnOnce(WorthQueryDeleteMutationBuilder) -> WorthQueryDeleteMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryDeleteMutationBuilder::new()).build_delete(entity_identity)?;
        self.write(execution, command)
    }

    #[cfg(test)]
    pub(crate) fn delete_existing(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        self.write(
            request_execution,
            WorthQueryWriteCommand::DeleteExistingAspects {
                binding,
                touched_aspects: Vec::new(),
                metadata: WorthQueryMutationMetadata::default(),
                naming_intent: None,
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn delete_existing_with(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
        declaration: impl FnOnce(WorthQueryDeleteMutationBuilder) -> WorthQueryDeleteMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let command =
            declaration(WorthQueryDeleteMutationBuilder::new()).build_delete_existing(binding)?;
        self.write(request_execution, command)
    }

    #[cfg(test)]
    pub(crate) fn delete_existing_verified(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryExistingTruthTargetBinding,
        verify: impl FnOnce(WorthQueryAspectMutationBuilder) -> WorthQueryAspectMutationBuilder,
        delete: impl FnOnce(WorthQueryDeleteMutationBuilder) -> WorthQueryDeleteMutationBuilder,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let asserted_aspects = verify(WorthQueryAspectMutationBuilder::new())
            .finish_existing_truth_verification_aspects("backend-verified existing-truth delete")?;
        let command = delete(WorthQueryDeleteMutationBuilder::new())
            .build_delete_existing_verified(binding, asserted_aspects)?;
        self.write(request_execution, command)
    }
    #[cfg(test)]
    pub(crate) fn batch(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: impl FnOnce(WorthQueryMutationBatchBuilder) -> WorthQueryMutationBatchBuilder,
    ) -> Result<WorthQueryBatchWriteReceipt, WorthQueryRuntimeError> {
        let commands = declaration(WorthQueryMutationBatchBuilder::new()).finish()?;
        self.write_batch_intent(commands).execute(request_execution)
    }
}

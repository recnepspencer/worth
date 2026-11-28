use worth_signal::facade::core::AsyncCapableNode;
use worth_signal::facade::{
    AsyncKeyedNodeCapabilityBinding, AsyncNodeCapabilityDeclaration,
    AsyncNodeHierarchyCancellationReport, AsyncNodePayloadContract,
    AsyncNodeRequestAdmissionReport, AsyncNodeRequestIntent, ResourceCancellationReason,
    SignalError,
};

use super::signal_recipes::BridgeSignalRecipeBinding;
use crate::facade::RuntimeBridge;

pub struct BridgeSignalRecipeAsyncAdmission {
    runtime_key: u64,
    report: AsyncNodeRequestAdmissionReport,
}

impl BridgeSignalRecipeAsyncAdmission {
    pub fn report(&self) -> &AsyncNodeRequestAdmissionReport {
        &self.report
    }
}

impl RuntimeBridge {
    pub fn install_signal_recipe_async_capability<F>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
        declaration: AsyncNodeCapabilityDeclaration,
    ) -> Result<AsyncCapableNode, SignalError> {
        binding.require_owner(self)?;
        if declaration.node() != binding.node() {
            return Err(SignalError::invalid_input(
                "async declaration targets a different Signal recipe node",
            ));
        }
        self.with_signal_source(|runtime| runtime.attach_async_capability(declaration))
    }

    pub fn bind_signal_recipe_async_capability<F>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
        payload: AsyncNodePayloadContract,
    ) -> Result<AsyncKeyedNodeCapabilityBinding, SignalError> {
        binding.require_owner(self)?;
        self.with_signal_source(|runtime| {
            binding
                .owner
                .recipe
                .keyed(binding.key.key.clone())
                .declare_async_capability(runtime, payload)
        })
    }

    pub fn signal_recipe_async_request_intent<F>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
    ) -> Result<AsyncNodeRequestIntent, SignalError> {
        binding.require_owner(self)?;
        self.with_signal_source(|runtime| {
            runtime
                .async_capable_node(binding.node())
                .map(|capability| capability.request_intent())
                .ok_or_else(|| SignalError::invalid_input("Signal recipe has no async capability"))
        })
    }

    pub fn admit_signal_recipe_async_request<F>(
        &self,
        binding: &BridgeSignalRecipeBinding<'_, F>,
        intent: AsyncNodeRequestIntent,
    ) -> Result<BridgeSignalRecipeAsyncAdmission, SignalError> {
        binding.require_owner(self)?;
        if intent.node() != binding.node() {
            return Err(SignalError::invalid_input(
                "async request targets a different Signal recipe node",
            ));
        }
        let report = self.with_signal_source(|runtime| runtime.admit_async_node_request(intent))?;
        Ok(BridgeSignalRecipeAsyncAdmission {
            runtime_key: self.signal_runtime_key,
            report,
        })
    }

    pub fn cancel_signal_recipe_async_request(
        &self,
        admission: &BridgeSignalRecipeAsyncAdmission,
        reason: ResourceCancellationReason,
    ) -> Result<AsyncNodeHierarchyCancellationReport, SignalError> {
        if admission.runtime_key != self.signal_runtime_key {
            return Err(SignalError::invalid_input(
                "async request belongs to a different Bridge runtime",
            ));
        }
        let handle = admission
            .report
            .resource_admission()
            .ok_or_else(|| SignalError::invalid_input("Signal did not admit an async request"))?
            .admitted_request()
            .handle();
        self.with_signal_source(|runtime| runtime.cancel_async_node_request(handle, reason))
    }
}

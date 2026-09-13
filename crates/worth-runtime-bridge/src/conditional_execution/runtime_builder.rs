use std::sync::Arc;

use worth_signal::facade::SignalGraph;

use super::{
    BridgeConditionalDenial, BridgeConditionalInstallationRequest,
    BridgeInstalledConditionalLowering, BridgeOwnedConditionalInstallationRequest,
    BridgeOwnedSignalRuntime,
};
use crate::facade::RuntimeBridge;

/// The only public construction posture for a Bridge-owned conditional runtime.
/// Sealing consumes the builder, so installation authority cannot accompany the
/// operational runtime.
pub struct BridgeConditionalRuntimeBuilder {
    runtime: BridgeOwnedSignalRuntime,
    managed_clocks:
        std::collections::BTreeMap<Arc<str>, super::managed_time::BridgeManagedClockDeclaration>,
}

impl BridgeConditionalRuntimeBuilder {
    pub fn with_owned_signal_graph(
        bridge: RuntimeBridge,
        evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) -> Result<Self, BridgeConditionalDenial> {
        BridgeOwnedSignalRuntime::with_owned_signal_graph(bridge, evaluation_budget)
            .map(Self::from_runtime)
    }

    pub fn new(
        bridge: RuntimeBridge,
        graph: Box<SignalGraph>,
        evaluation_budget: worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) -> Result<Self, BridgeConditionalDenial> {
        BridgeOwnedSignalRuntime::new(bridge, graph, evaluation_budget).map(Self::from_runtime)
    }

    pub fn install(
        &mut self,
        request: BridgeConditionalInstallationRequest,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeConditionalDenial> {
        self.runtime.install(request)
    }

    pub fn install_owned_conditional(
        &mut self,
        request: BridgeOwnedConditionalInstallationRequest,
    ) -> Result<Arc<BridgeInstalledConditionalLowering>, BridgeConditionalDenial> {
        self.runtime.install_owned_conditional(request)
    }

    pub fn install_managed_clock(
        &mut self,
        parts: super::BridgeManagedClockInstallationParts<'_>,
    ) -> Result<super::BridgeManagedClockBinding, super::BridgeManagedTemporalDenial> {
        self.runtime
            .require_managed_clock_lowering(parts.lowering)?;
        let declaration = super::managed_time::BridgeManagedClockDeclaration::admit(
            self.runtime.bridge.signal_runtime_key,
            &self.runtime.retention,
            parts,
        )?;
        if self.managed_clocks.contains_key(declaration.identity()) {
            return Err(super::BridgeManagedTemporalDenial::new(
                super::BridgeManagedTemporalDenialKind::DuplicateClockBinding,
                "managed clock binding identity is already declared",
            ));
        }
        let binding = declaration.binding();
        self.managed_clocks
            .insert(Arc::clone(declaration.identity()), declaration);
        Ok(binding)
    }

    pub fn install_owned_async_request_response(
        &mut self,
        declaration: super::BridgeOwnedAsyncRequestResponseDeclaration,
    ) -> Result<
        crate::facade::LoweredBridgeAsyncSourceDeclaration,
        crate::facade::BridgeAsyncSourceDeclarationRejection,
    > {
        self.runtime
            .install_owned_async_request_response(declaration)
    }

    pub fn seal(mut self) -> Result<super::BridgeSealedRuntimeAssembly, BridgeConditionalDenial> {
        self.runtime.seal_conditional_operations()?;
        for (identity, declaration) in self.managed_clocks {
            let lane = declaration.seal().map_err(|denial| {
                BridgeConditionalDenial::new(
                    super::BridgeConditionalDenialKind::SignalExecution,
                    denial.detail(),
                )
            })?;
            self.runtime
                .managed_clock_lanes
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(identity, Arc::new(std::sync::Mutex::new(lane)));
        }
        super::BridgeSealedRuntimeAssembly::from_sealed_runtime(self.runtime)
    }

    pub fn baseline_semantic_dependency_count(&self) -> usize {
        self.runtime.baseline_semantic_dependency_count()
    }

    pub fn active_semantic_dependency_count(&self) -> usize {
        self.runtime.active_semantic_dependency_count()
    }

    pub(super) fn from_runtime(runtime: BridgeOwnedSignalRuntime) -> Self {
        Self {
            runtime,
            managed_clocks: Default::default(),
        }
    }
}

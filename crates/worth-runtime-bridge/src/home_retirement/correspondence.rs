//! Consume the correspondence port's original Bridge without discarding metadata.
use super::{
    BridgeHomeRetirementDenial as Denial, BridgeHomeRetirementProgress as Progress,
    RetiringRuntimeBridge,
};
use crate::{
    correspondence::RuntimeWorldCorrespondenceInspectionLedger,
    facade::RuntimeWorldCorrespondencePort,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;

pub struct RetiringWorldCorrespondence {
    bridge: RetiringRuntimeBridge,
    _inspection: Arc<RuntimeWorldCorrespondenceInspectionLedger>,
}
impl RuntimeWorldCorrespondencePort {
    pub(crate) fn preflight_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        self.runtime.preflight_home_retirement(native, request)
    }
    pub fn into_home_retirement(
        self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<RetiringWorldCorrespondence, (Self, Denial)> {
        let Self {
            runtime,
            inspection,
        } = self;
        match runtime.into_home_retirement(native, request) {
            Ok(bridge) => Ok(RetiringWorldCorrespondence {
                bridge,
                _inspection: inspection,
            }),
            Err((runtime, cause)) => Err((
                Self {
                    runtime,
                    inspection,
                },
                cause,
            )),
        }
    }
}
impl RetiringWorldCorrespondence {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        self.bridge.advance(request, native)
    }
}

impl std::fmt::Debug for RetiringWorldCorrespondence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetiringWorldCorrespondence")
            .finish_non_exhaustive()
    }
}

//! Retire Query's ordinary Bridge and its local alias to Store's assembly.
use super::WorthQueryInstalledApplicationBridge;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryOperationHomeRetirementProgress as Progress,
};
use std::sync::{Arc, RwLock};
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{
    BridgeHomeRetirementDenial as BridgeDenial, BridgeHomeRetirementProgress as BridgeProgress,
    BridgeSealedRuntimeAssembly,
};

impl WorthQueryInstalledApplicationBridge {
    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        self.ordinary
            .as_ref()
            .expect("serving Bridge preflight")
            .preflight_home_retirement(native, request)
            .map_err(Denial::Bridge)?;
        self.conditional
            .as_ref()
            .expect("same original assembly")
            .try_write()
            .map_err(|_| Denial::Bridge(BridgeDenial::Busy))?
            .preflight_home_retirement(request, native)
            .map_err(Denial::Bridge)
    }

    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        owner: &Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    ) -> Result<Progress, Denial> {
        if self.conditional.is_none() {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        if !Arc::ptr_eq(
            self.conditional.as_ref().expect("same original alias"),
            owner,
        ) {
            return Err(Denial::Bridge(BridgeDenial::ForeignOwner));
        }
        if let Some(ordinary) = self.ordinary.take() {
            match ordinary.into_home_retirement(native, request) {
                Ok(task) => self.retiring_ordinary = Some(task),
                Err((ordinary, cause)) => {
                    self.ordinary = Some(ordinary);
                    return Err(Denial::Bridge(cause));
                }
            }
            return Ok(Progress::Advanced);
        }
        if let Some(task) = self.retiring_ordinary.as_mut() {
            match task.advance(request, native).map_err(Denial::Bridge)? {
                BridgeProgress::Advanced => return Ok(Progress::Advanced),
                BridgeProgress::Complete => {}
            }
            self.retired_ordinary = Some(
                self.retiring_ordinary
                    .take()
                    .expect("same completed Bridge")
                    .into_retired()
                    .unwrap_or_else(|_| unreachable!("owner reported Complete")),
            );
            return Ok(Progress::Advanced);
        }
        // Borrowing Store's actual matching Arc proves this local release
        // cannot destroy its assembly, Signal owner or arbitrary supplier data.
        drop(
            self.conditional
                .take()
                .expect("same final Query assembly alias"),
        );
        Ok(Progress::Complete)
    }
}

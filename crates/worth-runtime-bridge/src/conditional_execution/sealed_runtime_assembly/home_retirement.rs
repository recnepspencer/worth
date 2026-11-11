//! The same sealed assembly retains reversible Signal stop and Bridge retirement.
use super::BridgeSealedRuntimeAssembly;
use crate::facade::{
    BridgeHomeRetirementDenial as Denial, BridgeHomeRetirementProgress as Progress,
    RetiredRuntimeBridge, RetiringRuntimeBridge,
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_signal::facade::runtime::{
    PreparedInitialEmptySignalStop, StoppedInitialEmptySignalAllocation,
};

pub(super) struct FundedBridgeAssemblyRetirement {
    state: std::sync::Mutex<Box<BridgeAssemblyRetirement>>,
    _funding: Option<ExecutionMemoryReservation>,
}
struct BridgeAssemblyRetirement {
    prepared_signal: Option<PreparedInitialEmptySignalStop<(), (), (), (), ()>>,
    _stopped_signal: Option<StoppedInitialEmptySignalAllocation<(), (), (), (), ()>>,
    correspondence: Option<crate::home_retirement::RetiringWorldCorrespondence>,
    bridge: Option<RetiringRuntimeBridge>,
    _retired_bridge: Option<RetiredRuntimeBridge>,
    stage: AssemblyRetirementStage,
}
enum AssemblyRetirementStage {
    Unprepared,
    CommitSignal,
    Retention,
    Correspondence,
    Bridge,
    Complete,
}

impl BridgeSealedRuntimeAssembly {
    /// Allocate only the original-request-funded retirement backing while the
    /// Home still serves. The final World/Signal cut performs no allocation.
    pub fn preflight_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        self.runtime
            .bridge
            .preflight_home_retirement(native, request)?;
        self.correspondence
            .as_ref()
            .expect("serving assembly owns correspondence")
            .preflight_home_retirement(request, native)?;
        if self.home_retirement.is_none() {
            let funding = request
                .reserve_retained_memory(std::mem::size_of::<BridgeAssemblyRetirement>() as u64)
                .map_err(Denial::Execution)?;
            let state = Box::new(BridgeAssemblyRetirement {
                prepared_signal: None,
                _stopped_signal: None,
                correspondence: None,
                bridge: None,
                _retired_bridge: None,
                stage: AssemblyRetirementStage::Unprepared,
            });
            self.home_retirement = Some(FundedBridgeAssemblyRetirement {
                state: std::sync::Mutex::new(state),
                _funding: funding,
            });
        }
        Ok(())
    }

    /// Called only inside the Home's actual final World admission cut. Success
    /// retains CLOSING|0 in the already allocated slot; World begin follows
    /// immediately, with no intervening poll, allocation, callback or fallible step.
    #[doc(hidden)]
    pub fn prepare_initial_empty_home_stop(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        self.runtime
            .bridge
            .preflight_home_retirement(native, request)?;
        let state = self
            .home_retirement
            .as_mut()
            .ok_or(Denial::Unsupported)?
            .state
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .as_mut();
        if !matches!(state.stage, AssemblyRetirementStage::Unprepared) {
            return Err(Denial::Unsupported);
        }
        let signal = self.runtime.try_prepare_home_signal_stop(request)?;
        state.prepared_signal = Some(signal);
        state.stage = AssemblyRetirementStage::CommitSignal;
        Ok(())
    }

    /// Retry the same stopped owner. This never resumes service after World seal.
    pub fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        let state = self
            .home_retirement
            .as_mut()
            .ok_or(Denial::Unsupported)?
            .state
            .get_mut()
            .unwrap_or_else(|error| error.into_inner())
            .as_mut();
        if matches!(state.stage, AssemblyRetirementStage::Complete) {
            return Ok(Progress::Complete);
        }
        request.consult().map_err(Denial::Execution)?;
        match state.stage {
            AssemblyRetirementStage::Unprepared => return Err(Denial::Unsupported),
            AssemblyRetirementStage::CommitSignal => {
                let signal = state
                    .prepared_signal
                    .take()
                    .expect("original prepared Signal remains owned");
                match signal.commit(request) {
                    Ok(stopped) => {
                        state._stopped_signal = Some(stopped);
                        state.stage = AssemblyRetirementStage::Retention;
                    }
                    Err((signal, cause)) => {
                        state.prepared_signal = Some(signal);
                        return Err(Denial::Signal(cause));
                    }
                }
            }
            AssemblyRetirementStage::Retention => {
                self.runtime.retention.try_close_for_home_stop()?;
                state.stage = AssemblyRetirementStage::Correspondence;
            }
            AssemblyRetirementStage::Correspondence => {
                if state.correspondence.is_none() {
                    let original = self
                        .correspondence
                        .take()
                        .expect("same original correspondence");
                    match original.into_home_retirement(request, native) {
                        Ok(task) => state.correspondence = Some(task),
                        Err((original, cause)) => {
                            self.correspondence = Some(original);
                            return Err(cause);
                        }
                    }
                    return Ok(Progress::Advanced);
                }
                if state
                    .correspondence
                    .as_mut()
                    .expect("same correspondence task")
                    .advance(request, native)?
                    == Progress::Complete
                {
                    state.stage = AssemblyRetirementStage::Bridge;
                }
            }
            AssemblyRetirementStage::Bridge => {
                if state.bridge.is_none() {
                    let bridge = self
                        .runtime
                        .bridge
                        .take()
                        .expect("original Bridge remains until extraction");
                    match bridge.into_home_retirement(native, request) {
                        Ok(bridge) => state.bridge = Some(bridge),
                        Err((bridge, cause)) => {
                            self.runtime.bridge.restore(bridge);
                            return Err(cause);
                        }
                    }
                    return Ok(Progress::Advanced);
                }
                match state
                    .bridge
                    .as_mut()
                    .expect("same Bridge task")
                    .advance(request, native)?
                {
                    Progress::Advanced => {}
                    Progress::Complete => {
                        let bridge = state.bridge.take().expect("same completed Bridge task");
                        state._retired_bridge = Some(
                            bridge
                                .into_retired()
                                .unwrap_or_else(|_| unreachable!("owner reported Complete")),
                        );
                        state.stage = AssemblyRetirementStage::Complete;
                    }
                }
            }
            AssemblyRetirementStage::Complete => return Ok(Progress::Complete),
        }
        Ok(Progress::Advanced)
    }
}
#[cfg(test)]
#[path = "home_retirement_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

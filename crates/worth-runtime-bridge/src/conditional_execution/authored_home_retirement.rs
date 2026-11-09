//! Retire the actual authored assembly and retain its same stopped allocations.
use super::{
    authored_section::{
        branch_projection::BranchPosture, branch_registry::BranchSlot, readmission::Program,
    },
    BridgeReadmittedAuthoredAssembly,
};
use crate::facade::{
    BridgeHomeRetirementDenial as Denial, BridgeHomeRetirementProgress as Progress,
    RetiredRuntimeBridge, RetiringRuntimeBridge, RuntimeBridge,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::runtime::RelationalRuntime;

pub struct RetiringBridgeAuthoredAssembly {
    _stopped_signal: worth_signal::facade::durability::SignalRetainedAuthoredHomeAllocation,
    bridge: Option<RuntimeBridge>,
    task: Option<RetiringRuntimeBridge>,
    _retired: Option<RetiredRuntimeBridge>,
    _branches: Arc<super::authored_section::branch_registry::BranchRegistry>,
    _programme_memory: Option<ExecutionMemoryReservation>,
    _assembly_memory: Option<ExecutionMemoryReservation>,
    _funding: Option<ExecutionMemoryReservation>,
    stage: Stage,
}
enum Stage {
    Bridge,
    Complete,
}
impl BridgeReadmittedAuthoredAssembly {
    pub fn preflight_home_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<(), Denial> {
        self.program
            .bridge
            .preflight_home_retirement(native, request)?;
        if Arc::strong_count(&self.branches) != 1 {
            return Err(Denial::Busy);
        }
        let slots = self.branches.slots.try_lock().map_err(|_| Denial::Busy)?;
        for slot in slots.values() {
            match slot {
                BranchSlot::Serving { projection, .. } => {
                    if Arc::strong_count(projection) != 1 {
                        return Err(Denial::Busy);
                    }
                    let state = projection.state.try_lock().map_err(|_| Denial::Busy)?;
                    match state.posture {
                        BranchPosture::Serving => {}
                        BranchPosture::Stopped
                        | BranchPosture::ForkPreparing { .. }
                        | BranchPosture::Preparing { .. }
                        | BranchPosture::Pending { .. }
                        | BranchPosture::ReadmissionRequired { .. } => return Err(Denial::Busy),
                    }
                    if Arc::strong_count(&state.lowerings) != 1
                        || Arc::strong_count(&state.declarations) != 1
                        || Arc::strong_count(&state.source_owner) != 1
                    {
                        return Err(Denial::Busy);
                    }
                }
                BranchSlot::Reserved { .. } | BranchSlot::ReadmissionRequired(_) => {
                    return Err(Denial::Busy)
                }
            }
        }
        if Arc::strong_count(&self.program) != 1 {
            return Err(Denial::Busy);
        }
        Ok(())
    }
    /// Only the consumed actual assembly can enter this retirement; no graph or programme is rebuilt.
    pub fn into_home_retirement(
        self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Box<RetiringBridgeAuthoredAssembly>, (Self, Denial)> {
        if let Err(cause) = self.preflight_home_retirement(request, native) {
            return Err((self, cause));
        }
        let funding = match request
            .reserve_retained_memory(std::mem::size_of::<RetiringBridgeAuthoredAssembly>() as u64)
        {
            Ok(funding) => funding,
            Err(cause) => return Err((self, Denial::Execution(cause))),
        };
        let Self {
            program,
            signal,
            branches,
            _memory,
        } = self;
        let stopped_signal = match signal.retain_stopped_home_allocation(request) {
            Ok(stopped) => stopped,
            Err((signal, cause)) => {
                return Err((
                    Self {
                        program,
                        signal,
                        branches,
                        _memory,
                    },
                    Denial::AuthoredSignal(Box::new(cause)),
                ))
            }
        };
        {
            let slots = branches
                .slots
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for slot in slots.values() {
                match slot {
                    BranchSlot::Serving { projection, .. } => {
                        let mut state = projection
                            .state
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        state.retention.close();
                        for lowering in state
                            .lowerings
                            .read()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .values()
                        {
                            lowering.lease.revoke_liveness();
                        }
                        state.posture = BranchPosture::Stopped;
                    }
                    BranchSlot::Reserved { .. } | BranchSlot::ReadmissionRequired(_) => {
                        unreachable!("same original passed preflight without an active task")
                    }
                }
            }
        }
        let Program {
            bridge,
            _memory: programme_memory,
            installation: _,
        } = Arc::try_unwrap(program)
            .unwrap_or_else(|_| unreachable!("checked holders leave the original programme"));
        Ok(Box::new(RetiringBridgeAuthoredAssembly {
            _stopped_signal: stopped_signal,
            bridge: Some(bridge),
            task: None,
            _retired: None,
            _programme_memory: programme_memory,
            _assembly_memory: _memory,
            _funding: funding,
            _branches: branches,
            stage: Stage::Bridge,
        }))
    }
}
impl RetiringBridgeAuthoredAssembly {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<Progress, Denial> {
        request.consult().map_err(Denial::Execution)?;
        match self.stage {
            Stage::Bridge => {
                if let Some(bridge) = self.bridge.take() {
                    match bridge.into_home_retirement(native, request) {
                        Ok(task) => self.task = Some(task),
                        Err((bridge, cause)) => {
                            self.bridge = Some(bridge);
                            return Err(cause);
                        }
                    }
                    return Ok(Progress::Advanced);
                }
                match self
                    .task
                    .as_mut()
                    .expect("same original Bridge task")
                    .advance(request, native)?
                {
                    Progress::Advanced => Ok(Progress::Advanced),
                    Progress::Complete => {
                        self._retired = Some(
                            self.task
                                .take()
                                .expect("completed original")
                                .into_retired()
                                .unwrap_or_else(|_| unreachable!("same original completed")),
                        );
                        self.stage = Stage::Complete;
                        Ok(Progress::Complete)
                    }
                }
            }
            Stage::Complete => Ok(Progress::Complete),
        }
    }
}

//! An unpublished recovered successor carries one shared terminal capture phase.
use super::*;
use crate::branch::{
    AdmittedRelationalBranchBasis, ObservedRelationalMemoryComponent,
    RecordedRelationalComponentReference,
};
use std::sync::Mutex;

#[derive(Debug)]
enum State {
    Capturable,
    NativePublished,
    Installed,
    Refused,
}
#[derive(Debug)]
struct BootstrapPhase {
    state: Mutex<State>,
    _memory: Option<ExecutionMemoryReservation>,
}
#[derive(Debug)]
pub(in crate::durability::authority) struct BootstrapPhaseOwner(Arc<BootstrapPhase>);
impl BootstrapPhaseOwner {
    pub(in crate::durability::authority) fn finish_capture(&self, admitted: bool) {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *state = if admitted {
            State::NativePublished
        } else {
            State::Refused
        };
    }
}
impl Drop for BootstrapPhaseOwner {
    fn drop(&mut self) {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match *state {
            State::Capturable => *state = State::Refused,
            State::NativePublished | State::Installed | State::Refused => {}
        }
    }
}
#[derive(Clone, Debug)]
pub struct NativeInterimBootstrapContinuation(Arc<Continuation>);
#[derive(Debug)]
struct Continuation {
    receiver: AdmittedNativeInterimReceiver,
    phase: Arc<BootstrapPhase>,
    current: ObservedRelationalMemoryComponent,
    description: RecordedRelationalComponentReference,
    _memory: Option<ExecutionMemoryReservation>,
}
impl crate::durability::authority::DurabilityRecoveryAuthority<'_> {
    /// Exclusive recovery and its linear exact image witness the pre-World cut.
    pub fn prepare_interim_bootstrap_continuation(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        origin: &CapturedNativeInterimOrigin,
        recovered: &mut crate::durability::authority::RecoveredRelationalRuntimeAuthority,
    ) -> Result<NativeInterimBootstrapContinuation, Refusal> {
        request_scope::run(request, || {
            let receiver = self
                .runtime
                .durability_authority()
                .admit_interim_receiver(origin, request)?;
            let (_, basis) = self
                .runtime
                .observe_branch(&self.runtime.main_branch_identity())
                .map_err(|_| Refusal::OwnerUnavailable)?;
            if !recovered.owns_image(basis.descriptor()) {
                return Err(Refusal::BindingMismatch(Mismatch::NativeOrigin));
            }
            if recovered.interim_bootstrap_phase.is_none() {
                let memory = reserve_arc::<BootstrapPhase>(request)?;
                recovered.interim_bootstrap_phase =
                    Some(BootstrapPhaseOwner(Arc::new(BootstrapPhase {
                        state: Mutex::new(State::Capturable),
                        _memory: memory,
                    })));
            }
            let phase = &recovered
                .interim_bootstrap_phase
                .as_ref()
                .expect("the actual recovery owns its phase")
                .0;
            require_capture(phase)?;
            let current = self
                .runtime
                .observe_memory_component(request, &basis)
                .map_err(component_stop)?;
            let description = self
                .runtime
                .export_recorded_current_component(request, &basis)
                .map_err(component_stop)?;
            let memory = reserve_arc::<Continuation>(request)?;
            Ok(NativeInterimBootstrapContinuation(Arc::new(Continuation {
                receiver,
                phase: phase.clone(),
                current,
                description,
                _memory: memory,
            })))
        })
    }
}
fn require_capture(phase: &BootstrapPhase) -> Result<(), Refusal> {
    match *phase
        .state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    {
        State::Capturable => Ok(()),
        State::NativePublished | State::Installed | State::Refused => {
            Err(Refusal::OwnerUnavailable)
        }
    }
}
impl NativeInterimBootstrapContinuation {
    pub fn original_receiver(&self) -> &AdmittedNativeInterimReceiver {
        &self.0.receiver
    }
    pub(crate) fn capture_current(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
        capture: &CapturedNativeInterimAdmission,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimComponent, Refusal> {
        require_capture(&self.0.phase)?;
        self.require_owner(runtime)?;
        let captured = capture
            .capture_current_component(runtime, &self.0.current, request)
            .map_err(component_stop)?;
        comparison_work(&self.0.description, request)?;
        if captured.description() != &self.0.description {
            return Err(Refusal::BindingMismatch(Mismatch::NativeOrigin));
        }
        require_capture(&self.0.phase)?;
        Ok(captured)
    }
    pub(crate) fn install_receiver(
        &self,
        runtime: &crate::runtime::RelationalRuntime,
        receiver: &AdmittedNativeInterimReceiver,
        basis: &AdmittedRelationalBranchBasis,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Refusal> {
        self.require_owner(runtime)?;
        if !Arc::ptr_eq(&receiver.source, &self.0.receiver.source) {
            return Err(Refusal::BindingMismatch(Mismatch::Source));
        }
        {
            let mut phase = self
                .0
                .phase
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match *phase {
                State::NativePublished => {}
                State::Capturable | State::Installed | State::Refused => {
                    return Err(Refusal::OwnerUnavailable)
                }
            }
            // Both success and refusal consume publication custody for all aliases.
            *phase = State::Refused;
        }
        let actual = runtime
            .export_recorded_current_component(request, basis)
            .map_err(component_stop)?;
        comparison_work(&self.0.description, request)?;
        if actual != self.0.description {
            return Err(Refusal::BindingMismatch(Mismatch::NativeOrigin));
        }
        *self
            .0
            .phase
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = State::Installed;
        Ok(())
    }
    fn require_owner(&self, runtime: &crate::runtime::RelationalRuntime) -> Result<(), Refusal> {
        if !Weak::ptr_eq(&self.0.receiver.owner, &runtime.state_binding()) {
            return Err(Refusal::BindingMismatch(Mismatch::Runtime));
        }
        runtime
            .durability
            .checkpoint_selection()
            .require_interim_image(&self.0.receiver.image)?;
        Ok(())
    }
}
fn comparison_work(
    description: &RecordedRelationalComponentReference,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let descriptor = description.descriptor();
    let variable = descriptor
        .branch_id()
        .0
        .len()
        .checked_add(descriptor.reference().branch_id().as_str().len())
        .and_then(|n| {
            n.checked_add(
                descriptor
                    .reference()
                    .target()
                    .as_basis()
                    .map_or(0, |target| target.parent_commit_ids().len()),
            )
        })
        .and_then(|n| {
            n.checked_add(match description.target() {
                crate::branch::RecordedRelationalComponentTarget::Empty => 0,
                crate::branch::RecordedRelationalComponentTarget::Canonical(target) => target
                    .receipt()
                    .branch_id
                    .0
                    .len()
                    .checked_add(target.receipt().parents.len())?,
            })
        })
        .and_then(|n| n.checked_add(1))
        .ok_or(Refusal::CapacityOverflow)?;
    crate::durability::artifact_payload_emission::request::account_work(
        request,
        u64::try_from(variable).map_err(|_| Refusal::CapacityOverflow)?,
    )
    .map_err(crate::durability::checkpoint_admission::native_stop)
}
fn component_stop(cause: crate::branch::RelationalRecordedComponentRefusal) -> Refusal {
    use crate::branch::RelationalRecordedComponentRefusal as R;
    match cause {
        R::Request(cause) => Refusal::Request(cause),
        R::Source(cause) => crate::durability::checkpoint_admission::native_stop(cause),
        R::PayloadBacking(cause) => Refusal::Buffer(cause),
        R::Basis(_)
        | R::Frame(_)
        | R::MissingPerformedCommit { .. }
        | R::CanonicalMismatch { .. }
        | R::PayloadEncoding { .. } => Refusal::InvalidProjectedMeaning,
    }
}

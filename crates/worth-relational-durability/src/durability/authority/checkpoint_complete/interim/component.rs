//! Capture-issued component DATA cannot be minted from a decoded description.
use worth_relational_contracts::internal_port as contracts;
use super::*;
use contracts::branch::recorded_basis::refusal::RelationalRecordedComponentRefusal;
use worth_relational_branch::internal_port::branch::recorded_basis::memory_component::ObservedRelationalMemoryComponent;
use worth_relational_branch::internal_port::branch::recorded_basis::{RecordedRelationalComponentReference, RecordedRelationalComponentTarget};
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_branch::internal_port::runtime::state::subsystems::services::sequence::origin::RelationalRuntimeOrigin;
use worth_relational_branch::internal_port::branch::recorded_basis::memory_component::capture::ObservedRelationalMemoryComponentOwnerPort as _;

#[derive(Clone, Debug)]
pub struct CapturedNativeInterimComponent(Arc<Component>);
#[derive(Debug)]
struct Component {
    source: Arc<CaptureIdentity>,
    description: RecordedRelationalComponentReference,
    _memory: Option<ExecutionMemoryReservation>,
}
impl CapturedNativeInterimComponent {
    pub fn description(&self) -> &RecordedRelationalComponentReference {
        &self.0.description
    }
    pub(super) fn belongs_to(&self, source: &Arc<CaptureIdentity>) -> bool {
        Arc::ptr_eq(&self.0.source, source)
    }
}
impl CapturedNativeInterimAdmission {
    pub(crate) fn capture_current_component(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
        component: &ObservedRelationalMemoryComponent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimComponent, RelationalRecordedComponentRefusal> {
        let captured = self.capture_component(origin, history, component, request)?;
        let current = super::super::branches::BranchImages::capture(&self.current, request)
            .map_err(|cause| RelationalRecordedComponentRefusal::Source(cause.into()))?;
        if !current.is_current_component(captured.description().descriptor()) {
            return Err(RelationalRecordedComponentRefusal::Basis(
                contracts::branch::basis_denial::RelationalBranchBasisDenial::WrongImmutableTarget,
            ));
        }
        Ok(captured)
    }
    pub(crate) fn capture_component(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
        component: &ObservedRelationalMemoryComponent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimComponent, RelationalRecordedComponentRefusal> {
        if !self.owner.same_origin(origin) {
            return Err(RelationalRecordedComponentRefusal::Basis(
                contracts::branch::basis_denial::RelationalBranchBasisDenial::OwnerUnavailable,
            ));
        }
        let captured = match component.capture_commit_id() {
            None => None,
            Some(commit_id) => {
                let levels = self
                    .source
                    .canonical
                    .len()
                    .checked_ilog2()
                    .map_or(1, |n| n + 1);
                contracts::durability::data::recovery_stop::account_work(
                    request,
                    u64::from(levels),
                )
                .map_err(RelationalRecordedComponentRefusal::Source)?;
                let index =
                    self.source.canonical.get(&commit_id).ok_or(
                        RelationalRecordedComponentRefusal::CanonicalMismatch { commit_id },
                    )?;
                super::origin_readmission::receipt_lookup_work(
                    &index.commit,
                    &self.source,
                    request,
                )
                .map_err(|cause| {
                    RelationalRecordedComponentRefusal::Source(
                        contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(cause),
                    )
                })?;
                let ordinal = index
                    .position
                    .0
                    .checked_sub(1)
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or(RelationalRecordedComponentRefusal::CanonicalMismatch { commit_id })?;
                let artifact = self
                    ._artifacts
                    .get(ordinal)
                    .ok_or(RelationalRecordedComponentRefusal::CanonicalMismatch { commit_id })?;
                Some((index.as_ref(), artifact.as_ref()))
            }
        };
        let description = component.export_from_capture(origin, history, request, captured)?;
        if let RecordedRelationalComponentTarget::Canonical(target) = description.target() {
            super::origin_readmission::receipt_lookup_work(target.receipt(), &self.source, request)
                .map_err(|cause| match cause {
                    Refusal::Request(cause) => RelationalRecordedComponentRefusal::Request(cause),
                    cause => RelationalRecordedComponentRefusal::Source(
                        contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(cause),
                    ),
                })?;
            let index = self
                .source
                .canonical
                .get(&target.receipt().commit_id)
                .ok_or(RelationalRecordedComponentRefusal::CanonicalMismatch {
                    commit_id: target.receipt().commit_id,
                })?;
            if index.commit != *target.receipt()
                || index.position != target.position()
                || &index.payload_digest != target.artifact_digest()
                || &index.root != target.roots()
            {
                return Err(RelationalRecordedComponentRefusal::CanonicalMismatch {
                    commit_id: target.receipt().commit_id,
                });
            }
        }
        let memory = reserve_arc::<Component>(request).map_err(|cause| {
            RelationalRecordedComponentRefusal::Source(
                contracts::durability::data::recovery_stop::RelationalRecoveryStop::AdmissionImage(
                    cause,
                ),
            )
        })?;
        Ok(CapturedNativeInterimComponent(Arc::new(Component {
            source: self.source.clone(),
            description,
            _memory: memory,
        })))
    }
}

#[doc(hidden)]
pub trait CapturedNativeInterimAdmissionOwnerPort: Sized {
    fn capture_current_component(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
        component: &ObservedRelationalMemoryComponent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimComponent, RelationalRecordedComponentRefusal>;

    fn capture_component(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
        component: &ObservedRelationalMemoryComponent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimComponent, RelationalRecordedComponentRefusal>;
}

impl CapturedNativeInterimAdmissionOwnerPort for CapturedNativeInterimAdmission {
    fn capture_current_component(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
        component: &ObservedRelationalMemoryComponent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimComponent, RelationalRecordedComponentRefusal> {
        self.capture_current_component(origin, history, component, request)
    }

    fn capture_component(
        &self,
        origin: &RelationalRuntimeOrigin,
        history: &HistorySubsystem,
        component: &ObservedRelationalMemoryComponent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedNativeInterimComponent, RelationalRecordedComponentRefusal> {
        self.capture_component(origin, history, component, request)
    }
}

#[doc(hidden)]
pub trait CapturedNativeInterimComponentOwnerPort: Sized {
    fn belongs_to(&self, source: &Arc<CaptureIdentity>) -> bool;
}

impl CapturedNativeInterimComponentOwnerPort for CapturedNativeInterimComponent {
    fn belongs_to(&self, source: &Arc<CaptureIdentity>) -> bool {
        self.belongs_to(source)
    }
}

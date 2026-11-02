//! Consuming, request-bounded retirement of the authentic source-backed owner.
mod graph;
mod history;
mod progression;
use super::{RelationalRuntime, RelationalRuntimeCloseDenial};
use crate::durability::data::NativeDirectoryRetirementHandoff;
use crate::durability::medium::{
    NativeMemberSlotDenial, RelationalCheckpointAcknowledgementDenial, RelationalDurableColdSource,
    RelationalDurableMemberRetirementRefusal, RelationalMediumRetirementDenial,
    SealedRelationalMediumBinding,
};
use crate::storage::residency::{
    NativeRegionSourceRetirement, NativeRegionSourceRetirementDenial,
    NativeRegionSourceRetirementProgress, NativeSourcePass,
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
pub enum RelationalSourceCloseDenial {
    SharedRuntime,
    Busy,
    Native(RelationalRuntimeCloseDenial),
    Medium(RelationalMediumRetirementDenial),
    Member(RelationalDurableMemberRetirementRefusal),
    Checkpoint(RelationalCheckpointAcknowledgementDenial),
    Execution(worth_execution::WorkCeilingDenial),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalSourceCloseProgress {
    Advanced,
    Complete,
}

/// The same original runtime/state and callback tasks survive every refusal.
/// Final Drop is unbudgeted abandonment, never successful physical retirement.
#[must_use]
pub struct RelationalSourceClose {
    allocation: PreparedRelationalSourceClose,
}
/// Original-request funded storage, issued before Home admission effects.
/// It carries no runtime, source, serving capability or completed-close fact.
#[must_use]
pub struct PreparedRelationalSourceClose {
    frame: Box<SourceCloseFrame>,
    _memory: Option<ExecutionMemoryReservation>,
    outer: Option<ExecutionMemoryReservation>,
}
struct SourceCloseFrame {
    state: Option<SourceCloseState>,
}
/// Original nonserving payload; only explicit reclaim destroys it.
/// ```compile_fail
/// use worth_relational::facade::runtime::RetiredSourceRelationalAllocation;
/// fn cannot_resume(value: RetiredSourceRelationalAllocation) { value.runtime(); }
/// ```
#[must_use]
pub struct RetiredSourceRelationalAllocation {
    _allocation: PreparedRelationalSourceClose,
}
impl PreparedRelationalSourceClose {
    pub fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_execution::WorkCeilingDenial> {
        request.consult()?;
        let memory =
            request.reserve_retained_memory(std::mem::size_of::<SourceCloseFrame>() as u64)?;
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<RelationalRuntime>())
            .expect("fixed runtime layout")
            .0
            .pad_to_align()
            .size();
        let outer = request.reserve_retained_memory(bytes as u64)?;
        Ok(Self {
            frame: Box::new(SourceCloseFrame { state: None }),
            _memory: memory,
            outer,
        })
    }
}
impl std::fmt::Debug for PreparedRelationalSourceClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PreparedRelationalSourceClose")
    }
}

enum Stage {
    Quiesce,
    Validate,
    Seal,
    Cache,
    Retire,
    Medium,
    Source,
    Business,
    Complete,
}
struct SourceCloseState {
    runtime: RelationalRuntime,
    source_identity: usize,
    stage: Stage,
    graph: Option<graph::NativeSourceGraphRetirement>,
    medium: Option<SealedRelationalMediumBinding>,
    cache: usize,
    cache_retired: bool,
}
impl RelationalRuntime {
    /// Consume the original outer Arc into its original prepaid close frame.
    /// The inner state and legitimate dormant Weak services stay unchanged.
    pub fn into_source_close(
        self: Arc<Self>,
        allocation: PreparedRelationalSourceClose,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<
        RelationalSourceClose,
        (
            Arc<Self>,
            PreparedRelationalSourceClose,
            RelationalSourceCloseDenial,
        ),
    > {
        if let Err(cause) = request.consult() {
            return Err((
                self,
                allocation,
                RelationalSourceCloseDenial::Execution(cause),
            ));
        }
        let runtime = match Arc::try_unwrap(self) {
            Ok(runtime) => runtime,
            Err(original) => {
                return Err((
                    original,
                    allocation,
                    RelationalSourceCloseDenial::SharedRuntime,
                ))
            }
        };
        Ok(allocation.fill(runtime, source))
    }
    /// The unshared pre-service owner has never issued an outer Arc.
    pub fn into_owned_source_close(
        self,
        allocation: PreparedRelationalSourceClose,
        request: &ExecutionRequest<'_, '_>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<
        RelationalSourceClose,
        (
            Self,
            PreparedRelationalSourceClose,
            RelationalSourceCloseDenial,
        ),
    > {
        if let Err(cause) = request.consult() {
            return Err((
                self,
                allocation,
                RelationalSourceCloseDenial::Execution(cause),
            ));
        }
        Ok(allocation.fill(self, source))
    }
}
impl PreparedRelationalSourceClose {
    fn fill(
        mut self,
        mut runtime: RelationalRuntime,
        source: &dyn RelationalDurableColdSource,
    ) -> RelationalSourceClose {
        if self.outer.is_none() {
            self.outer = runtime._outer_memory.take();
        } else {
            drop(runtime._outer_memory.take());
        }
        assert!(
            self.frame.state.is_none(),
            "only unoccupied original preparation enters close"
        );
        self.frame.state = Some(SourceCloseState {
            runtime,
            source_identity: (source as *const dyn RelationalDurableColdSource).cast::<()>()
                as usize,
            stage: Stage::Quiesce,
            graph: None,
            medium: None,
            cache: 0,
            cache_retired: false,
        });
        RelationalSourceClose { allocation: self }
    }
}
impl RelationalSourceClose {
    fn state(&self) -> &SourceCloseState {
        self.allocation
            .frame
            .state
            .as_ref()
            .expect("active original close frame")
    }
    fn state_mut(&mut self) -> &mut SourceCloseState {
        self.allocation
            .frame
            .state
            .as_mut()
            .expect("active original close frame")
    }
    /// Before any cache or source retirement, restore the SAME runtime/state.
    pub fn resume(mut self) -> Result<(RelationalRuntime, PreparedRelationalSourceClose), Self> {
        if self.state().cache_retired
            || !matches!(
                self.state().stage,
                Stage::Quiesce | Stage::Validate | Stage::Cache | Stage::Seal
            )
        {
            return Err(self);
        }
        let SourceCloseState { mut runtime, .. } = self
            .allocation
            .frame
            .state
            .take()
            .expect("same original close state");
        if let Some(close) = runtime.tenure.close_authority() {
            close.resume_admission();
        }
        runtime._outer_memory = self.allocation.outer.take();
        Ok((runtime, self.allocation))
    }
    pub fn into_retired(mut self) -> Result<RetiredSourceRelationalAllocation, Self> {
        if !matches!(self.state().stage, Stage::Complete) {
            return Err(self);
        }
        // Serving restoration is now impossible; no outer allocation was made.
        drop(self.allocation.outer.take());
        Ok(RetiredSourceRelationalAllocation {
            _allocation: self.allocation,
        })
    }
}

impl RetiredSourceRelationalAllocation {
    pub fn reclaim(self) {
        drop(self);
    }
}
impl std::fmt::Debug for RelationalSourceClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RelationalSourceClose")
    }
}
impl std::fmt::Debug for RetiredSourceRelationalAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RetiredSourceRelationalAllocation")
    }
}
impl From<NativeMemberSlotDenial> for RelationalSourceCloseDenial {
    fn from(cause: NativeMemberSlotDenial) -> Self {
        match cause {
            NativeMemberSlotDenial::Busy => Self::Busy,
            NativeMemberSlotDenial::Refused(cause) => Self::Member(cause),
        }
    }
}
impl From<NativeRegionSourceRetirementDenial> for RelationalSourceCloseDenial {
    fn from(cause: NativeRegionSourceRetirementDenial) -> Self {
        match cause {
            NativeRegionSourceRetirementDenial::Busy => Self::Busy,
            NativeRegionSourceRetirementDenial::Member(cause) => Self::Member(cause),
            NativeRegionSourceRetirementDenial::Execution(cause) => Self::Execution(cause),
        }
    }
}

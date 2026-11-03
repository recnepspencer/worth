//! Consuming original World custody deferred until Query's actual retirement.
use super::{
    RuntimeWorldCloseDenial as CloseDenial, RuntimeWorldCloseReport, RuntimeWorldCloseState,
    RuntimeWorldResultSnapshotCleanup,
};
use crate::lifecycle::{owner::RuntimeWorldOwnerState, RuntimeWorldOwner};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;

#[derive(Debug)]
pub enum RuntimeWorldHomeRetirementDenial {
    Close(CloseDenial),
    Medium(crate::durability::WorldDurableMediumDenial),
    Bridge(worth_runtime_bridge::facade::BridgeHomeRetirementDenial),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RuntimeWorldHomeRetirementProgress {
    Advanced,
    Complete,
}
trait HomeRetirementOwner: Send + Sync {
    fn advance_original_home(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RuntimeWorldHomeRetirementDenial>;
}
/// Owns the SAME closed World state allocation. No live accessor or Clone is exposed.
#[must_use]
pub struct RuntimeWorldHomeRetirement {
    owner: Arc<dyn HomeRetirementOwner>,
    _cleanup: RuntimeWorldResultSnapshotCleanup,
    complete: bool,
}
#[derive(Debug)]
#[must_use]
pub struct RetiredRuntimeWorldHome {
    _original: RuntimeWorldHomeRetirement,
}
impl RuntimeWorldHomeRetirement {
    pub fn advance(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<RuntimeWorldHomeRetirementProgress, RuntimeWorldHomeRetirementDenial> {
        if self.complete {
            return Ok(RuntimeWorldHomeRetirementProgress::Complete);
        }
        self.complete = Arc::get_mut(&mut self.owner)
            .ok_or(RuntimeWorldHomeRetirementDenial::Close(CloseDenial::Busy))?
            .advance_original_home(request, native)?;
        Ok(if self.complete {
            RuntimeWorldHomeRetirementProgress::Complete
        } else {
            RuntimeWorldHomeRetirementProgress::Advanced
        })
    }
    pub fn into_retired(self) -> Result<RetiredRuntimeWorldHome, Self> {
        if self.complete {
            Ok(RetiredRuntimeWorldHome { _original: self })
        } else {
            Err(self)
        }
    }
}
impl RetiredRuntimeWorldHome {
    pub fn reclaim(self) {
        drop(self);
    }
}
impl<D, I, E, Ctx, T> RuntimeWorldOwner<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Consumes the owner only with its exact original closed report and cleanup.
    /// Arc erasure moves the existing allocation; it allocates no new backing.
    pub fn into_home_retirement(
        self,
        report: &RuntimeWorldCloseReport,
        cleanup: RuntimeWorldResultSnapshotCleanup,
    ) -> Result<
        RuntimeWorldHomeRetirement,
        (
            Self,
            RuntimeWorldResultSnapshotCleanup,
            RuntimeWorldHomeRetirementDenial,
        ),
    > {
        let admitted = (|| {
            let close = self
                .root
                .state
                .close
                .try_lock()
                .map_err(|_| CloseDenial::Busy)?;
            if close.state() != RuntimeWorldCloseState::Closed {
                return Err(CloseDenial::CloseNotCompleted);
            }
            let task = close.retirement_task()?;
            if !report.matches_original_task(&task)
                || !cleanup.completed_for(self.root.owner_identity())
            {
                return Err(CloseDenial::CloseProgressInvariant);
            }
            Ok(())
        })();
        if let Err(cause) = admitted {
            return Err((
                self,
                cleanup,
                RuntimeWorldHomeRetirementDenial::Close(cause),
            ));
        }
        let Self { root, mut presence } = self;
        presence.disarm();
        let root = match Arc::try_unwrap(root) {
            Ok(root) => root,
            Err(root) => {
                presence.rearm();
                return Err((
                    Self { root, presence },
                    cleanup,
                    RuntimeWorldHomeRetirementDenial::Close(CloseDenial::Busy),
                ));
            }
        };
        root.state
            .owner_present
            .store(false, std::sync::atomic::Ordering::Release);
        // Move the actual payload allocation; outer Root contains only this Arc.
        // Dormant weak public ports cannot issue service after this handoff.
        Ok(RuntimeWorldHomeRetirement {
            owner: root.state,
            _cleanup: cleanup,
            complete: false,
        })
    }
}
impl<D, I, E, Ctx, T> HomeRetirementOwner for RuntimeWorldOwnerState<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn advance_original_home(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RuntimeWorldHomeRetirementDenial> {
        super::home_owner_progress::advance(self, request, native)
    }
}

impl std::fmt::Debug for RuntimeWorldHomeRetirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeWorldHomeRetirement")
            .field("complete", &self.complete)
            .finish_non_exhaustive()
    }
}

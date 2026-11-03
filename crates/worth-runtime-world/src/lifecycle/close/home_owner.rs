//! Consuming original World custody deferred until Query's actual retirement.
use super::{
    RuntimeWorldCloseDenial as CloseDenial, RuntimeWorldCloseReport, RuntimeWorldCloseState,
    RuntimeWorldResultSnapshotCleanup,
};
use crate::lifecycle::{RuntimeWorldOwner, RuntimeWorldOwnerRoot};
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;

#[derive(Debug)]
pub enum RuntimeWorldHomeRetirementDenial {
    Close(CloseDenial),
    Medium(crate::durability::WorldDurableMediumDenial),
}
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RuntimeWorldHomeRetirementProgress {
    Advanced,
    Complete,
}
trait HomeRetirementOwner: std::fmt::Debug + Send + Sync {
    fn advance_original_home(
        &self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RuntimeWorldHomeRetirementDenial>;
}
/// Owns the SAME closed World allocation. No live accessor or Clone is exposed.
#[derive(Debug)]
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
        self.complete = self.owner.advance_original_home(request, native)?;
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
        mut self,
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
        // Preserve the exact allocation before consuming the public owner. Its
        // ordinary Drop locks admission gates, which the sealed handoff avoids.
        let owner = self.root.clone();
        self.root
            .state
            .owner_present
            .store(false, std::sync::atomic::Ordering::Release);
        self.transferred_to_home_retirement = true;
        Ok(RuntimeWorldHomeRetirement {
            owner,
            _cleanup: cleanup,
            complete: false,
        })
    }
}
impl<D, I, E, Ctx, T> HomeRetirementOwner for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn advance_original_home(
        &self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<bool, RuntimeWorldHomeRetirementDenial> {
        super::home_owner_progress::advance(&self.state, request, native)
    }
}

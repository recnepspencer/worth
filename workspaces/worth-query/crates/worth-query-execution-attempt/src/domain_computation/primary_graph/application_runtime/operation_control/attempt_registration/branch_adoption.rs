//! Sealed branch-adoption stage pause and preparation-entry observer.
use super::*;
use crate::basis::WorthQueryProductBranch;

/// Durable owner-record boundaries within one single-branch adoption.
/// Grants no adoption, publication or mutation authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBranchAdoptionStageForTest {
    BeforeRetainBranch,
    AfterRetainBranch,
    AfterWorldOpen,
    AfterWorldPublished,
    AfterObligationReleased,
}

#[derive(Default)]
pub(super) struct PendingAdoptionStagePause {
    armed: AtomicBool,
    pending: Mutex<Option<(WorthQueryBranchAdoptionStageForTest, Arc<ExactAttemptPause>)>>,
}

#[derive(Default)]
pub(super) struct AdoptionPreparationObserverSlot {
    armed: AtomicBool,
    active: Mutex<Option<Arc<Mutex<Vec<WorthQueryProductBranch>>>>>,
}

/// Parks one real branch adoption at an exact durable owner-record boundary.
pub struct WorthQueryBranchAdoptionStagePauseForTest {
    slot: Arc<PendingAdoptionStagePause>,
    latch: Arc<ExactAttemptPause>,
}

/// Read-only observer recording branch selectors that enter actual preparation.
pub struct WorthQueryBranchAdoptionPreparationObserverForTest {
    slot: Arc<AdoptionPreparationObserverSlot>,
    recorded: Arc<Mutex<Vec<WorthQueryProductBranch>>>,
}

impl WorthQueryApplicationAttemptOperationControl {
    pub fn pause_branch_adoption_stage(
        &self,
        stage: WorthQueryBranchAdoptionStageForTest,
    ) -> WorthQueryBranchAdoptionStagePauseForTest {
        let latch = Arc::new(ExactAttemptPause {
            target: 1,
            state: Mutex::new((0, false, None)),
            backstop: Duration::from_secs(120),
            changed: Condvar::new(),
        });
        let mut pending = self
            .pending_branch_adoption_stage
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(
            pending.is_none(),
            "one branch-adoption stage pause may be armed at a time"
        );
        *pending = Some((stage, Arc::clone(&latch)));
        self.pending_branch_adoption_stage
            .armed
            .store(true, Ordering::Release);
        WorthQueryBranchAdoptionStagePauseForTest {
            slot: Arc::clone(&self.pending_branch_adoption_stage),
            latch,
        }
    }

    pub fn reach_branch_adoption_stage(
        &self,
        stage: WorthQueryBranchAdoptionStageForTest,
    ) {
        if !self
            .pending_branch_adoption_stage
            .armed
            .load(Ordering::Acquire)
        {
            return;
        }
        let mut pending = self
            .pending_branch_adoption_stage
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let Some((armed_stage, latch)) = pending.as_ref() else {
            return;
        };
        if *armed_stage != stage {
            return;
        }
        let latch = Arc::clone(latch);
        pending.take();
        self.pending_branch_adoption_stage
            .armed
            .store(false, Ordering::Release);
        drop(pending);
        let mut state = latch.state.lock().unwrap_or_else(|e| e.into_inner());
        state.0 += 1;
        latch.changed.notify_all();
        drop(state);
        latch.wait(false);
    }

    pub fn observe_branch_adoption_preparations(
        &self,
    ) -> WorthQueryBranchAdoptionPreparationObserverForTest {
        let recorded = Arc::new(Mutex::new(Vec::new()));
        let mut active = self
            .branch_adoption_preparation_observer
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        assert!(
            active.is_none(),
            "one branch-adoption preparation observer may be active at a time"
        );
        *active = Some(Arc::clone(&recorded));
        self.branch_adoption_preparation_observer
            .armed
            .store(true, Ordering::Release);
        WorthQueryBranchAdoptionPreparationObserverForTest {
            slot: Arc::clone(&self.branch_adoption_preparation_observer),
            recorded,
        }
    }

    pub fn record_branch_adoption_preparation(
        &self,
        branch: WorthQueryProductBranch,
    ) {
        if !self
            .branch_adoption_preparation_observer
            .armed
            .load(Ordering::Acquire)
        {
            return;
        }
        let active = self
            .branch_adoption_preparation_observer
            .active
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(recorded) = active.as_ref() {
            recorded
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(branch);
        }
    }
}

impl WorthQueryBranchAdoptionStagePauseForTest {
    pub fn wait_until_reached(&self, timeout: Duration) -> bool {
        self.latch.observe_arrivals(timeout)
    }

    pub fn release(&self) {
        let mut pending = self.slot.pending.lock().unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|(_, armed)| Arc::ptr_eq(armed, &self.latch))
        {
            pending.take();
            self.slot.armed.store(false, Ordering::Release);
        }
        drop(pending);
        self.latch.state.lock().unwrap_or_else(|e| e.into_inner()).1 = true;
        self.latch.changed.notify_all();
    }
}

impl Drop for WorthQueryBranchAdoptionStagePauseForTest {
    fn drop(&mut self) {
        self.release();
    }
}

impl WorthQueryBranchAdoptionPreparationObserverForTest {
    pub fn recorded_branches(&self) -> Vec<WorthQueryProductBranch> {
        self.recorded
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

impl Drop for WorthQueryBranchAdoptionPreparationObserverForTest {
    fn drop(&mut self) {
        let mut active = self.slot.active.lock().unwrap_or_else(|e| e.into_inner());
        if active
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &self.recorded))
        {
            active.take();
            self.slot.armed.store(false, Ordering::Release);
        }
    }
}

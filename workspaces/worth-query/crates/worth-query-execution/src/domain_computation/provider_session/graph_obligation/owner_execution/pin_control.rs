//! One-shot native-pin stimuli, owned and unwound by the arming guard.
use std::cell::RefCell;
use worth_relational::facade::snapshots::SnapshotHandle;

enum Action {
    Snapshot(SnapshotHandle),
    Cancel(worth_execution::CancellationSource),
}
struct Armed {
    skip: usize,
    action: Action,
}
thread_local! { static ARMED: RefCell<Option<Armed>> = const { RefCell::new(None) }; }
pub(in crate::domain_computation) struct PinControlGuardForTest(Option<Armed>);
impl PinControlGuardForTest {
    pub(in crate::domain_computation) fn unavailable_snapshot(
        skip: usize,
        snapshot: SnapshotHandle,
    ) -> Self {
        Self::install(Armed {
            skip,
            action: Action::Snapshot(snapshot),
        })
    }
    pub(in crate::domain_computation) fn cancellation(
        source: worth_execution::CancellationSource,
    ) -> Self {
        Self::install(Armed {
            skip: 0,
            action: Action::Cancel(source),
        })
    }
    fn install(armed: Armed) -> Self {
        assert!(crate::domain_computation::execution_observation::is_installed());
        Self(ARMED.with(|slot| slot.replace(Some(armed))))
    }
}
impl Drop for PinControlGuardForTest {
    fn drop(&mut self) {
        ARMED.with(|slot| {
            slot.replace(self.0.take());
        });
    }
}
pub(super) fn snapshot() -> Option<SnapshotHandle> {
    ARMED.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(armed) = slot.as_mut()
            && armed.skip != 0
        {
            armed.skip -= 1;
            return None;
        }
        match slot.take().map(|armed| armed.action) {
            Some(Action::Snapshot(snapshot)) => Some(snapshot),
            Some(Action::Cancel(source)) => {
                source.cancel();
                None
            }
            None => None,
        }
    })
}

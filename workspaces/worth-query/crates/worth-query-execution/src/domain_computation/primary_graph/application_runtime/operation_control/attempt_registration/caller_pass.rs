//! Law 2, scope: caller-pass arms belong to the controlled runtime receiver.
use crate::domain_computation::execution_observation::owned_control_scope;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct CallerPassControl {
    next_arm: AtomicU64,
    exhausted: AtomicU64,
    interleaving_owner: AtomicU64,
    interleaving: Mutex<Option<Box<dyn FnOnce() + Send>>>,
}
impl CallerPassControl {
    pub(in crate::domain_computation::primary_graph) fn arm_exhaustion(self: &Arc<Self>) {
        let owner = self
            .next_arm
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .expect("receiver arm identity exhausted")
            + 1;
        let receiver = Arc::downgrade(self);
        owned_control_scope::register(move || {
            if let Some(receiver) = receiver.upgrade() {
                let _ = receiver.exhausted.compare_exchange(
                    owner,
                    0,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
            }
        });
        self.exhausted.store(owner, Ordering::Release);
    }
    pub(in crate::domain_computation::primary_graph) fn take_exhaustion(&self) -> bool {
        // Unarmed: one atomic read; no allocation or lock.
        self.exhausted.load(Ordering::Acquire) != 0 && self.exhausted.swap(0, Ordering::AcqRel) != 0
    }
    pub(in crate::domain_computation::primary_graph) fn arm_interleaving(
        self: &Arc<Self>,
        action: impl FnOnce() + Send + 'static,
    ) {
        let owner = self
            .next_arm
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .expect("receiver arm identity exhausted")
            + 1;
        let receiver = Arc::downgrade(self);
        owned_control_scope::register(move || {
            if let Some(receiver) = receiver.upgrade() {
                let mut pending = receiver
                    .interleaving
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                if receiver.interleaving_owner.load(Ordering::Acquire) != owner {
                    return;
                }
                let action = pending.take();
                receiver.interleaving_owner.store(0, Ordering::Release);
                drop(pending);
                drop(action);
            }
        });
        let mut pending = self.interleaving.lock().unwrap_or_else(|e| e.into_inner());
        let previous = pending.replace(Box::new(action));
        self.interleaving_owner.store(owner, Ordering::Release);
        drop(pending);
        drop(previous);
    }
    pub(in crate::domain_computation::primary_graph) fn run_interleaving(&self) {
        // Unarmed: one atomic read; the callback runs after the mutex is released.
        if self.interleaving_owner.load(Ordering::Acquire) == 0 {
            return;
        }
        let mut pending = self.interleaving.lock().unwrap_or_else(|e| e.into_inner());
        let action = pending.take();
        self.interleaving_owner.store(0, Ordering::Release);
        drop(pending);
        if let Some(action) = action {
            action();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_computation::execution_observation::WorthQueryExecutionObservation as Observation;
    #[test]
    fn another_runtime_and_worker_cannot_consume_the_armed_receivers_controls() {
        let _guard = Observation::install_on_this_thread();
        let controlled = Arc::new(CallerPassControl::default());
        let other = Arc::new(CallerPassControl::default());
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let recorded = Arc::clone(&calls);
        controlled.arm_exhaustion();
        controlled.arm_interleaving(move || {
            recorded.fetch_add(1, Ordering::Relaxed);
        });
        std::thread::spawn(move || {
            assert!(!other.take_exhaustion());
            other.run_interleaving();
        })
        .join()
        .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        std::thread::spawn(move || {
            assert!(controlled.take_exhaustion());
            assert!(!controlled.take_exhaustion());
            controlled.run_interleaving();
            controlled.run_interleaving();
        })
        .join()
        .unwrap();
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    #[test]
    fn an_older_thread_guard_cannot_clear_a_later_receivers_arm() {
        let receiver = Arc::new(CallerPassControl::default());
        let armed = Arc::clone(&receiver);
        let (ready, reached) = std::sync::mpsc::channel();
        let (finish, release) = std::sync::mpsc::channel();
        let older = std::thread::spawn(move || {
            let _guard = Observation::install_on_this_thread();
            armed.arm_exhaustion();
            armed.arm_interleaving(|| panic!("the replaced callback must not run"));
            ready.send(()).unwrap();
            release.recv().unwrap();
        });
        reached.recv().unwrap();
        let _guard = Observation::install_on_this_thread();
        receiver.arm_exhaustion();
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let recorded = Arc::clone(&calls);
        receiver.arm_interleaving(move || {
            recorded.fetch_add(1, Ordering::Relaxed);
        });
        finish.send(()).unwrap();
        older.join().unwrap();
        assert!(receiver.take_exhaustion());
        receiver.run_interleaving();
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    #[test]
    fn panic_cleanup_clears_unreached_receiver_arms() {
        let receiver = Arc::new(CallerPassControl::default());
        assert!(std::panic::catch_unwind(|| {
            let _guard = Observation::install_on_this_thread();
            receiver.arm_exhaustion();
            receiver.arm_interleaving(|| panic!("a cleared arm must never run"));
            panic!("unwind before dispatch");
        })
        .is_err());
        assert!(!receiver.take_exhaustion());
        receiver.run_interleaving();
        assert!(receiver.interleaving.lock().unwrap().is_none());
    }
}

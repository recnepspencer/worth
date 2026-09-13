use std::sync::{Arc, Condvar, Mutex, PoisonError};

pub(super) struct WorthQueryPreparationCell<T> {
    state: Mutex<WorthQueryPreparationState<T>>,
    ready: Condvar,
}

struct WorthQueryPreparationState<T> {
    value: WorthQueryPreparedValue<T>,
    waiters: usize,
}

enum WorthQueryPreparedValue<T> {
    Vacant,
    Preparing,
    Ready(Arc<T>),
}

struct WorthQueryPreparationGuard<'a, T> {
    cell: &'a WorthQueryPreparationCell<T>,
    armed: bool,
}

impl<T> Default for WorthQueryPreparationCell<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(WorthQueryPreparationState {
                value: WorthQueryPreparedValue::Vacant,
                waiters: 0,
            }),
            ready: Condvar::new(),
        }
    }
}

impl<T> WorthQueryPreparationCell<T> {
    pub(super) fn get_or_try_init<E>(
        &self,
        initialize: impl FnOnce() -> Result<T, E>,
    ) -> Result<Arc<T>, E> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            match &state.value {
                WorthQueryPreparedValue::Ready(value) => return Ok(Arc::clone(value)),
                WorthQueryPreparedValue::Preparing => {
                    state.waiters += 1;
                    state = self
                        .ready
                        .wait(state)
                        .unwrap_or_else(PoisonError::into_inner);
                    state.waiters -= 1;
                }
                WorthQueryPreparedValue::Vacant => {
                    state.value = WorthQueryPreparedValue::Preparing;
                    break;
                }
            }
        }
        drop(state);

        let guard = WorthQueryPreparationGuard {
            cell: self,
            armed: true,
        };
        match initialize() {
            Ok(value) => Ok(guard.publish(value)),
            Err(error) => {
                drop(guard);
                Err(error)
            }
        }
    }

    #[cfg(test)]
    fn waiter_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .waiters
    }
}

impl<T> WorthQueryPreparationGuard<'_, T> {
    fn publish(mut self, value: T) -> Arc<T> {
        let value = Arc::new(value);
        self.cell
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .value = WorthQueryPreparedValue::Ready(Arc::clone(&value));
        self.armed = false;
        self.cell.ready.notify_all();
        value
    }
}

impl<T> Drop for WorthQueryPreparationGuard<'_, T> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self
            .cell
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if matches!(state.value, WorthQueryPreparedValue::Preparing) {
            state.value = WorthQueryPreparedValue::Vacant;
        }
        drop(state);
        self.cell.ready.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::sync::mpsc;
    use std::thread;

    #[test]
    fn admission_unwind_wakes_peer_and_allows_warm_reuse() {
        let cell = Arc::new(WorthQueryPreparationCell::<usize>::default());
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let panicking = {
            let cell = Arc::clone(&cell);
            thread::spawn(move || {
                catch_unwind(AssertUnwindSafe(|| {
                    let _: Result<Arc<usize>, ()> = cell.get_or_try_init(|| {
                        entered_tx.send(()).unwrap();
                        release_rx.recv().unwrap();
                        panic!("injected admission unwind")
                    });
                }))
            })
        };
        entered_rx.recv().unwrap();
        let waiting = {
            let cell = Arc::clone(&cell);
            thread::spawn(move || cell.get_or_try_init(|| Ok::<_, ()>(7)).unwrap())
        };
        while cell.waiter_count() == 0 {
            thread::yield_now();
        }
        release_tx.send(()).unwrap();
        assert!(panicking.join().unwrap().is_err());
        let recovered = waiting.join().unwrap();
        assert_eq!(*recovered, 7);
        let reused = cell
            .get_or_try_init(|| -> Result<usize, ()> { panic!("warm value must be reused") })
            .unwrap();
        assert!(Arc::ptr_eq(&recovered, &reused));
    }
}

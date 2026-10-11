//! Serialize shared test authority without sharing a failed test's poison.
use std::sync::{Mutex, MutexGuard, PoisonError};

pub(crate) fn guard(lock: &Mutex<()>) -> MutexGuard<'_, ()> {
    lock.lock().unwrap_or_else(PoisonError::into_inner)
}

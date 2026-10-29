// Serialize shared test authority without sharing a failed test's poison.
pub(crate) use {std::sync::Mutex, std::sync::MutexGuard, std::sync::PoisonError};

pub(crate) fn guard(lock: &Mutex<()>) -> MutexGuard<'_, ()> {
    lock.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
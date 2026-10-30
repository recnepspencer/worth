use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::operation_counters::{increment, MediaCounterCells};

impl MediaCounterCells {
    pub(super) fn file_handle_opened(
        self: &Arc<Self>,
        kind: super::NamespaceFileOpenKind,
    ) -> MediaFileHandleAccounting {
        match kind {
            super::NamespaceFileOpenKind::Existing => increment(&self.file_opens, 1),
            super::NamespaceFileOpenKind::CreatedNew => increment(&self.file_creates, 1),
        }
        acquire_live_handle(&self.live_file_handles, &self.peak_file_handles);
        MediaFileHandleAccounting {
            counters: Arc::clone(self),
        }
    }

    pub(super) fn directory_handle_opened(self: &Arc<Self>) -> MediaDirectoryHandleAccounting {
        increment(&self.directory_opens, 1);
        acquire_live_handle(&self.live_directory_handles, &self.peak_directory_handles);
        MediaDirectoryHandleAccounting {
            counters: Arc::clone(self),
        }
    }
}

#[derive(Debug)]
pub(super) struct MediaFileHandleAccounting {
    counters: Arc<MediaCounterCells>,
}

impl Drop for MediaFileHandleAccounting {
    fn drop(&mut self) {
        increment(&self.counters.file_closes, 1);
        release_live_handle(&self.counters.live_file_handles);
    }
}

#[derive(Debug)]
pub(super) struct MediaDirectoryHandleAccounting {
    counters: Arc<MediaCounterCells>,
}

impl Drop for MediaDirectoryHandleAccounting {
    fn drop(&mut self) {
        increment(&self.counters.directory_closes, 1);
        release_live_handle(&self.counters.live_directory_handles);
    }
}

fn acquire_live_handle(live: &AtomicU64, peak: &AtomicU64) {
    let current = {
        let update_counter = live;
        let mut value = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = Some(value.saturating_add(1));
            let Some(updated_value) = next_value else {
                break Err(value);
            };
            match update_counter.compare_exchange_weak(
                value,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => value = reloaded_value,
            }
        }
    }
    .unwrap_or(u64::MAX)
    .saturating_add(1);
    let _ = {
        let update_counter = peak;
        let mut value = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = (current > value).then_some(current);
            let Some(updated_value) = next_value else {
                break Err(value);
            };
            match update_counter.compare_exchange_weak(
                value,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => value = reloaded_value,
            }
        }
    };
}

fn release_live_handle(live: &AtomicU64) {
    let _ = {
        let update_counter = live;
        let mut value = update_counter.load(Ordering::Acquire);
        loop {
            let next_value = value.checked_sub(1);
            let Some(updated_value) = next_value else {
                break Err(value);
            };
            match update_counter.compare_exchange_weak(
                value,
                updated_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(previous) => break Ok(previous),
                Err(reloaded_value) => value = reloaded_value,
            }
        }
    };
}

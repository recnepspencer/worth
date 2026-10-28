//! Receiver-owned pre-runtime profile selection, confined to the calling thread.
use std::cell::Cell;
use std::num::NonZeroU64;

thread_local! {
    static WAL_SEGMENT_BYTES: Cell<Option<NonZeroU64>> = const { Cell::new(None) };
}

/// A certification fixture's physical profile. Already admitted runtimes keep
/// their owned policy; this scope affects only initialization and open calls.
#[derive(Debug, Clone, Copy)]
pub struct RelationalStoreCertificationProfile {
    wal_segment_bytes: NonZeroU64,
}

impl RelationalStoreCertificationProfile {
    pub const fn with_wal_segment_bytes(bytes: NonZeroU64) -> Self {
        Self {
            wal_segment_bytes: bytes,
        }
    }

    pub const fn wal_segment_bytes(self) -> NonZeroU64 {
        self.wal_segment_bytes
    }

    /// Use this profile for synchronous opens on this thread. Nested scopes
    /// and unwinding restore the previous selection; other threads are untouched.
    pub fn run<T>(&self, action: impl FnOnce() -> T) -> T {
        let previous = WAL_SEGMENT_BYTES.replace(Some(self.wal_segment_bytes));
        let _restore = PreviousWalSegmentBytes(previous);
        action()
    }
}

pub(super) fn wal_segment_bytes() -> Option<NonZeroU64> {
    WAL_SEGMENT_BYTES.get()
}

struct PreviousWalSegmentBytes(Option<NonZeroU64>);

impl Drop for PreviousWalSegmentBytes {
    fn drop(&mut self) {
        WAL_SEGMENT_BYTES.set(self.0);
    }
}

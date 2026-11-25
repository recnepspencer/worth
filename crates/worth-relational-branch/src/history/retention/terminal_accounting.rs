use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct RelationalExternalRetentionTerminalAccounting {
    explicit_releases: AtomicU64,
    dropped_releases: AtomicU64,
}

impl RelationalExternalRetentionTerminalAccounting {
    pub fn record_explicit_release(&self) {
        self.explicit_releases.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_dropped_release(&self) {
        self.dropped_releases.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> RelationalExternalRetentionTerminalCounts {
        RelationalExternalRetentionTerminalCounts {
            explicit_releases: self.explicit_releases.load(Ordering::Relaxed),
            dropped_releases: self.dropped_releases.load(Ordering::Relaxed),
        }
    }
}

/// Explicit and dropped external retention releases, as counted.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationalExternalRetentionTerminalCounts {
    pub explicit_releases: u64,
    pub dropped_releases: u64,
}

impl RelationalExternalRetentionTerminalCounts {
    pub(crate) fn total(self) -> u64 {
        self.explicit_releases.saturating_add(self.dropped_releases)
    }
}

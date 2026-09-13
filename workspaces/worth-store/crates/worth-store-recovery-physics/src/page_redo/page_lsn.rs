use worth_store_wal::{LogSequenceNumber, WalLsnRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageLsn {
    lsn: LogSequenceNumber,
}

impl PageLsn {
    pub const fn from_lsn(lsn: LogSequenceNumber) -> Self {
        Self { lsn }
    }

    pub const fn lsn(self) -> LogSequenceNumber {
        self.lsn
    }

    pub const fn is_at_or_beyond(self, frontier: Self) -> bool {
        self.lsn.get() >= frontier.lsn.get()
    }

    pub const fn is_not_beyond_wal_frontier(self, frontier: LogSequenceNumber) -> bool {
        self.lsn.get() <= frontier.get()
    }

    pub const fn is_covered_by_wal_range(self, range: WalLsnRange) -> bool {
        self.lsn.get() >= range.start().get() && self.lsn.get() < range.end_exclusive().get()
    }
}

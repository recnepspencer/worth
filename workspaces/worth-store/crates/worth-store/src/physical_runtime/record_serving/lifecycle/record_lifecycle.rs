use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordServingCounterSnapshot {
    owner_live: u64,
    reader_acquisitions: u64,
    readers_live: u64,
    read_session_acquisitions: u64,
    read_sessions_live: u64,
    scan_session_acquisitions: u64,
    scan_sessions_live: u64,
}

macro_rules! counter_accessors {
    ($($name:ident),+ $(,)?) => {
        $(pub const fn $name(self) -> u64 { self.$name })+
    };
}

impl RecordServingCounterSnapshot {
    counter_accessors!(
        owner_live,
        reader_acquisitions,
        readers_live,
        read_session_acquisitions,
        read_sessions_live,
        scan_session_acquisitions,
        scan_sessions_live,
    );

    pub const fn live_handles(self) -> u64 {
        self.readers_live
            .saturating_add(self.read_sessions_live)
            .saturating_add(self.scan_sessions_live)
    }
}

pub(in crate::physical_runtime) struct RecordServingOwner {
    counters: Arc<RecordServingCounterCells>,
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct RecordReaderLeaseFactory {
    counters: Arc<RecordServingCounterCells>,
}

impl RecordServingOwner {
    pub(in crate::physical_runtime) fn new() -> Self {
        Self {
            counters: Arc::new(RecordServingCounterCells::new()),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn reader(&self) -> RecordReaderLease {
        RecordReaderLease::acquire(Arc::clone(&self.counters))
    }

    pub(in crate::physical_runtime) fn reader_factory(&self) -> RecordReaderLeaseFactory {
        RecordReaderLeaseFactory {
            counters: Arc::clone(&self.counters),
        }
    }

    pub(in crate::physical_runtime::record_serving) fn observer(
        &self,
    ) -> Arc<RecordServingCounterCells> {
        Arc::clone(&self.counters)
    }

    pub(in crate::physical_runtime) fn into_terminal_snapshot(
        self,
    ) -> RecordServingCounterSnapshot {
        let counters = Arc::clone(&self.counters);
        drop(self);
        counters.snapshot()
    }
}

impl RecordReaderLeaseFactory {
    pub(in crate::physical_runtime::record_serving) fn reader(&self) -> RecordReaderLease {
        RecordReaderLease::acquire(Arc::clone(&self.counters))
    }
}

impl Drop for RecordServingOwner {
    fn drop(&mut self) {
        decrement(&self.counters.owner_live);
    }
}

pub(in crate::physical_runtime::record_serving) struct RecordReaderLease {
    counters: Arc<RecordServingCounterCells>,
}

impl RecordReaderLease {
    fn acquire(counters: Arc<RecordServingCounterCells>) -> Self {
        increment(&counters.reader_acquisitions);
        increment(&counters.readers_live);
        Self { counters }
    }

    pub(in crate::physical_runtime::record_serving) fn read_session(
        &self,
    ) -> RecordReadSessionLease {
        RecordReadSessionLease::acquire(Arc::clone(&self.counters))
    }

    pub(in crate::physical_runtime::record_serving) fn scan_session(
        &self,
    ) -> RecordScanSessionLease {
        RecordScanSessionLease::acquire(Arc::clone(&self.counters))
    }
}

impl Drop for RecordReaderLease {
    fn drop(&mut self) {
        decrement(&self.counters.readers_live);
    }
}

pub(in crate::physical_runtime::record_serving) struct RecordReadSessionLease {
    counters: Arc<RecordServingCounterCells>,
}

impl RecordReadSessionLease {
    fn acquire(counters: Arc<RecordServingCounterCells>) -> Self {
        increment(&counters.read_session_acquisitions);
        increment(&counters.read_sessions_live);
        Self { counters }
    }
}

impl Drop for RecordReadSessionLease {
    fn drop(&mut self) {
        decrement(&self.counters.read_sessions_live);
    }
}

pub(in crate::physical_runtime::record_serving) struct RecordScanSessionLease {
    counters: Arc<RecordServingCounterCells>,
}

impl RecordScanSessionLease {
    fn acquire(counters: Arc<RecordServingCounterCells>) -> Self {
        increment(&counters.scan_session_acquisitions);
        increment(&counters.scan_sessions_live);
        Self { counters }
    }
}

impl Drop for RecordScanSessionLease {
    fn drop(&mut self) {
        decrement(&self.counters.scan_sessions_live);
    }
}

pub(in crate::physical_runtime::record_serving) struct RecordServingCounterCells {
    owner_live: AtomicU64,
    reader_acquisitions: AtomicU64,
    readers_live: AtomicU64,
    read_session_acquisitions: AtomicU64,
    read_sessions_live: AtomicU64,
    scan_session_acquisitions: AtomicU64,
    scan_sessions_live: AtomicU64,
}

impl RecordServingCounterCells {
    fn new() -> Self {
        Self {
            owner_live: AtomicU64::new(1),
            reader_acquisitions: AtomicU64::new(0),
            readers_live: AtomicU64::new(0),
            read_session_acquisitions: AtomicU64::new(0),
            read_sessions_live: AtomicU64::new(0),
            scan_session_acquisitions: AtomicU64::new(0),
            scan_sessions_live: AtomicU64::new(0),
        }
    }

    pub(in crate::physical_runtime::record_serving::lifecycle) fn snapshot(
        &self,
    ) -> RecordServingCounterSnapshot {
        RecordServingCounterSnapshot {
            owner_live: load(&self.owner_live),
            reader_acquisitions: load(&self.reader_acquisitions),
            readers_live: load(&self.readers_live),
            read_session_acquisitions: load(&self.read_session_acquisitions),
            read_sessions_live: load(&self.read_sessions_live),
            scan_session_acquisitions: load(&self.scan_session_acquisitions),
            scan_sessions_live: load(&self.scan_sessions_live),
        }
    }
}

fn increment(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::AcqRel);
}

fn decrement(counter: &AtomicU64) {
    counter
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
            current.checked_sub(1)
        })
        .expect("record-serving resource release must match one acquisition");
}

fn load(counter: &AtomicU64) -> u64 {
    counter.load(Ordering::Acquire)
}

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::facade::{RuntimeBridgeBuilder, SnapshotReaderPool, TruthSnapshotIdentity};
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};

#[derive(Clone)]
struct CountingSnapshotReaderPool<S> {
    source: S,
    acquire_count: Arc<AtomicUsize>,
    release_count: Arc<AtomicUsize>,
}

impl<S> CountingSnapshotReaderPool<S> {
    fn new(source: S) -> Self {
        Self {
            source,
            acquire_count: Arc::new(AtomicUsize::new(0)),
            release_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn acquire_count(&self) -> usize {
        self.acquire_count.load(Ordering::SeqCst)
    }

    fn release_count(&self) -> usize {
        self.release_count.load(Ordering::SeqCst)
    }
}

impl<S: crate::facade::SnapshotReadSource> SnapshotReaderPool for CountingSnapshotReaderPool<S> {
    fn acquire(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Box<dyn crate::facade::TruthSnapshotReader>,
        crate::facade::RelationalBridgeSourceError,
    > {
        self.acquire_count.fetch_add(1, Ordering::SeqCst);
        crate::facade::SnapshotReadSource::open_snapshot(&self.source, identity, execution)
    }

    fn release(&self, _reader: Box<dyn crate::facade::TruthSnapshotReader>) {
        self.release_count.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn bridge_snapshot_reader_pool_is_used_when_configured() {
    let serial = crate::snapshot::test_serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let pool = CountingSnapshotReaderPool::new(truth.source.clone());
    let runtime = RuntimeBridgeBuilder::new()
        .with_relational_source(truth.source)
        .with_snapshot_reader_pool(pool.clone())
        .with_signal_sink(RecordingSignalBridgeSink::default())
        .register_mapping(NamedTruth::mapping())
        .build()
        .unwrap();

    runtime.route(commit, execution).unwrap();

    assert_eq!(pool.acquire_count(), 1);
    assert_eq!(pool.release_count(), 1);
}

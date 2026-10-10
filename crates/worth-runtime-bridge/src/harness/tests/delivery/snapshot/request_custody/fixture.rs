//! An explicit external-source fixture; the Bridge wrappers own custody.
use super::super::super::{committed_patch, registration, snapshot};
use crate::facade::*;
use crate::harness::fixtures::{InMemoryRelationalBridgeSource, RecordingSignalBridgeSink};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

#[derive(Default)]
pub(super) struct Contacts {
    pub(super) opens: AtomicUsize,
    pub(super) reads: AtomicUsize,
    pub(super) sinks: AtomicUsize,
    pub(super) order: Mutex<Vec<&'static str>>,
}
impl Contacts {
    pub(super) fn reset(&self) {
        self.opens.store(0, Ordering::SeqCst);
        self.reads.store(0, Ordering::SeqCst);
        self.sinks.store(0, Ordering::SeqCst);
        self.order.lock().unwrap().clear();
    }
    pub(super) fn counts(&self) -> (usize, usize, usize) {
        (
            self.opens.load(Ordering::SeqCst),
            self.reads.load(Ordering::SeqCst),
            self.sinks.load(Ordering::SeqCst),
        )
    }
}
struct CountingSource(InMemoryRelationalBridgeSource, Arc<Contacts>);
impl SnapshotReadSource for CountingSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        self.1.opens.fetch_add(1, Ordering::SeqCst);
        self.1.order.lock().unwrap().push("open");
        Ok(Box::new(CountingReader(
            self.0.open_snapshot(identity, request)?,
            self.1.clone(),
        )))
    }
}
struct CountingReader(Box<dyn TruthSnapshotReader>, Arc<Contacts>);
impl TruthSnapshotReader for CountingReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.0.snapshot_identity()
    }
    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.1.reads.fetch_add(1, Ordering::SeqCst);
        self.1.order.lock().unwrap().push("read");
        self.0.read_packet(packet, request)
    }
}
struct CountingSink(RecordingSignalBridgeSink, Arc<Contacts>);
impl InvalidationSink for CountingSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        request: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        self.1.sinks.fetch_add(1, Ordering::SeqCst);
        self.1.order.lock().unwrap().push("sink");
        self.0.deliver_invalidation(delivery, request)
    }
}
pub(super) struct Fixture {
    pub(super) runtime: RuntimeBridge,
    pub(super) routes: Vec<BridgePlannedRoute>,
    pub(super) context: BridgeSnapshotContext<Box<dyn TruthSnapshotReader>>,
    pub(super) contacts: Arc<Contacts>,
    pub(super) sink: RecordingSignalBridgeSink,
}
pub(super) fn fixture() -> Fixture {
    let source = InMemoryRelationalBridgeSource::default();
    for (id, value) in [(1, "Ada"), (2, "Grace")] {
        source.insert_committed_patch(committed_patch(
            crate::truth_identity_fixtures::truth_commit(id),
            crate::truth_identity_fixtures::truth_patch(id),
            crate::truth_identity_fixtures::truth_snapshot(id, 1),
            worth_foundational::facade::FieldKey::new("name".to_owned()).unwrap(),
        ));
        source.insert_snapshot(snapshot(
            crate::truth_identity_fixtures::truth_snapshot(id, 1),
            value,
        ));
    }
    let contacts = Arc::new(Contacts::default());
    let sink = RecordingSignalBridgeSink::default();
    let runtime = RuntimeBridge::builder()
        .with_committed_patch_source(source.clone())
        .with_snapshot_read_source(CountingSource(source.clone(), contacts.clone()))
        .with_signal_sink(CountingSink(sink.clone(), contacts.clone()))
        .register_mapping(registration())
        .build()
        .unwrap();
    let serial = crate::snapshot::test_serial_request();
    let request = worth_execution::ExecutionRequest::serial(&serial);
    let routes = (1..=2)
        .map(|id| {
            runtime
                .plan_committed_patch(
                    BridgeRouteRequest::for_commit(crate::truth_identity_fixtures::truth_commit(
                        id,
                    )),
                    request,
                )
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert!(routes
        .iter()
        .all(|route| !route.read_packet().reads().is_empty()));
    let reader = CountingSource(source, contacts.clone())
        .open_snapshot(
            &crate::truth_identity_fixtures::truth_snapshot(1, 1),
            request,
        )
        .unwrap();
    let context = BridgeSnapshotContext::bind(reader);
    contacts.reset();
    Fixture {
        runtime,
        routes,
        context,
        contacts,
        sink,
    }
}
